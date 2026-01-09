/*
 * <message type>
 * at <filename>
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

pub fn throw_message(source_name: &str,message_type: MessageType, line: usize, pos:usize, message: &str) {
    eprintln!("{}: {}\nat {}", message_type, message, format!("{}:{}:{}", source_name,line,pos).bold())
}

#[macro_export]
macro_rules! panic_error {
    ($source: expr_2021, $line: expr_2021, $pos: expr_2021, $message: expr_2021) => {
        throw_message($source, MessageType::Error, $line, $pos, $message);
        exit(1)
    };
}