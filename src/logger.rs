use anyhow::Result;
use log::{LevelFilter, Log, Metadata, Record, max_level, set_boxed_logger, set_max_level};

pub struct Logger {}

impl Logger {
    pub fn init(verbose: bool) -> Result<()> {
        let logger = Self {};
        set_boxed_logger(Box::new(logger))?;
        set_max_level({
            if verbose {
                LevelFilter::Debug
            } else {
                LevelFilter::Warn
            }
        });
        Ok(())
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}
