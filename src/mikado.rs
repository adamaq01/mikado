use crate::handlers::save::process_save;
use crate::handlers::scores::process_scores;
use crate::sys::{
    property_clear_error, property_mem_write, property_node_name, property_node_refer,
    property_query_size, property_search, property_set_flag, NodeType,
};
use crate::types::game::Property;
use crate::{helpers, CONFIGURATION, TACHI_STATUS_URL};
use anyhow::Result;
use lazy_static::lazy_static;
use log::{debug, error, info};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

static USER: AtomicU64 = AtomicU64::new(0);

pub fn hook_init() -> Result<()> {
    if !CONFIGURATION.general.enable {
        return Ok(());
    }

    // Trying to reach Tachi API
    let response: serde_json::Value =
        helpers::request_tachi("GET", TACHI_STATUS_URL.as_str(), None::<()>)?;
    let user = response["body"]["whoami"]
        .as_u64()
        .ok_or(anyhow::anyhow!("Couldn't parse user from Tachi response"))?;
    USER.store(user, Ordering::Relaxed);
    info!("Tachi API successfully reached, user {}", user);

    // Initializing function detours
    crochet::enable!(property_destroy_hook)
        .map_err(|err| anyhow::anyhow!("Could not enable function detour: {:#}", err))?;

    info!("Hook successfully initialized");

    Ok(())
}

pub fn hook_release() -> Result<()> {
    if !CONFIGURATION.general.enable {
        return Ok(());
    }

    if crochet::is_enabled!(property_destroy_hook) {
        crochet::disable!(property_destroy_hook)
            .map_err(|err| anyhow::anyhow!("Could not disable function detour: {:#}", err))?;
    }

    Ok(())
}

#[crochet::hook("avs2-core.dll", "XCgsqzn0000091")]
pub unsafe fn property_destroy_hook(property: *mut ()) -> i32 {
    if property.is_null() {
        return 0;
    }

    let game_node = property_search(property, std::ptr::null(), b"/call/game\0".as_ptr());
    if game_node.is_null() {
        property_clear_error(property);
        return call_original!(property);
    }

    let mut buffer = [0u8; 256];
    let result = property_node_name(game_node, buffer.as_mut_ptr(), buffer.len() as u32);
    if result < 0 {
        return call_original!(property);
    }

    let name = {
        let result = std::str::from_utf8(&buffer[0..4]);
        if let Err(err) = result {
            error!("Could not convert buffer to string: {:#}", err);
            return call_original!(property);
        }

        result.unwrap()
    };
    if name != "game" {
        return call_original!(property);
    }

    let result = property_node_refer(
        property,
        game_node,
        b"method@\0".as_ptr(),
        NodeType::NodeAttr,
        buffer.as_mut_ptr() as *mut (),
        256,
    );
    if result < 0 {
        return call_original!(property);
    }

    let method = {
        let result = std::str::from_utf8(&buffer[0..11]);
        if let Err(err) = result {
            error!("Could not convert buffer to string: {:#}", err);
            return call_original!(property);
        }

        result.unwrap().replace('\0', "")
    };
    debug!("Intercepted Game Method: {}", method);
    if method != "sv6_save_m" && (!CONFIGURATION.general.export_class || method != "sv6_save") {
        return call_original!(property);
    }

    property_set_flag(property, 0x800, 0x008);

    let size = property_query_size(property);
    if size < 0 {
        property_set_flag(property, 0x008, 0x800);
        return call_original!(property);
    }

    let buffer = vec![0u8; size as usize];
    let result = property_mem_write(property, buffer.as_ptr() as *mut u8, buffer.len() as u32);
    property_set_flag(property, 0x008, 0x800);
    if result < 0 {
        return call_original!(property);
    }

    // Read buf to string
    let property_str = {
        let result = std::str::from_utf8(&buffer);
        if let Err(err) = result {
            error!("Could not convert buffer to string: {:#}", err);
            return call_original!(property);
        }

        result.unwrap()
    };

    debug!("Processing property: {}", property_str);
    if let Err(err) = match method.as_str() {
        "sv6_save_m" => serde_json::from_str::<Property>(property_str)
            .map_err(|err| anyhow::anyhow!("Could not parse property: {:#}", err))
            .and_then(|prop| {
                process_scores(
                    prop.call
                        .game
                        .left()
                        .ok_or(anyhow::anyhow!("Could not process scores property"))?,
                )
            }),
        "sv6_save" => serde_json::from_str::<Property>(property_str)
            .map_err(|err| anyhow::anyhow!("Could not parse property: {:#}", err))
            .and_then(|prop| {
                process_save(
                    prop.call
                        .game
                        .right()
                        .ok_or(anyhow::anyhow!("Could not process save property"))?,
                )
            }),
        _ => unreachable!(),
    } {
        error!("{:#}", err);
    }

    call_original!(property)
}
