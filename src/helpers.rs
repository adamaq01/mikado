use crate::mikado::CURRENT_CARD_ID;
use crate::sys::{property_node_refer, NodeType};
use crate::CONFIGURATION;
use anyhow::Result;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

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
    body: Option<T>,
) -> Result<ureq::Response>
where
    T: Serialize + Debug,
{
    let agent = request_agent();

    let method = method.as_ref();
    let url = url.as_ref();
    debug!("{} request to {} with body: {:#?}", method, url, body);

    let authorization = format!("Bearer {}", CONFIGURATION.tachi.api_key);
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

pub fn call_tachi<T>(method: impl AsRef<str>, url: impl AsRef<str>, body: Option<T>) -> Result<()>
where
    T: Serialize + Debug,
{
    let response = request(method, url, body)?;
    let response: serde_json::Value = response.into_json()?;
    debug!("Tachi API response: {:#?}", response);

    Ok(())
}

pub fn request_tachi<T, R>(
    method: impl AsRef<str>,
    url: impl AsRef<str>,
    body: Option<T>,
) -> Result<R>
where
    T: Serialize + Debug,
    R: for<'de> Deserialize<'de> + Debug,
{
    let response = request(method, url, body)?;
    let response = response.into_json()?;
    debug!("Tachi API response: {:#?}", response);

    Ok(response)
}

pub fn get_current_card_id() -> Option<String> {
    let guard = CURRENT_CARD_ID.read().unwrap_or_else(|err| {
        error!("Current card ID RwLock is poisoned: {:#}", err);
        err.into_inner()
    });

    guard.clone()
}

pub fn read_node_str(node: *const (), path: *const u8, length: usize) -> Option<String> {
    let mut buffer = [0u8; 32];
    let result = unsafe {
        property_node_refer(
            node,
            node,
            path,
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
