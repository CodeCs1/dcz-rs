/*
 * <source_name>:<line>:<pos>: <message type>: message
 *
 *
 * */

#![allow(dead_code)]

use std::fmt::Display;
use colored::Colorize;
pub enum MessageType {
    Warning,
    Info,
    Error
}

impl Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s=match self {
            MessageType::Info => "Info".bright_blue().bold(),
            MessageType::Error => "Error".bright_red().bold(),
            MessageType::Warning => "Warning".bright_yellow().bold(),
        };
        write!(f, "{}", s)
    }
}


pub fn throw_message(source_name: &str,message_type: MessageType, line: i64, pos: i64, message: &str) {
    eprintln!("{}: {}\nat {}", message_type, message, format!("{}:{}:{}", source_name,line,pos).bold())
}
