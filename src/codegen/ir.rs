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

    pub fn get_const_pool(&self) -> ConstantPool {
        self.c_pool.clone()
    }


    pub fn append_from_vec(mut self, v: &mut Vec<Opcode>) -> Self {
        for x in v {
            match x {
                Opcode::Constant(v)=> {
                    let v_ = v.value().expect("null value");
                    if v_.is::<String>() {
                        self.c_pool.append(v.clone());
                        self.instr.push(Opcode::LoadConstant(self.c_pool.len()-1));
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
