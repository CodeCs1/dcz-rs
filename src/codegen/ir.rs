/*dcz IR*/


use crate::Value::Value;
use super::ir_opcode::*;

#[derive(Debug)]
pub struct Ir {
    pub instr: Vec<Opcode>
}

impl std::fmt::Display for Ir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s =self.instr.iter().map(|opcode| format!("{}\n",opcode.to_string())).collect::<String>();
        s.pop();
        write!(f, "{}", s)
    }
}

#[derive(Clone)]
pub struct IrBuilder {
    instr: Vec<Opcode>,
    c_pool: ConstantPool,
}


impl IrBuilder {
    pub fn new() -> Self {
        Self { instr: Vec::new(), c_pool: ConstantPool::new() }
    }
    pub fn args(mut self, idx: u128, is_write: bool, v: Value) -> Self {
        self.instr.push(
            Opcode::Args(idx, is_write, v)
            );
        self
    }

    pub fn ret(mut self) -> Self {
        self.instr.push(Opcode::Return);
        self
    }

    pub fn constant(mut self, v: Value) -> Self {
        self.c_pool.append(v);
        self
    }

    pub fn load_constant(mut self, idx: usize) -> Self {
        self.instr.push(Opcode::LoadConstant(idx));
        self
    }

    pub fn get_const_pool(&self) -> ConstantPool {
        self.c_pool.clone()
    }

    pub fn append_from(mut self, op: Opcode) -> Self {
        self.instr.push(op);
        self
    }

    pub fn append_from_vec(mut self, v: &mut Vec<Opcode>) -> Self {
        for x in v {
            match x {
                Opcode::Constant(v)=> {
                    if v.value().expect("null value").is::<String>() {
                        self.c_pool.append(v.clone());
                    } else {
                        self.instr.push(x.clone());
                    }
                }
                _ => {
                    self.instr.push(x.clone());
                }
            }
        }
        self
    }


    pub fn build(self) -> Ir {
        Ir { instr: self.instr }
    }
}

impl FromIterator<Opcode> for IrBuilder {
    fn from_iter<T: IntoIterator<Item = Opcode>>(iter: T) -> Self {
        let mut irb = Self::new();

        for x in iter {
            irb=irb.append_from(x);
        }
        irb
    }
}
