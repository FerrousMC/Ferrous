#![allow(dead_code)] //this will get called *eventually*
use std::{fmt::Display};
use colored::{Colorize};

/// A logger built upon the println!() macro.
pub fn log(level: LogLevel, threadname: ThreadName, message: &str) {
    println!("{thread}{level}: {msg}", level = level, thread = threadname, msg = message)
}

//this should be a macro, but my brain is too small
pub fn debug(threadname: ThreadName, message: &str) {
    log(LogLevel::Debug, threadname, message)
}

pub fn info(threadname: ThreadName, message: &str) {
    log(LogLevel::Info, threadname, message)
}

pub fn warn(threadname: ThreadName, message: &str) {
    log(LogLevel::Warn, threadname, message)
}

pub fn error(threadname: ThreadName, message: &str) {
    log(LogLevel::Error, threadname, message)
}

pub fn fatal(threadname: ThreadName, message: &str) {
    log(LogLevel::Fatal, threadname, message)
}

pub fn debug_main(message: &str) {
    log(LogLevel::Debug, ThreadName::Main, message)
}

pub fn info_main(message: &str) {
    log(LogLevel::Info, ThreadName::Main, message)
}

pub fn warn_main(message: &str) {
    log(LogLevel::Warn, ThreadName::Main, message)
}

pub fn error_main(message: &str) {
    log(LogLevel::Error, ThreadName::Main, message)
}

pub fn fatal_main(message: &str) {
    log(LogLevel::Fatal, ThreadName::Main, message)
}

pub fn debug_net(message: &str) {
    log(LogLevel::Debug, ThreadName::Network, message)
}

pub fn info_net(message: &str) {
    log(LogLevel::Info, ThreadName::Network, message)
}

pub fn warn_net(message: &str) {
    log(LogLevel::Warn, ThreadName::Network, message)
}

pub fn error_net(message: &str) {
    log(LogLevel::Error, ThreadName::Network, message)
}

pub fn fatal_net(message: &str) {
    log(LogLevel::Fatal, ThreadName::Network, message)
}

pub enum ThreadName {
    Main,
    Network
}

impl Display for ThreadName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            ThreadName::Main => "[Main]",
            ThreadName::Network => "[Network]",
        };

        write!(f, "{}", &string)
    }
}

pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Fatal
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            LogLevel::Debug => "[Debug]".blue(),
            LogLevel::Info => "[Info]".white(),
            LogLevel::Warn => "[Warning]".yellow(),
            LogLevel::Error => "[Error]".red(),
            LogLevel::Fatal => "[Fatal]".bright_red(),
        };

        write!(f, "{}", &string)
    }
}
