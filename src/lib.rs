mod cloudlink;
mod configuration;
mod handlers;
mod helpers;
mod log;
mod mikado;
mod sys;
mod types;

use crate::log::Logger;
use crate::mikado::{hook_init, hook_release};
use ::log::{error, info};
use configuration::Configuration;
use lazy_static::lazy_static;
use url::Url;
use winapi::shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID, TRUE};
use winapi::um::consoleapi::AllocConsole;
use winapi::um::winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};

lazy_static! {
    pub static ref CONFIGURATION: Configuration = {
        let result = Configuration::load();
        if let Err(err) = result {
            error!("{err:#}");
            std::process::exit(1);
        }

        result.unwrap()
    };
    pub static ref TACHI_STATUS_URL: String = {
        let result = Url::parse(&CONFIGURATION.tachi.base_url)
            .and_then(|url| url.join(&CONFIGURATION.tachi.status));
        if let Err(err) = result {
            error!("Could not parse Tachi status URL: {err:#}");
            std::process::exit(1);
        }

        result.unwrap().to_string()
    };
    pub static ref TACHI_IMPORT_URL: String = {
        let result = Url::parse(&CONFIGURATION.tachi.base_url)
            .and_then(|url| url.join(&CONFIGURATION.tachi.import));
        if let Err(err) = result {
            error!("Could not parse Tachi import URL: {err:#}");
            std::process::exit(1);
        }

        result.unwrap().to_string()
    };
    pub static ref TACHI_PBS_URL: String = {
        let result = Url::parse(&CONFIGURATION.tachi.base_url)
            .and_then(|url| url.join(&CONFIGURATION.tachi.pbs));
        if let Err(err) = result {
            error!("Could not parse Tachi import URL: {err:#}");
            std::process::exit(1);
        }

        result
            .unwrap()
            .to_string()
            .replace("%7B", "{")
            .replace("%7D", "}")
    };
}

fn init_logger() {
    env_logger::builder()
        .filter_level(::log::LevelFilter::Error)
        .filter_module(
            "mikado",
            if cfg!(debug_assertions) {
                ::log::LevelFilter::Debug
            } else {
                ::log::LevelFilter::Info
            },
        )
        .parse_default_env()
        .target(env_logger::Target::Pipe(Box::new(Logger::new())))
        .format(|f, record| {
            use crate::log::{colored_level, max_target_width, Padded};
            use std::io::Write;

            let target = record.target();
            let max_width = max_target_width(target);

            let mut style = f.style();
            let level = colored_level(&mut style, record.level());

            let mut style = f.style();
            let target = style.set_bold(true).value(Padded {
                value: target,
                width: max_width,
            });

            let time = chrono::Local::now().format("%d/%m/%Y %H:%M:%S");

            writeln!(f, "[{time}] {level} {target} -> {}", record.args())
        })
        .init();
}

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
        .into_json::<serde_json::Value>()?
        .get("tag_name")
        .and_then(|value| value.as_str())
        .ok_or(anyhow::anyhow!("Could not get latest release tag name"))
        .and_then(|tag| {
            helpers::request_agent()
                .get(&format!(
                    "https://api.github.com/repos/adamaq01/mikado/git/refs/tags/{tag}"
                ))
                .call()?
                .into_json::<serde_json::Value>()?
                .get("object")
                .and_then(|value| value.get("sha"))
                .and_then(|value| value.as_str())
                .map(|value| value.to_string())
                .ok_or(anyhow::anyhow!("Could not get latest release commit hash"))
        })?;

    if commit_hash != latest_commit_hash && !cfg!(debug_assertions) {
        info!("A newer version of Mikado is available at https://github.com/adamaq01/mikado/releases/latest");
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

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: DWORD, reserved: LPVOID) -> BOOL {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            unsafe { AllocConsole() };
            init_logger();

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
