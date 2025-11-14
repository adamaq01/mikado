use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Configuration {
    pub general: GeneralConfiguration,
    pub cards: HashMap<String, CardConfiguration>,
    pub tachi: TachiConfiguration,
}

impl Configuration {
    pub fn load() -> Result<Self> {
        if !Path::new("mikado.toml").exists() {
            File::create("mikado.toml")
                .and_then(|mut file| file.write_all(include_bytes!("../mikado.toml")))
                .map_err(|err| anyhow::anyhow!("Could not create default config file: {}", err))?;
        }

        confy::load_path("mikado.toml")
            .map_err(|err| anyhow::anyhow!("Could not load config: {}", err))
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeneralConfiguration {
    #[serde(default = "default_true")]
    pub enable: bool,
    #[serde(default)]
    pub export_class: bool,
    #[serde(default)]
    pub inject_cloud_pbs: bool,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

fn default_true() -> bool {
    true
}

fn default_timeout() -> u64 {
    3000
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CardConfiguration {
    pub api_key: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TachiConfiguration {
    // TODO: it could be useful to move base_url to CardConfiguration as well
    //       in case different users want different Tachi instances
    pub base_url: String,
    pub status: String,
    pub import: String,
    pub pbs: String,
}
