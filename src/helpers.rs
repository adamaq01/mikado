use crate::CONFIGURATION;
use anyhow::Result;
use log::debug;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

fn request<T>(
    method: impl AsRef<str>,
    url: impl AsRef<str>,
    body: Option<T>,
) -> Result<ureq::Response>
where
    T: Serialize + Debug,
{
    let timeout = CONFIGURATION.general.timeout;
    let timeout = if timeout > 10000 { 10000 } else { timeout };
    let agent = ureq::builder()
        .timeout(std::time::Duration::from_millis(timeout))
        .build();

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
