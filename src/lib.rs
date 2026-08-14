mod cloudlink;
mod configuration;
mod handlers;
mod helpers;
mod log;
mod mikado;
mod sys;
mod types;

use std::collections::HashMap;

use crate::log::Logger;
use crate::mikado::{hook_init, hook_release};
use crate::types::user::Profile;
use ::log::{error, info, warn};
use configuration::Configuration;
use std::sync::LazyLock;
use url::Url;
use windows::Win32::Foundation::{HINSTANCE, TRUE};
use windows::Win32::System::Console::AllocConsole;
use windows::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};
use windows::core::BOOL;

pub static CONFIGURATION: LazyLock<Configuration> = LazyLock::new(|| {
    let result = Configuration::load();
    if let Err(err) = result {
        error!("{err:#}");
        std::process::exit(1);
    }

    result.unwrap()
});

pub static CARD_PROFILES: LazyLock<HashMap<String, Profile>> = LazyLock::new(|| {
    let mut cards: HashMap<String, Profile> = HashMap::new();

    for (profile_name, profile_config) in &CONFIGURATION.profiles {
        for card in &profile_config.cards {
            if let Some(cards_config) = &CONFIGURATION.cards
                && !cards_config.whitelist.is_empty()
                && cards_config.whitelist.contains(card)
            {
                warn!(
                    "Card {} is in the default [cards] whitelist and also assigned to profile \"{}\". The profile assignment will be ignored. Remove it from the [cards] whitelist if you want it to use the profile.",
                    card, profile_name
                );
                continue;
            }
            if let Some(existing_profile) = cards.get(card.as_str()) {
                warn!(
                    "Card {} is already assigned to profile \"{}\" but appears again in profile \"{}\". Ignoring.",
                    card, existing_profile.name, profile_name
                );
                continue;
            }
            cards.insert(
                card.to_string(),
                Profile {
                    name: profile_name.clone(),
                    api_key: profile_config.api_key.clone(),
                },
            );
        }
    }

    cards
});

pub static TACHI_STATUS_URL: LazyLock<String> = LazyLock::new(|| {
    let result = Url::parse(&CONFIGURATION.tachi.base_url)
        .and_then(|url| url.join(&CONFIGURATION.tachi.status));
    if let Err(err) = result {
        error!("Could not parse Tachi status URL: {err:#}");
        std::process::exit(1);
    }

    result.unwrap().to_string()
});

pub static TACHI_IMPORT_URL: LazyLock<String> = LazyLock::new(|| {
    let result = Url::parse(&CONFIGURATION.tachi.base_url)
        .and_then(|url| url.join(&CONFIGURATION.tachi.import));
    if let Err(err) = result {
        error!("Could not parse Tachi import URL: {err:#}");
        std::process::exit(1);
    }

    result.unwrap().to_string()
});

pub static TACHI_PBS_URL: LazyLock<String> = LazyLock::new(|| {
    let result = Url::parse(&CONFIGURATION.tachi.base_url)
        .and_then(|url| url.join(&CONFIGURATION.tachi.pbs));
    if let Err(err) = result {
        error!("Could not parse Tachi PBS URL: {err:#}");
        std::process::exit(1);
    }

    result
        .unwrap()
        .to_string()
        .replace("%7B", "{")
        .replace("%7D", "}")
});

fn print_infos() {
    info!(
        "Starting Mikado v{}-{} by adamaq01",
        env!("CARGO_PKG_VERSION"),
        option_env!("VERGEN_GIT_DESCRIBE").unwrap_or("unknown")
    );

    if let Some(build_date) = option_env!("VERGEN_BUILD_DATE") {
        info!("Build date: {build_date}");
    }
}

fn check_for_update() -> anyhow::Result<()> {
    let commit_hash = option_env!("VERGEN_GIT_SHA").unwrap_or("unknown");
    let latest_commit_hash = helpers::request_agent()
        .get("https://api.github.com/repos/adamaq01/mikado/releases/latest")
        .call()?
        .body_mut()
        .read_json::<serde_json::Value>()?
        .get("tag_name")
        .and_then(|value| value.as_str())
        .ok_or_else(|| anyhow::anyhow!("Could not get latest release tag name"))
        .and_then(|tag| {
            helpers::request_agent()
                .get(&format!(
                    "https://api.github.com/repos/adamaq01/mikado/git/refs/tags/{tag}"
                ))
                .call()?
                .body_mut()
                .read_json::<serde_json::Value>()?
                .get("object")
                .and_then(|value| value.get("sha"))
                .and_then(|value| value.as_str())
                .map(|value| value.to_string())
                .ok_or_else(|| anyhow::anyhow!("Could not get latest release commit hash"))
        })?;

    if commit_hash != latest_commit_hash && !cfg!(debug_assertions) {
        info!(
            "A newer version of Mikado is available at https://github.com/adamaq01/mikado/releases/latest"
        );
    }

    Ok(())
}

#[crochet::hook("avs2-ea3.dll", "XEmdwapa000024")]
unsafe fn avs_ea3_boot_startup_hook(node: *const ()) -> i32 {
    if let Err(err) = hook_init(node) {
        error!("{err:#}");
    }

    call_original!(node)
}

#[unsafe(no_mangle)]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(
    dll_module: HINSTANCE,
    call_reason: u32,
    reserved: *mut core::ffi::c_void,
) -> BOOL {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            let _ = unsafe { AllocConsole() };
            Logger::new().init();
            panic_log::initialize_hook(panic_log::Configuration::default());

            print_infos();
            if let Err(err) = check_for_update() {
                error!("Unable to get update informations {err:#}");
            }

            if let Err(err) = crochet::enable!(avs_ea3_boot_startup_hook) {
                error!("{err:#}");
            }
        }
        DLL_PROCESS_DETACH => {
            if let Err(err) = crochet::disable!(avs_ea3_boot_startup_hook) {
                error!("{err:#}");
            }

            if let Err(err) = hook_release() {
                error!("{err:#}");
            }
        }
        _ => {}
    }

    TRUE
}
