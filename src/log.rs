use std::fmt;
use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct Logger {
    file: File,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            file: File::create("mikado.log").unwrap(),
        }
    }
}

impl Write for Logger {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Ignore the result of the write to stdout, since it's not really important
        let _ = std::io::stdout().write(buf);
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // Ignore the result of the write to stdout, since it's not really important
        let _ = std::io::stdout().flush();
        self.file.flush()
    }
}

pub(crate) struct Padded<T> {
    pub(crate) value: T,
    pub(crate) width: usize,
}

impl<T: fmt::Display> fmt::Display for Padded<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: <width$}", self.value, width = self.width)
    }
}

pub(crate) static MAX_MODULE_WIDTH: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn max_target_width(target: &str) -> usize {
    let max_width = MAX_MODULE_WIDTH.load(Ordering::Relaxed);
    if max_width < target.len() {
        MAX_MODULE_WIDTH.store(target.len(), Ordering::Relaxed);
        target.len()
    } else {
        max_width
    }
}

pub(crate) fn colored_level(
    style: &mut env_logger::fmt::Style,
    level: log::Level,
) -> env_logger::fmt::StyledValue<&'static str> {
    match level {
        log::Level::Trace => style
            .set_color(env_logger::fmt::Color::Magenta)
            .value("TRACE"),
        log::Level::Debug => style.set_color(env_logger::fmt::Color::Blue).value("DEBUG"),
        log::Level::Info => style.set_color(env_logger::fmt::Color::Green).value("INFO"),
        log::Level::Warn => style
            .set_color(env_logger::fmt::Color::Yellow)
            .value("WARN"),
        log::Level::Error => style.set_color(env_logger::fmt::Color::Red).value("ERROR"),
    }
}
