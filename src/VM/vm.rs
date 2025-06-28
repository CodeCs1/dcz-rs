use std::{collections::HashMap, fmt::Debug, hash::Hash};

use crate::{codegen::ir_opcode::*, Value::Value};

use super::stack::Stack;

pub struct VM {
    c_pool: ConstantPool,
    stack: Vec<Stack>,
    variable_stack: HashMap<String, Stack>,
    local_stack: Vec<Vec<String>>
}

pub enum VMError {
    CompileError,
    RuntimeError
}

impl Debug for VMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VMError::CompileError => write!(f, "Throw compile error!"),
            VMError::RuntimeError => write!(f, "Throw runtime error!")
        }
    }
}
impl VM {
    pub fn new(c_pool: ConstantPool) -> Self {
        Self { c_pool: c_pool, stack: Vec::new(), variable_stack: HashMap::new(), local_stack: Vec::new() }
    }

    pub fn run(&mut self, opcodes: Vec<Opcode>, mut ip: usize) -> Result<(), VMError> {
        dbg!(&opcodes);
        let mut result = Result::Ok(());

        let opcode_vec = &opcodes;

        while ip < opcode_vec.len() {
            let op = &opcode_vec[ip];
            match op.clone() {
                Opcode::Nop => {},
                Opcode::LoadConstant(idx) => {
                    // store to stack
                    self.stack.push(Stack::Value(self.c_pool.get(idx).expect("none_value").clone()));
                }
                Opcode::StoreLocal(s) => {
                    let tmp1 = self.stack.pop().unwrap();
                    self.variable_stack.entry(s.clone()).or_insert_with(|| tmp1);
                    let len = self.local_stack.len();
                    self.local_stack[len-1].push(s.clone());
                }
                Opcode::ClearLocal => {
                    if self.local_stack.last().is_some() {
                        self.local_stack.last().unwrap().iter().for_each(|f| {
                            self.variable_stack.remove(f);
                        });

                        for _i in 0..self.local_stack.len() {
                            self.stack.pop();
                        }

                        self.local_stack.pop();
                     }
                }
                Opcode::Begin => {
                    self.local_stack.push(Vec::new());
                }
                Opcode::StoreName(s) => {
                    let tmp1 = self.stack.pop().unwrap(); // get value
                    self.variable_stack.entry(s.clone()).or_insert_with(|| tmp1);
                }
                Opcode::LoadName(n) => {
                    self.stack.push(
                        self.variable_stack.get(&n).unwrap().clone()
                    );
                }
                Opcode::BinOp(op) => {
                    let tmp1 = self.stack.pop().unwrap().as_value();
                    let tmp2 = self.stack.pop().unwrap().as_value();
                    self.stack.push(Stack::Value(
                        match op.tok_type {
                            crate::token::token_type::TokenType::Plus => tmp2 + tmp1,
                            crate::token::token_type::TokenType::Minus => tmp2 - tmp1,
                            crate::token::token_type::TokenType::Star => tmp2 * tmp1,
                            crate::token::token_type::TokenType::Slash => tmp2 / tmp1,
                            crate::token::token_type::TokenType::Less => Value::Boolean(tmp2 < tmp1),
                            crate::token::token_type::TokenType::LessEqual => Value::Boolean(tmp2 <= tmp1),
                            crate::token::token_type::TokenType::Greater => Value::Boolean(tmp2 > tmp1),
                            crate::token::token_type::TokenType::GreaterEqual => Value::Boolean(tmp2 >= tmp1),
                            crate::token::token_type::TokenType::NotEqual => Value::Boolean(tmp2 != tmp1),
                            crate::token::token_type::TokenType::EqualEqual => Value::Boolean(tmp2 == tmp1),
                            crate::token::token_type::TokenType::ShiftLeft => tmp2 << tmp1,
                            crate::token::token_type::TokenType::ShiftRight => tmp2 >> tmp1,
                            _ => unimplemented!("I wont implement that operator!")
                        }));
                },
                Opcode::Push(v) => {self.stack.push(Stack::Value(v)); },
                Opcode::Pop => { self.stack.pop(); },
                Opcode::MakeFunc(sz) => {

                    let v = Vec::from( opcodes[ip+1..=ip+sz].to_vec() );
                    ip+=sz;
                    self.stack.push(Stack::CompressedFunc(v));
                },
                Opcode::Call => {
                    let func = self.stack.pop().unwrap();
                    self.run(func.as_compressed_func(),0)?;
                }
                Opcode::JBackward(offset) => {
                    ip -= offset-1;
                    continue;
                }
                Opcode::Agn(n) => {
                    let v = self.stack.pop().unwrap();
                    *self.variable_stack.get_mut(&n).unwrap() = v;
                }
                Opcode::JIfFalse(offset) => {
                    let mut curr = self.stack.pop().unwrap().as_value();
                    if !matches!(curr, Value::Boolean(_)) {
                        curr = Value::new_boolean_from(curr.to_literal());
                    }
                    if curr == Value::Boolean(false) {
                        ip += offset;
                    }
                }
                Opcode::Jmp(offset) => {
                    ip += offset;
                    continue;
                }
                Opcode::Neg => {
                    let tmp1 = self.stack.pop().unwrap().as_value();
                    self.stack.push(Stack::Value(-tmp1));
                },
                Opcode::Not => {
                    let tmp1 = self.stack.pop().unwrap().as_value();
                    self.stack.push(Stack::Value(!tmp1));
                }

                Opcode::Constant(v) => {
                    self.stack.push(Stack::Value(v.clone()));
                }
                _ => result=Err(VMError::RuntimeError),
            }
            ip+=1;        
        }
        self.stack.clear();
        self.variable_stack.clear();
        result
    }
}
