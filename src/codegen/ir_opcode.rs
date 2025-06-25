#![allow(dead_code)]

use std::{fmt::{Debug, Display}};
use crate::Value::Value;

#[derive(Clone)]
pub enum Opcode {
    /// ARG(num, RW?, Value)
    Args(u128, bool, Value),
    /// RET opcode
    Return,
    /// LOADCONSTANT(idx)
    LoadConstant(usize),
    ///CONSTANT
    Constant(Value),
    /// ADD(lhs, rhs)
    Add,
    /// SUB(lhs, rhs)
    Sub,
    /// MUL(lhs, rhs)
    Mul,
    /// DIV(lhs, rhs)
    Div,
    /// SHL(lhs, rhs)
    Shl,
    /// SHR(lhs, rhs)
    Shr,
    /// NOT
    Not,
    /// NEG
    Neg,
    /// STORE_NAME
    StoreName(String),
    /// LOAD_NAME
    LoadName(String),
    /// STORE_LOCAL
    StoreLocal(String),
    /// CLEAR_LOCAL
    ClearLocal,
    /// BEGIN
    Begin,
    /// NOP
    Nop
}

impl Debug for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Opcode::Args(n, r, _) => {
                write!(f,"[ARG (idx: {}, is_write: {}, value: not_implemented)]", n,r)
            }
            Opcode::Return => {
                write!(f,"[RET]")
            },
            Opcode::LoadConstant(idx) => write!(f, "[LOADCONSTANT (idx: {})]", idx),
            Opcode::Constant(v) => {
                let mut value = String::new();
                if v.value().unwrap().is::<i64>() {
                    value = v.clone().to_literal().to_string();
                } else if v.value().unwrap().is::<String>() {
                    value = v.clone().to_string();
                } else {
                    value = v.clone().to_float().to_string();
                }
                write!(f, "[VALUE (v: {})]", value)
            }
            Opcode::Add => write!(f, "[ADD (lhs + rhs)]"),
            Opcode::Sub => write!(f, "[SUB (lhs + rhs)]"),
            Opcode::Mul => write!(f, "[MUL (lhs * rhs)]"),
            Opcode::Div => write!(f, "[DIV (lhs / rhs)]"),
            Opcode::Shl => write!(f, "[SHL (lhs << rhs)]"),
            Opcode::Shr => write!(f, "[SHR (lhs >> rhs)]"),
            Opcode::Not => write!(f, "[NOT (rhs)]"),
            Opcode::Neg => write!(f, "[NEG (rhs)]"),
            Opcode::Nop => write!(f,"[NOP]"),
            Opcode::LoadName(n) => write!(f, "[LOAD_NAME ({})]", n),
            Opcode::StoreName(s) => write!(f, "[STORE_NAME ({})]", s),
            Opcode::StoreLocal(s) => write!(f, "[STORE_LOCAL ({})]", s),
            Opcode::ClearLocal => write!(f, "[CLEAR_LOCAL]"),
            Opcode::Begin => write!(f, "[BEGIN]")
        }
    }
}

impl Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Opcode::Args(n, r, v) => {
                match r {
                    true =>write!(f,"arg{} {}", n, v.clone().to_literal()),
                    false =>write!(f, "arg{}", n),
                }
            },
            Opcode::Return => write!(f, "ret"),
            Opcode::Nop => write!(f, "nop"),
            _ => todo!()
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConstantPool {
    pub v: Vec<Value>
}

impl ConstantPool {
    pub fn new() -> Self {
        Self { v: Vec::new() }
    }
    pub fn append(&mut self, v: Value) {
        self.v.push(v);
    }
    pub fn get(&self, idx: usize) -> Option<&Value> {
        self.v.iter().nth(idx)
    }
}
