use core::fmt;
use log::{Level, LevelFilter, Metadata, Record};

struct SimpleLogger {
    level: LevelFilter,
}

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        // todo timestamp
        print_in_color(
            format_args!(
                "[{:<5}] {}\n",
                record.level(),
                record.args()
            ),
            record.level(),
        );
    }

    fn flush(&self) {}
}

// 不需要Mutex,因为底层的_print是安全的
static LOGGER: SimpleLogger = SimpleLogger {
    level: LevelFilter::Trace,
};

pub fn init_logger() {
    log::set_max_level(LOGGER.level);
    log::set_logger(&LOGGER).expect("init logger error");
}

fn print_in_color(args: fmt::Arguments, level: Level) {
    use crate::drivers::gpu::vga_buffer::_print;
    match level {
        Level::Error => _print(format_args!("\x1b[31m{}\x1b[0m", args)), // Red
        Level::Warn => _print(format_args!("\x1b[93m{}\x1b[0m", args)),  // BrightYellow
        Level::Info => _print(format_args!("\x1b[32m{}\x1b[0m", args)),  // Blue
        Level::Debug => _print(format_args!("\x1b[36m{}\x1b[0m", args)), // Green
        Level::Trace => _print(format_args!("\x1b[90m{}\x1b[0m", args)), // BrightBlack
    };
}
