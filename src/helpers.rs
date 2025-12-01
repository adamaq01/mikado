use crate::configuration::{Profile, ProfileConfiguration};
use crate::mikado::{User, CURRENT_USER};
use crate::sys::{property_node_refer, NodeType};
use crate::{CARD_PROFILES, CONFIGURATION};
use anyhow::Result;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use winapi::ctypes::c_char;

pub fn request_agent() -> ureq::Agent {
    let timeout = CONFIGURATION.general.timeout;
    let timeout = if timeout > 10000 { 10000 } else { timeout };

    ureq::builder()
        .timeout(std::time::Duration::from_millis(timeout))
        .build()
}

fn request<T>(
    method: impl AsRef<str>,
    url: impl AsRef<str>,
    key: impl AsRef<str>,
    body: Option<T>,
) -> Result<ureq::Response>
where
    T: Serialize + Debug,
{
    let agent = request_agent();

    let method = method.as_ref();
    let url = url.as_ref();
    debug!("{method} request to {url} with body: {body:#?}");

    let authorization = format!("Bearer {}", key.as_ref());
    let request = agent
        .request(method, url)
        .set("Authorization", authorization.as_str());
    let response = match body {
        Some(body) => request.send_json(body),
        None => request.call(),
    }
    .map_err(|err| anyhow::anyhow!("Could not reach Tachi API: {:#}", err))?;

    Ok(response)
}

pub fn call_tachi<T>(method: impl AsRef<str>, url: impl AsRef<str>, key: impl AsRef<str>, body: Option<T>) -> Result<()>
where
    T: Serialize + Debug,
{
    let response = request(method, url, key, body)?;
    let response: serde_json::Value = response.into_json()?;
    debug!("Tachi API response: {response:#?}");

    Ok(())
}

pub fn request_tachi<T, R>(
    method: impl AsRef<str>,
    url: impl AsRef<str>,
    key: impl AsRef<str>,
    body: Option<T>,
) -> Result<R>
where
    T: Serialize + Debug,
    R: for<'de> Deserialize<'de> + Debug,
{
    let response = request(method, url, key, body)?;
    let response = response.into_json()?;
    debug!("Tachi API response: {response:#?}");

    Ok(response)
}

pub fn get_current_user() -> Option<User> {
    let guard = CURRENT_USER.read().unwrap_or_else(|err| {
        error!("Current user RwLock is poisoned: {err:#}");
        err.into_inner()
    });

    guard.clone()
}

pub fn get_profile(card: impl AsRef<str>) -> Option<Profile> {
    // find the card in a profile ...
    CARD_PROFILES
        .get(card.as_ref())
        .cloned()
        // ... or fallback to single-user config, if present
        .or_else(|| {
            let cards_config = CONFIGURATION.cards.as_ref()?;
            let api_key = CONFIGURATION.tachi.api_key.as_ref()?;

            let is_whitelisted = cards_config.whitelist.is_empty()
                || cards_config.whitelist.contains(&card.as_ref().to_string());

            is_whitelisted.then(|| Profile {
                name: "default".to_string(),
                config: ProfileConfiguration {
                    api_key: api_key.to_string(),
                    cards: cards_config.whitelist.clone(),
                },
            })
        })
}

pub unsafe fn read_node_str(node: *const (), path: *const c_char, length: usize) -> Option<String> {
    let mut buffer = [0u8; 32];
    let result = unsafe {
        property_node_refer(
            node,
            node,
            path as _,
            NodeType::NodeStr,
            buffer.as_mut_ptr() as *mut (),
            32,
        )
    };

    if result < 0 {
        return None;
    }

    Some(String::from_utf8_lossy(&buffer[..length]).to_string())
}
