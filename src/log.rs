use anstyle::{AnsiColor, Style};
pub use log::*;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct Logger {
    // Routed through `AutoStream` so that, on older Windows consoles without native ANSI/VT
    // support, our ANSI codes get translated into Windows Console API calls instead of being
    // written out as raw (and unsupported) escape bytes.
    console: anstream::AutoStream<std::io::Stdout>,
    // Wrapped so that any ANSI codes we write for the (possibly colored) console output get
    // stripped back out before landing in the log file, regardless of `style_enabled()`.
    file: anstream::StripStream<File>,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            console: anstream::AutoStream::new(std::io::stdout(), color_choice()),
            file: anstream::StripStream::new(File::create("mikado.log").unwrap()),
        }
    }

    pub fn init(self) {
        env_logger::builder()
            .filter_level(LevelFilter::Error)
            .filter_module(
                "mikado",
                if cfg!(debug_assertions) {
                    LevelFilter::Trace
                } else {
                    LevelFilter::Info
                },
            )
            .parse_default_env()
            .target(env_logger::Target::Pipe(Box::new(self)))
            .format(|f, record| {
                let target = record.target();
                let max_width = max_target_width(target);

                let level = colored_level(record.level());

                let target_style = if style_enabled() {
                    Style::new().bold()
                } else {
                    Style::new()
                };
                let target = Padded {
                    value: target,
                    width: max_width,
                }
                .styled(target_style);

                let time = chrono::Local::now().format("%d/%m/%Y %H:%M:%S");

                writeln!(f, "[{time}] {level} {target} -> {}", record.args())
            })
            .init();
    }
}

impl Write for Logger {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Ignore the result of the write to stdout, since it's not really important
        let _ = self.console.write(buf);
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // Ignore the result of the write to stdout, since it's not really important
        let _ = self.console.flush();
        self.file.flush()
    }
}

struct Padded<T> {
    value: T,
    width: usize,
}

impl<T: fmt::Display> fmt::Display for Padded<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: <width$}", self.value, width = self.width)
    }
}

static MAX_MODULE_WIDTH: AtomicUsize = AtomicUsize::new(0);

fn max_target_width(target: &str) -> usize {
    let max_width = MAX_MODULE_WIDTH.load(Ordering::Relaxed);
    if max_width < target.len() {
        MAX_MODULE_WIDTH.store(target.len(), Ordering::Relaxed);
        target.len()
    } else {
        max_width
    }
}

struct Styled<T> {
    style: Style,
    item: T,
}

impl<T: fmt::Display> fmt::Display for Styled<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}{:#}", self.style, self.item, self.style)
    }
}

trait ToStyled<T> {
    fn styled(self, style: Style) -> Styled<T>;
}

impl<T> ToStyled<T> for T {
    fn styled(self, style: Style) -> Styled<T> {
        Styled { style, item: self }
    }
}

// Our writer always goes through `Target::Pipe`, which env_logger treats as a stream it
// can't inspect, so its own `Formatter::default_level_style` is always disabled for us
// (confirmed in its source: `Target::Pipe` never runs the stdout/stderr terminal check, and
// falls back to `WriteStyle::Never`). To still color the console when it actually supports
// it, we resolve color support ourselves the same way env_logger would for `Target::Stdout`:
// honor `RUST_LOG_STYLE` if set, otherwise auto-detect via `anstream`'s terminal/env checks.
fn color_choice() -> anstream::ColorChoice {
    static CHOICE: OnceLock<anstream::ColorChoice> = OnceLock::new();
    *CHOICE.get_or_init(|| match std::env::var("RUST_LOG_STYLE").as_deref() {
        Ok("always") => anstream::ColorChoice::Always,
        Ok("never") => anstream::ColorChoice::Never,
        _ => anstream::AutoStream::choice(&std::io::stdout()),
    })
}

fn style_enabled() -> bool {
    color_choice() != anstream::ColorChoice::Never
}

fn colored_level(level: Level) -> Styled<&'static str> {
    let (text, color) = match level {
        Level::Trace => ("TRACE", AnsiColor::Magenta),
        Level::Debug => ("DEBUG", AnsiColor::Blue),
        Level::Info => (" INFO", AnsiColor::Green),
        Level::Warn => (" WARN", AnsiColor::Yellow),
        Level::Error => ("ERROR", AnsiColor::Red),
    };

    let style = if style_enabled() {
        Style::new().fg_color(Some(color.into()))
    } else {
        Style::new()
    };

    text.styled(style)
}
