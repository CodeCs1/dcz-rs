/*dcz IR*/

use std::{fmt::Debug, sync::Arc};

pub trait Opcode {
    fn to_string(&self) -> String;
}

impl Debug for dyn Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

pub struct Args {
    pub idx: u128,
    pub is_write: bool,
    pub value: Arc<dyn std::any::Any>
}
impl Opcode for Args {
    fn to_string(&self) -> String {
        match self.is_write {
            true => {
                format!("arg{} {}", self.idx, *self.value.downcast_ref::<i32>().expect("Null"))
            },
            false => {
                format!("arg{}", self.idx)
            }
        }
    }
}

#[derive(Debug)]
pub struct BasicBlock {
    pub name: String,
    pub body: Ir
}

impl Opcode for BasicBlock {
    fn to_string(&self) -> String {
        let mut ir_code = String::new();
        ir_code.push_str( format!("{}:\n", self.name).as_str() );

        self.body.instr.iter().for_each(|opcode_instr| {
            ir_code.push('\t');
            ir_code.push_str(opcode_instr.to_string().as_str());
            ir_code.push('\n');
        });
        
        ir_code
    }
}

#[derive(Debug)]
pub struct Ir {
    pub instr: Vec<Arc<dyn Opcode>>
}

pub struct IrBuilder {
    instr: Vec<Arc<dyn Opcode>>,
}


impl IrBuilder {
    pub fn new() -> Self {
        Self { instr: Vec::new() }
    }
    pub fn args(mut self, vargs: Args) -> Self {
        self.instr.push(
            Arc::new(
                vargs
                )
            );
        self
    }

    pub fn block(mut self, block: BasicBlock) -> Self {
        self.instr.push(Arc::new(block));
        self
    }

    pub fn build(self) -> Ir {
        Ir { instr: self.instr }
    }
}
