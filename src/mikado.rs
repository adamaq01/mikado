use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::RwLock;

use anyhow::Result;
use bytes::Bytes;
use kbinxml::{CompressionType, EncodingType, Node, Options, Value};
use log::{debug, error, info, warn};

use crate::handlers::save::process_save;
use crate::handlers::scores::process_scores;
use crate::sys::{
    property_clear_error, property_mem_write, property_node_name, property_node_refer,
    property_query_size, property_search, property_set_flag, NodeType,
};
use crate::types::game::Property;
use crate::{helpers, CONFIGURATION, TACHI_STATUS_URL};

pub static USER: AtomicU64 = AtomicU64::new(0);
pub static CURRENT_CARD_ID: RwLock<Option<String>> = RwLock::new(None);
pub static IS_VALKYRIE: AtomicBool = AtomicBool::new(false);

pub fn hook_init(ea3_node: *const ()) -> Result<()> {
    if !CONFIGURATION.general.enable {
        return Ok(());
    }

    if let Some((model, dest, spec, revision, ext)) =
        helpers::read_node_str(ea3_node, b"/soft/model\0".as_ptr(), 3).and_then(|model| {
            let dest = helpers::read_node_str(ea3_node, b"/soft/dest\0".as_ptr(), 1)?;
            let spec = helpers::read_node_str(ea3_node, b"/soft/spec\0".as_ptr(), 1)?;
            let revision = helpers::read_node_str(ea3_node, b"/soft/rev\0".as_ptr(), 1)?;
            let ext = helpers::read_node_str(ea3_node, b"/soft/ext\0".as_ptr(), 10)?
                .parse::<u64>()
                .unwrap_or(0);
            Some((model, dest, spec, revision, ext))
        })
    {
        IS_VALKYRIE.store(spec == "G" || spec == "H", Ordering::Relaxed);

        if model != "KFC" || revision == "O" || revision == "X" || ext < 2022083000 {
            error!(
                "Unsupported model/revision/ext '{}:{}:{}:{}:{}', hook will not be enabled",
                model, dest, spec, revision, ext
            );
            return Ok(());
        } else if spec == "G" || spec == "H" {
            info!(
                "Detected game software '{}:{}:{}:{}:{} (Valkyrie Model)'",
                model, dest, spec, revision, ext
            );
        } else {
            info!(
                "Detected game software '{}:{}:{}:{}:{}'",
                model, dest, spec, revision, ext
            );
        }
    } else {
        warn!("Could not read game version, hook might not work properly");
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
    if CONFIGURATION.general.inject_cloud_pbs {
        debug!("PBs injection enabled");
        crochet::enable!(property_mem_read_hook)
            .map_err(|err| anyhow::anyhow!("Could not enable function detour: {:#}", err))?;
    }

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

    if crochet::is_enabled!(property_mem_read_hook) {
        crochet::disable!(property_mem_read_hook)
            .map_err(|err| anyhow::anyhow!("Could not disable function detour: {:#}", err))?;
    }

    Ok(())
}

static LOAD: AtomicBool = AtomicBool::new(false);
static LOAD_M: AtomicBool = AtomicBool::new(false);
static COMMON: AtomicBool = AtomicBool::new(false);

#[crochet::hook("avs2-core.dll", "XCgsqzn00000b7")]
pub unsafe fn property_mem_read_hook(
    ptr: *const (),
    something: i32,
    flags: i32,
    data: *const u8,
    size: u32,
) -> *const () {
    let load = LOAD.load(Ordering::SeqCst);
    let load_m = LOAD_M.load(Ordering::SeqCst);
    let common = COMMON.load(Ordering::SeqCst);
    if !load && !load_m && !common {
        return call_original!(ptr, something, flags, data, size);
    }

    let bytes = std::slice::from_raw_parts(ptr as *const u8, something as usize).to_vec();
    match property_mem_read_hook_wrapped(bytes, load, load_m, common) {
        Some(Ok(response)) => {
            call_original!(
                response.as_ptr() as *const (),
                response.len() as i32,
                flags,
                data,
                size
            )
        }
        Some(Err(err)) => {
            error!(
                "Error while processing an important e-amusement response node: {:#}",
                err
            );
            call_original!(ptr, something, flags, data, size)
        }
        None => call_original!(ptr, something, flags, data, size),
    }
}

fn build_response(
    original_signature: &[u8],
    response: Node,
    encoding: EncodingType,
) -> Result<Vec<u8>> {
    if kbinxml::is_binary_xml(original_signature) {
        let bytes = kbinxml::to_binary_with_options(
            Options::new(CompressionType::from_byte(original_signature[1])?, encoding),
            &response,
        )?;
        Ok(bytes)
    } else {
        let bytes = kbinxml::to_text_xml(&response)?;
        Ok(bytes)
    }
}

#[allow(clippy::manual_map)]
pub unsafe fn property_mem_read_hook_wrapped(
    original: Vec<u8>,
    load: bool,
    load_m: bool,
    common: bool,
) -> Option<Result<Vec<u8>>> {
    let original_signature = original[..2].to_vec();
    let (mut root, encoding) = kbinxml::from_bytes(Bytes::from(original))
        .and_then(|(node, encoding)| node.as_node().map(|node| (node, encoding)))
        .ok()?;

    if common
        .then(|| root.pointer(&["game", "event"]))
        .flatten()
        .is_some()
    {
        Some((|| {
            let events = root
                .pointer_mut(&["game", "event"])
                .expect("Could not find events node");

            events.children_mut().retain(|info| {
                if let Some(Value::String(event_id)) = info
                    .pointer(&["event_id"])
                    .and_then(|event_id| event_id.value())
                {
                    event_id != "CLOUD_LINK_ENABLE"
                } else {
                    true
                }
            });
            events.children_mut().push(Node::with_nodes(
                "info",
                vec![Node::with_value(
                    "event_id",
                    Value::String("CLOUD_LINK_ENABLE".to_string()),
                )],
            ));
            let response = build_response(&original_signature, root, encoding)?;
            COMMON.store(false, Ordering::Relaxed);

            Ok(response)
        })())
    } else if load
        .then(|| root.pointer(&["game", "code"]))
        .flatten()
        .is_some()
    {
        Some((|| {
            let game = root
                .pointer_mut(&["game"])
                .expect("Could not find game node");
            game.children_mut().retain(|node| {
                return node.key() != "cloud";
            });
            game.children_mut().push(Node::with_nodes(
                "cloud",
                vec![Node::with_value("relation", Value::S8(1))],
            ));
            let response = build_response(&original_signature, root, encoding)?;
            LOAD.store(false, Ordering::Relaxed);

            Ok(response)
        })())
    } else if let Some(music) = load_m.then(|| root.pointer(&["game", "music"])).flatten() {
        Some((|| {
            let user = USER.load(Ordering::SeqCst).to_string();
            let response = crate::cloudlink::process_pbs(user.as_str(), music)?;
            let response = build_response(&original_signature, response, encoding)?;
            LOAD_M.store(false, Ordering::Relaxed);

            Ok(response)
        })())
    } else {
        None
    }
}

#[crochet::hook("avs2-core.dll", "XCgsqzn0000091")]
pub unsafe fn property_destroy_hook(property: *mut ()) -> i32 {
    if property.is_null() {
        return 0;
    }

    let node = property_search(property, std::ptr::null(), b"/call/game\0".as_ptr());
    let node = if node.is_null() {
        property_search(property, std::ptr::null(), b"/call/cardmng\0".as_ptr())
    } else {
        node
    };
    if node.is_null() {
        property_clear_error(property);
        return call_original!(property);
    }

    let mut buffer = [0u8; 256];
    let result = property_node_name(node, buffer.as_mut_ptr(), buffer.len() as u32);
    if result < 0 {
        return call_original!(property);
    }

    let name = {
        let result = std::str::from_utf8(&buffer[0..32]);
        if let Err(err) = result {
            error!("Could not convert buffer to string: {:#}", err);
            return call_original!(property);
        }

        result.unwrap().replace('\0', "")
    };
    if name != "game" && name != "cardmng" {
        return call_original!(property);
    }

    let result = property_node_refer(
        property,
        node,
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
    debug!("Intercepted '{}' method: {}", name, method);

    if name == "cardmng" {
        if method != "inquire" {
            return call_original!(property);
        }

        let result = property_node_refer(
            property,
            node,
            b"cardid@\0".as_ptr(),
            NodeType::NodeAttr,
            buffer.as_mut_ptr() as *mut (),
            256,
        );
        if result < 0 {
            return call_original!(property);
        }

        let cardid = {
            let result = std::str::from_utf8(&buffer[..32]);
            if let Err(err) = result {
                error!("Could not convert buffer to string: {:#}", err);
                return call_original!(property);
            }

            result.unwrap().replace('\0', "")
        };

        if let Ok(mut guard) = CURRENT_CARD_ID.write() {
            debug!("Set current card id to {}", cardid);
            *guard = Some(cardid);
        } else {
            warn!("Could not acquire write lock on current card id");
        }

        return call_original!(property);
    }

    if CONFIGURATION.general.inject_cloud_pbs {
        if method == "sv6_load_m" {
            LOAD_M.store(true, Ordering::Relaxed);
        } else if method == "sv6_common" {
            COMMON.store(true, Ordering::Relaxed);
        } else if method == "sv6_load" {
            LOAD.store(true, Ordering::Relaxed);
        }
    }

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
