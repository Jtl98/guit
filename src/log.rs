use log::{Level, LevelFilter, Log, Metadata, Record};
use std::{
    fmt::{self, Display, Formatter},
    sync::RwLock,
};

pub static LOGGER: Logger = Logger::new();

pub fn init() {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);
}

pub struct Logger {
    pub entries: RwLock<Vec<Entry>>,
}

impl Logger {
    const fn new() -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
        }
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info && metadata.target().starts_with("guit::")
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

        let entry = Entry {
            level: record.level(),
            message: trimmed_args.to_owned(),
        };

        match record.level() {
            Level::Error | Level::Warn => eprintln!("{}", entry),
            Level::Info | Level::Debug | Level::Trace => println!("{}", entry),
        }

        self.entries.write().unwrap().push(entry);
    }

    fn flush(&self) {}
}

pub struct Entry {
    pub level: Level,
    message: String,
}

impl Display for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.level, self.message)
    }
}

impl From<&Entry> for String {
    fn from(value: &Entry) -> Self {
        value.to_string()
    }
}
