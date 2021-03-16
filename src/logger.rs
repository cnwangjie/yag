use std::fmt::Display;

use anyhow::Result;
use log::{LevelFilter, Log, Metadata, Level, Record, max_level, set_boxed_logger, set_max_level};
use colored::*;

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
            println!("{} - {}", self.colored(record.level()), record.args());
        }
    }

    fn flush(&self) {}
}

impl Logger {
    #[inline]
    fn colored(&self, level: Level) -> ColoredString {
        match level {
            Level::Error => "ERROR".red(),
            Level::Warn => "WARN".yellow(),
            Level::Info => "INFO".blue(),
            Level::Debug => "DEBUG".bold(),
            Level::Trace => "TRACE".normal(),
        }
    }
}
