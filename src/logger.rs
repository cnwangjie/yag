use anyhow::Result;
use log::{set_boxed_logger, set_max_level, Level, LevelFilter, Log, Metadata, Record};

pub struct Logger {
    verbose: bool,
}

impl Logger {
    pub fn init(verbose: bool) -> Result<()> {
        let logger = Self { verbose };
        set_boxed_logger(Box::new(logger))?;
        set_max_level(LevelFilter::Debug);
        Ok(())
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.verbose || metadata.level() <= Level::Warn
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}
