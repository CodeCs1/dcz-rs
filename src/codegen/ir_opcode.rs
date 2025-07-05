#![allow(dead_code)]

use std::fmt::{Debug, Display};
use crate::{token::TokenData, Value::Value, AST::expr_node::DataType};

#[derive(Clone)]
pub enum Opcode {
    ///INVAILD
    Invaild,
    /// RET
    Return(Option<Value>),
    /// LOADCONSTANT(idx)
    LoadConstant(usize),
    ///CONSTANT
    Constant(Value),
    /// NOT
    Not,
    /// NEG
    Neg,
    /// STORE_GLOBAL
    StoreGlobal(DataType,bool, String),
    /// LOAD_NAME
    LoadName(String),
    /// STORE_LOCAL
    StoreLocal(DataType, bool,String),
    /// STORE_Param
    StoreParam(DataType, String),
    /// STORE_ARG
    StoreArg(Value),
    /// End
    End,
    /// BEGIN
    Begin,
    /// BINOP(Binop)
    BinOp(TokenData),
    /// JMP(offset)
    Jmp(usize),
    /// JBackward
    JBackward(usize),
    /// JIFFALSE(offset)
    JIfFalse(usize),
    ///AGN(name)
    Agn(String),
    /// MAKEFUNC(Size, Name)
    MakeFunc(usize,String),
    ///END_FUNC
    EndFunc,
    /// CALL(name)
    Call(String),
    /// PUSH(Value)
    Push(Value),
    /// POP
    Pop,
    /// NOP
    Nop,
}

impl Debug for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Opcode::Return(v) => write!(f,"[RET {:?}]", v.clone()),
            Opcode::LoadConstant(idx) => write!(f, "[LOADCONSTANT (idx: {})]", idx),
            Opcode::Constant(v) => {
                /*
                let mut value = String::new();
                if v.value().unwrap().is::<i64>() {
                    value = v.clone().to_literal().to_string();
                } else if v.value().unwrap().is::<String>() {
                    value = v.clone().to_string();
                } else if v.value().unwrap().is::<char>() {
                    value = v.clone().to_char().to_string();
                }
                else {
                    value = v.clone().to_float().to_string();
                }*/
                let any = v.clone().to_any();
                let value=
                    if any.is::<i64>() {
                        v.clone().to_literal().to_string()
                    } else if any.is::<String>() {
                        v.clone().to_string()
                    } else if any.is::<char>() {
                        v.clone().to_char().to_string()
                    } else {
                        v.clone().to_float().to_string()
                    };

                write!(f, "[CONSTANT (v: {})]", value)
            }
            Opcode::Not => write!(f, "[NOT (rhs)]"),
            Opcode::Neg => write!(f, "[NEG (rhs)]"),
            Opcode::Nop => write!(f,"[NOP]"),
            Opcode::LoadName(n) => write!(f, "[LOAD_NAME ({})]", n),
            Opcode::StoreGlobal(d,is_p,s) => write!(f, "[STORE_GLOBAL ({:?}(ptr: {}) {})]", d,is_p,s),
            Opcode::StoreLocal(d,is_p,s) => write!(f, "[STORE_LOCAL ({:?}(ptr: {}) {})]", d,is_p,s),
            Opcode::End => write!(f, "[END]"),
            Opcode::Begin => write!(f, "[BEGIN]"),
            Opcode::Jmp(offset) => write!(f, "[JMP ({})]", offset),
            Opcode::JIfFalse(offset) => write!(f, "[JIFFALSE ({})]", offset),
            Opcode::JBackward(offset) => write!(f, "[JBackward ({})]", offset),
            Opcode::BinOp(tt) => write!(f, "[BINOP (lhs {:?} rhs)]", tt.tok_type),
            Opcode::Agn(n) => write!(f, "[AGN ({})]", n),
            Opcode::MakeFunc(sz,name) => write!(f, "[MAKEFUNC {}({})]", name,sz),
            Opcode::Push(v) => write!(f, "[PUSH ({})]", v.clone().to_literal()),
            Opcode::Pop => write!(f, "[POP]"),
            Opcode::Call(n) => write!(f, "[CALL ({})]", n.clone()),
            Opcode::EndFunc => write!(f, "[END_FUNC]"),
            Opcode::StoreParam(d, n) => write!(f, "[STORE_PARAM ({:?} {})]", d,n.to_string()),
            Opcode::StoreArg(v) => write!(f, "[STORE_ARG ({:?})]", v),
            Opcode::Invaild => write!(f, "[INVAILD]")
        }
    }
}

impl Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Opcode::Return(v) => write!(f, "ret {:?}", v),
            Opcode::Nop => write!(f, "nop"),
            _ => todo!()
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConstantPool {
    v: Vec<Value>
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
    pub fn len(&self) -> usize {
        self.v.len()
    }
}
