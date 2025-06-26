use std::{collections::HashMap, fmt::Debug, hash::Hash};

use crate::{codegen::{ir::Ir, ir_opcode::*}, Value::Value};

pub struct VM {
    c_pool: ConstantPool,
    opcode: Ir,
    stack: Vec<Value>,
    ip: usize,
    variable_stack: HashMap<String, Value>,
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
    pub fn new(c_pool: ConstantPool, opcode: Ir) -> Self {
        Self { c_pool: c_pool, ip: 0, opcode: opcode, stack: Vec::new(), variable_stack: HashMap::new(), local_stack: Vec::new() }
    }

    pub fn run(&mut self) -> Result<(), VMError> {
        dbg!(&self.opcode.instr);
        let mut result = Result::Ok(());

        let opcode_vec = &self.opcode.instr;

        while self.ip < opcode_vec.len() {
            let op = &opcode_vec[self.ip];
            match op.clone() {
                Opcode::Nop => {},
                Opcode::LoadConstant(idx) => {
                    // store to stack
                    self.stack.push(self.c_pool.get(idx).expect("none_value").clone());
                }
                Opcode::Add => {
                    let tmp1 = self.stack.pop().unwrap();
                    let tmp2 = self.stack.pop().unwrap();
                    self.stack.push(tmp1+tmp2);
                }
                Opcode::StoreLocal(s) => {
                    let tmp1 = self.stack.pop().unwrap();
                    self.variable_stack.entry(s.clone()).or_insert_with(|| tmp1);
                    let len = self.local_stack.len();
                    self.local_stack[len-1].push(s.clone());
                }
                Opcode::ClearLocal => {

                    self.local_stack.last().unwrap().iter().for_each(|f| {
                        self.variable_stack.remove(f);
                    });

                    for _i in 0..self.local_stack.len() {
                        self.stack.pop();
                    }

                    self.local_stack.pop();
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
                Opcode::Sub => {
                    let tmp1 = self.stack.pop().unwrap();
                    let tmp2 = self.stack.pop().unwrap();
                    self.stack.push(tmp1-tmp2);
                }

                Opcode::Mul => {
                    let tmp1 = self.stack.pop().unwrap();
                    let tmp2 = self.stack.pop().unwrap();
                    self.stack.push(tmp1*tmp2);
                }

                Opcode::Div => {
                    let tmp1 = self.stack.pop().unwrap();
                    let tmp2 = self.stack.pop().unwrap();
                    self.stack.push(tmp1/tmp2);
                }
                Opcode::CmpLT => {
                    let tmp1 = self.stack.pop().unwrap();
                    let tmp2 = self.stack.pop().unwrap();
                    self.stack.push(Value::Boolean(tmp2 < tmp1));
                }
                Opcode::JIfFalse(offset) => {
                    let curr = self.stack.pop().unwrap();
                    if curr == Value::Boolean(false) {
                        self.ip += offset;
                    }
                }
                Opcode::Jmp(offset) => {
                    self.ip += offset;
                }
                Opcode::Neg => {
                    let tmp1 = self.stack.pop().unwrap();
                    self.stack.push(-tmp1);
                },
                Opcode::Not => {
                    let tmp1 = self.stack.pop().unwrap();
                    self.stack.push(!tmp1);
                }

                Opcode::Constant(v) => {
                    self.stack.push(v.clone());
                }
                _ => result=Err(VMError::RuntimeError),
            }
            self.ip+=1;
        }
        result
    }
}
