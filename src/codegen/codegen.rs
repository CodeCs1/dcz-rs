use iced_x86::{code_asm::*, Encoder, Instruction};
use crate::{Value::Value, VM::stack::Stack};

use super::{ir::Ir, ir_opcode::Opcode};

pub struct Codegen {
    opcodes: Vec<Opcode>,
    pub blob_location: Vec<usize>,
    pub assign_location: Vec<(String, Value)>, // name, value
    pseudo_stack: Vec<Stack>,
    ip: usize,
}
impl Codegen {
    pub fn new(ir: Ir) -> Self {
        Self { opcodes:ir.instr , blob_location: Vec::new(), ip: 0, assign_location: Vec::new(), pseudo_stack: Vec::new() }
    }

    pub fn instr(&mut self) -> CodeAssembler {
        let mut instr = CodeAssembler::new(64).expect("can't create code assembler class");
        let mut location: usize = 0;

        while self.ip < self.opcodes.len() {
            let op = &self.opcodes[self.ip];
            match op {
                Opcode::Nop => instr.nop().expect("NOP"),
                Opcode::Constant(v) => self.pseudo_stack.push(Stack::Value(v.clone())),
                Opcode::StoreName(n) => { //store global
                    self.assign_location.push((n.clone(),self.pseudo_stack.last().unwrap().clone().as_value()));
                }
                _ => todo!("Opcode '{:?}' does not implement to be transformed to instruction yet.", op)
            }
            self.ip+=1;
        }
        

        instr
    }
}
