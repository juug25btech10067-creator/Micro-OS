//! Structured kernel diagnostics and logging subsystem.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info  = 2,
    Warn  = 3,
    Error = 4,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info  => write!(f, "INFO "),
            LogLevel::Warn  => write!(f, "WARN "),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// A structured log entry capturing timestamp, subsystem, level, and message.
#[derive(Debug, Clone)]
pub struct LogRecord {
    pub tick: usize,
    pub level: LogLevel,
    pub subsystem: &'static str,
    pub message: String,
}

impl LogRecord {
    pub fn to_trace_line(&self) -> String {
        format!(
            "[{:06}] [{}] [{:5}] {}",
            self.tick, self.subsystem, self.level, self.message
        )
    }

    pub fn to_json(&self) -> String {
        format!(
            r#"{{"tick":{},"subsystem":"{}","level":"{}","message":"{}"}}"#,
            self.tick,
            self.subsystem,
            self.level,
            self.message.replace('"', "\\\"")
        )
    }
}

/// In-memory ring/trace buffer for kernel diagnostics.
pub struct KernelLogger {
    min_level: LogLevel,
    records: Mutex<Vec<LogRecord>>,
    current_tick: AtomicUsize,
    console_enabled: bool,
}

impl KernelLogger {
    pub const fn new(min_level: LogLevel, console_enabled: bool) -> Self {
        Self {
            min_level,
            records: Mutex::new(Vec::new()),
            current_tick: AtomicUsize::new(0),
            console_enabled,
        }
    }

    pub fn set_tick(&self, tick: usize) {
        self.current_tick.store(tick, Ordering::Relaxed);
    }

    pub fn get_tick(&self) -> usize {
        self.current_tick.load(Ordering::Relaxed)
    }

    pub fn log(&self, level: LogLevel, subsystem: &'static str, message: String) {
        if level < self.min_level {
            return;
        }

        let tick = self.get_tick();
        let record = LogRecord {
            tick,
            level,
            subsystem,
            message,
        };

        if self.console_enabled {
            let color = match level {
                LogLevel::Trace => "\x1b[90m", // Gray
                LogLevel::Debug => "\x1b[36m", // Cyan
                LogLevel::Info  => "\x1b[32m", // Green
                LogLevel::Warn  => "\x1b[33m", // Yellow
                LogLevel::Error => "\x1b[31m", // Red
            };
            let reset = "\x1b[0m";
            println!(
                "{}[{:06}] [{:<6}] [{:5}] {}{}",
                color, record.tick, record.subsystem, record.level, record.message, reset
            );
        }

        if let Ok(mut lock) = self.records.lock() {
            lock.push(record);
        }
    }

    pub fn dump_records(&self) -> Vec<LogRecord> {
        if let Ok(lock) = self.records.lock() {
            lock.clone()
        } else {
            Vec::new()
        }
    }

    pub fn clear(&self) {
        if let Ok(mut lock) = self.records.lock() {
            lock.clear();
        }
    }
}

// Global logger instance for kernel-wide convenience.
lazy_static_or_local!();

// Simple singleton-like accessor without external crates
static GLOBAL_LOGGER: KernelLogger = KernelLogger::new(LogLevel::Info, true);

pub fn set_tick(tick: usize) {
    GLOBAL_LOGGER.set_tick(tick);
}

pub fn get_tick() -> usize {
    GLOBAL_LOGGER.get_tick()
}

pub fn klog(level: LogLevel, subsystem: &'static str, message: String) {
    GLOBAL_LOGGER.log(level, subsystem, message);
}

pub fn dump_logs() -> Vec<LogRecord> {
    GLOBAL_LOGGER.dump_records()
}

pub fn clear_logs() {
    GLOBAL_LOGGER.clear();
}

#[macro_export]
macro_rules! kinfo {
    ($sub:expr, $($arg:tt)*) => {
        $crate::kernel::logger::klog($crate::kernel::logger::LogLevel::Info, $sub, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! kwarn {
    ($sub:expr, $($arg:tt)*) => {
        $crate::kernel::logger::klog($crate::kernel::logger::LogLevel::Warn, $sub, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! kerror {
    ($sub:expr, $($arg:tt)*) => {
        $crate::kernel::logger::klog($crate::kernel::logger::LogLevel::Error, $sub, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! kdebug {
    ($sub:expr, $($arg:tt)*) => {
        $crate::kernel::logger::klog($crate::kernel::logger::LogLevel::Debug, $sub, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! ktrace {
    ($sub:expr, $($arg:tt)*) => {
        $crate::kernel::logger::klog($crate::kernel::logger::LogLevel::Trace, $sub, format!($($arg)*))
    };
}

macro_rules! lazy_static_or_local {
    () => {};
}
