use log::{Level, LevelFilter, Log, Metadata, Record};

static LOGGER: Logger = Logger;

pub fn init() {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);
}

struct Logger;

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let args = format!("{}", record.args());
        let trimmed_args = args.trim();
        if trimmed_args.is_empty() {
            return;
        }

        match record.level() {
            Level::Error | Level::Warn => {
                eprintln!("{} {} {}", record.level(), record.target(), trimmed_args)
            }
            Level::Info | Level::Debug | Level::Trace => {
                println!("{} {} {}", record.level(), record.target(), trimmed_args)
            }
        }
    }

    fn flush(&self) {}
}
