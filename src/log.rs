use log::{Level, LevelFilter, Log, Metadata, Record};
use std::{
    collections::VecDeque,
    fmt::{self, Display, Formatter},
    sync::{RwLock, RwLockReadGuard},
    time::{SystemTime, UNIX_EPOCH},
};

pub static LOGGER: Logger = Logger::new();

pub fn init() {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);
}

pub struct Logger {
    entries: RwLock<VecDeque<Entry>>,
}

impl Logger {
    pub fn read(&self) -> RwLockReadGuard<'_, VecDeque<Entry>> {
        self.entries.read().unwrap()
    }

    pub fn clear(&self) {
        self.entries.write().unwrap().clear();
    }

    const fn new() -> Self {
        Self {
            entries: RwLock::new(VecDeque::new()),
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

        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let entry = Entry {
            time,
            level: record.level(),
            message: trimmed_args.to_owned(),
        };

        match record.level() {
            Level::Error | Level::Warn => eprintln!("{}", entry),
            Level::Info | Level::Debug | Level::Trace => println!("{}", entry),
        }

        self.entries.write().unwrap().push_front(entry);
    }

    fn flush(&self) {}
}

pub struct Entry {
    pub level: Level,
    time: u64,
    message: String,
}

impl Display for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{} {}] {}", self.time, self.level, self.message)
    }
}

impl From<&Entry> for String {
    fn from(value: &Entry) -> Self {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Arguments;

    const GUIT_TARGET: &str = "guit::test";
    const OTHER_TARGET: &str = "other::test";

    #[test]
    fn logger_new_has_no_entries() {
        let logger = Logger::new();

        assert!(logger.read().is_empty());
    }

    #[test]
    fn logger_clear_has_no_entries() {
        let logger = Logger::new();
        logger.clear();

        assert!(logger.read().is_empty());
    }

    #[test]
    fn logger_clear_removes_entries() {
        let logger = Logger::new();
        let record = create_record(format_args!("info message"), Level::Info, GUIT_TARGET);

        logger.log(&record);
        assert_eq!(logger.read().len(), 1);

        logger.clear();
        assert!(logger.read().is_empty());
    }

    #[test]
    fn logger_enabled_true_for_error_warn_info_levels() {
        let logger = Logger::new();
        let error = create_metadata(Level::Error, GUIT_TARGET);
        let warn = create_metadata(Level::Warn, GUIT_TARGET);
        let info = create_metadata(Level::Info, GUIT_TARGET);

        assert!(logger.enabled(&error));
        assert!(logger.enabled(&warn));
        assert!(logger.enabled(&info));
    }

    #[test]
    fn logger_enabled_false_for_debug_trace_levels() {
        let logger = Logger::new();
        let debug = create_metadata(Level::Debug, GUIT_TARGET);
        let trace = create_metadata(Level::Trace, GUIT_TARGET);

        assert!(!logger.enabled(&debug));
        assert!(!logger.enabled(&trace));
    }

    #[test]
    fn logger_enabled_true_for_guit_modules() {
        let logger = Logger::new();
        let guit = create_metadata(Level::Info, GUIT_TARGET);

        assert!(logger.enabled(&guit));
    }

    #[test]
    fn logger_enabled_false_for_other_modules() {
        let logger = Logger::new();
        let other = create_metadata(Level::Info, OTHER_TARGET);

        assert!(!logger.enabled(&other));
    }

    #[test]
    fn logger_log_ignores_empty_message() {
        let logger = Logger::new();
        let record = create_record(format_args!("   \n\t  "), Level::Info, GUIT_TARGET);

        logger.log(&record);

        assert!(logger.read().is_empty());
    }

    #[test]
    fn logger_log_stores_valid_message() {
        let logger = Logger::new();
        let record = create_record(format_args!("info message"), Level::Info, GUIT_TARGET);

        logger.log(&record);
        let entries = logger.read();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, Level::Info);
        assert_eq!(entries[0].message, "info message");
    }

    #[test]
    fn logger_log_reverses_multiple_entries() {
        let logger = Logger::new();
        let first_record = create_record(format_args!("first message"), Level::Info, GUIT_TARGET);
        let second_record = create_record(format_args!("second message"), Level::Info, GUIT_TARGET);

        logger.log(&first_record);
        logger.log(&second_record);
        let entries = logger.read();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].message, "second message");
        assert_eq!(entries[1].message, "first message");
    }

    #[test]
    fn logger_log_ignores_other_modules() {
        let logger = Logger::new();
        let record = create_record(format_args!("info message"), Level::Info, OTHER_TARGET);

        logger.log(&record);

        assert!(logger.read().is_empty());
    }

    #[test]
    fn logger_log_stores_error_warn_info_levels() {
        let logger = Logger::new();
        let error = create_record(format_args!("error message"), Level::Error, GUIT_TARGET);
        let warn = create_record(format_args!("warn message"), Level::Warn, GUIT_TARGET);
        let info = create_record(format_args!("info message"), Level::Info, GUIT_TARGET);

        logger.log(&error);
        logger.log(&warn);
        logger.log(&info);
        let entries = logger.read();

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].level, Level::Info);
        assert_eq!(entries[1].level, Level::Warn);
        assert_eq!(entries[2].level, Level::Error);
    }

    #[test]
    fn logger_log_ignores_debug_trace_levels() {
        let logger = Logger::new();
        let debug = create_record(format_args!("debug message"), Level::Debug, GUIT_TARGET);
        let trace = create_record(format_args!("trace message"), Level::Trace, GUIT_TARGET);

        logger.log(&debug);
        logger.log(&trace);

        assert!(logger.read().is_empty());
    }

    #[test]
    fn entry_format() {
        let entry = Entry {
            time: 1234567890,
            level: Level::Error,
            message: "error message".to_string(),
        };
        let formatted = format!("{}", entry);

        assert_eq!(formatted, "[1234567890 ERROR] error message");
    }

    #[test]
    fn entry_to_string() {
        let entry = Entry {
            time: 999,
            level: Level::Info,
            message: "info message".to_string(),
        };
        let stringified = entry.to_string();

        assert_eq!(stringified, "[999 INFO] info message");
    }

    fn create_metadata<'a>(level: Level, target: &'a str) -> Metadata<'a> {
        Metadata::builder().level(level).target(target).build()
    }

    fn create_record<'a>(args: Arguments<'a>, level: Level, target: &'a str) -> Record<'a> {
        Record::builder()
            .args(args)
            .level(level)
            .target(target)
            .build()
    }
}
