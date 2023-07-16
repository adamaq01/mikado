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
use ::log::error;
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
            error!("{:#}", err);
            std::process::exit(1);
        }

        result.unwrap()
    };
    pub static ref TACHI_STATUS_URL: String = {
        let result = Url::parse(&CONFIGURATION.tachi.base_url)
            .and_then(|url| url.join(&CONFIGURATION.tachi.status));
        if let Err(err) = result {
            error!("Could not parse Tachi status URL: {:#}", err);
            std::process::exit(1);
        }

        result.unwrap().to_string()
    };
    pub static ref TACHI_IMPORT_URL: String = {
        let result = Url::parse(&CONFIGURATION.tachi.base_url)
            .and_then(|url| url.join(&CONFIGURATION.tachi.import));
        if let Err(err) = result {
            error!("Could not parse Tachi import URL: {:#}", err);
            std::process::exit(1);
        }

        result.unwrap().to_string()
    };
    pub static ref TACHI_PBS_URL: String = {
        let result = Url::parse(&CONFIGURATION.tachi.base_url)
            .and_then(|url| url.join(&CONFIGURATION.tachi.pbs));
        if let Err(err) = result {
            error!("Could not parse Tachi import URL: {:#}", err);
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
        .filter_module("mikado", ::log::LevelFilter::Debug)
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

            writeln!(f, "[{}] {} {} -> {}", time, level, target, record.args())
        })
        .init();
}

#[crochet::hook("avs2-ea3.dll", "XEmdwapa000024")]
unsafe fn avs_ea3_boot_startup_hook(node: *const ()) -> i32 {
    if let Err(err) = hook_init(node) {
        error!("{:#}", err);
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

            if let Err(err) = crochet::enable!(avs_ea3_boot_startup_hook) {
                error!("{:#}", err);
            }
        }
        DLL_PROCESS_DETACH => {
            if let Err(err) = crochet::disable!(avs_ea3_boot_startup_hook) {
                error!("{:#}", err);
            }

            if let Err(err) = hook_release() {
                error!("{:#}", err);
            }
        }
        _ => {}
    }

    TRUE
}
