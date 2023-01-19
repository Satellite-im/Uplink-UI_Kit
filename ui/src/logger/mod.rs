use derive_more::Display;
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::io::prelude::*;
use warp::sync::RwLock;

use chrono::Local;

use crate::STATIC_ARGS;

static LOGGER: Lazy<RwLock<Logger>> = Lazy::new(|| RwLock::new(Logger::load()));

pub fn set_save_to_file(b: bool) {
    LOGGER.write().save_to_file = b;
}

pub fn set_write_to_stdout(b: bool) {
    LOGGER.write().write_to_stdout = b;
}

pub fn set_max_logs(s: usize) {
    LOGGER.write().max_logs = s;
}

pub fn debug(message: &str) {
    LOGGER.write().log(LogLevel::Debug, message);
}

pub fn warn(message: &str) {
    LOGGER.write().log(LogLevel::Warn, message);
}

pub fn info(message: &str) {
    LOGGER.write().log(LogLevel::Info, message);
}

pub fn error(message: &str) {
    LOGGER.write().log(LogLevel::Error, message);
}

pub fn get_log_entries() -> Vec<Log> {
    Vec::from_iter(LOGGER.read().log_entries.iter().cloned())
}

#[derive(Debug, Clone)]
pub struct Log {
    pub level: LogLevel,
    pub message: String,
    pub datetime: String,
}

impl std::fmt::Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} | {} | {}", self.level, self.datetime, self.message)
    }
}

#[derive(Debug, Clone, Display)]
pub enum LogLevel {
    #[display(fmt = "DEBUG")]
    Debug,
    #[display(fmt = "WARN")]
    Info,
    #[display(fmt = "INFO")]
    Warn,
    #[display(fmt = "ERROR")]
    Error,
}

impl LogLevel {
    pub fn color(&self) -> &'static str {
        match self {
            LogLevel::Debug => "rgb(0, 255, 0)",
            LogLevel::Info => "rgb(0, 195, 255)",
            LogLevel::Warn => "yellow",
            LogLevel::Error => "red",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Logger {
    save_to_file: bool,
    write_to_stdout: bool,
    log_file: String,
    log_entries: VecDeque<Log>,
    max_logs: usize,
}

impl Logger {
    fn load() -> Logger {
        let logger_path = STATIC_ARGS.logger_path.to_string_lossy().to_string();
        let _ = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&logger_path);

        let log_entries = VecDeque::new();
        Logger {
            save_to_file: false,
            write_to_stdout: false,
            log_file: logger_path,
            log_entries,
            max_logs: 100, // todo: configurable?
        }
    }
}

impl Logger {
    fn log(&mut self, level: LogLevel, message: &str) {
        let new_log = Log {
            level,
            message: message.to_string(),
            datetime: Local::now().to_string(),
        };

        let log_to_log_entries = Log {
            level: new_log.level.clone(),
            message: new_log.message.clone(),
            datetime: new_log.datetime[0..19].to_string(),
        };

        self.log_entries.push_back(log_to_log_entries);
        if self.log_entries.len() >= self.max_logs {
            self.log_entries.pop_front();
        }

        if self.save_to_file {
            let mut file = OpenOptions::new()
                .append(true)
                .open(&self.log_file)
                .unwrap();

            if let Err(error) = writeln!(file, "{:?}", new_log) {
                self::error(format!("Couldn't write to debug.log file. {error}").as_str());
            }
        }

        if self.write_to_stdout {
            println!("{:?}", new_log)
        }
    }
}
