use std::sync::atomic::AtomicU32;

use iced_x86::{code_asm::*, Encoder, Instruction};
use crate::{Value::Value, AST::expr_node::DataType, VM::stack::Stack};

use super::{ir::Ir, ir_opcode::Opcode};

pub struct Codegen {
    pub blob_location: Vec<usize>,
    pub assign_location: Vec<(String, Value)>, // name, value
    pub func_location: Vec<(String, CodeAssembler)>, // name, opcodes
    pseudo_stack: Vec<Stack>,
}

fn padding(dt: DataType, offset: u32) -> u32 {
    if !matches!(dt, DataType::Char) {
        (dt.size()-(offset%dt.size()))%dt.size()
    } else {
        0
    }
}

impl Codegen {
    pub fn new() -> Self {
        Self { blob_location: Vec::new(), assign_location: Vec::new(), pseudo_stack: Vec::new(), func_location: Vec::new() }
    }

    pub fn instr(&mut self, opcodes: Vec<Opcode>) -> CodeAssembler {
        let mut instr = CodeAssembler::new(64).expect("can't create code assembler class");
       
        let mut local_pad_offset = 0;
        
        let mut ip =0;

        while ip < opcodes.len() {
            let op = &opcodes[ip];
            match op {
                Opcode::End => {}
                Opcode::Nop => instr.nop().expect("NOP"),
                Opcode::Constant(v) => self.pseudo_stack.push(Stack::Value(v.clone())),
                Opcode::StoreLocal(d,_n) => { //store local
                    let value = self.pseudo_stack.pop().unwrap();
                    let pad = padding(d.clone(), local_pad_offset);
                    let off = local_pad_offset+pad;
                    local_pad_offset = off+d.clone().size();

                    let p = match *d {
                        DataType::Int | DataType::Float => {
                            dword_ptr(rbp-local_pad_offset)
                        }
                        DataType::Char => {
                            byte_ptr(rbp-local_pad_offset)
                        }
                        DataType::Short => {
                            word_ptr(rbp-local_pad_offset)
                        }
                        DataType::Long | DataType::Suu => {
                            qword_ptr(rbp-local_pad_offset)
                            //qword_ptr(rbp)
                        }
                    };

                    let v = value.clone().as_value();

                    if v.clone().to_any().is::<i64>() {
                        instr.mov(p, (v.to_literal() & 0xffff) as i32).expect("MOV(STORELOCAL)");
                    } else if v.clone().to_any().is::<f32>() {
                        instr.movss(xmm0, dword_ptr(0)).expect("MOVSS(STORELOCAL) - FLOAT");
                        instr.movss(p, xmm0).expect("MOVSS(STORELOCAL) - FLOAT");
                    } else {
                        instr.mov(p, v.to_char() as i32).expect("MOV(STORELOCAL)");
                    }
                }
                Opcode::EndFunc => {
                }
                Opcode::StoreGlobal(d,n) => {
                    //let value = self.pseudo_stack.pop().unwrap();
                    //self.assign_location.push((n.clone(),value.clone().as_value()));
                    //instr.mov(dword_ptr(0), value.as_value().to_literal() as u32).expect("MOV(STORENAME)");
                }
                Opcode::Jmp(offset) => {
                    instr.jmp(*offset as u64).expect("JMP");
                },
                Opcode::MakeFunc(sz,name) => {
                    let v = Vec::from( opcodes[ip+1..=ip+sz].to_vec() );
                    let mut code_func=CodeAssembler::new(64).expect("MAKE_FUNC");
                    code_func.push(rbp).expect("MAKEFUNC(1)");
                    code_func.mov(rbp,rsp).expect("MAKEFUNC(2)");
                    let mut body_func = self.instr(v);
                    let body_vec = body_func.take_instructions();

                    for x in body_vec {
                        code_func.add_instruction(x).expect("MAKE_FUNC - ADD_INSTR");
                    }
                    code_func.pop(rbp).expect("END_FUNC(POP)");
                    code_func.ret().expect("END_FUNC(RET)");

                    self.func_location.push((name.clone(), code_func));
                },
                Opcode::Begin => {
                    instr.anonymous_label().expect("BEGIN");
                    instr.nop().expect("NOP");
                    //label.push(instr.bwd().expect("BEGIN(LABEL)"));
                }
                Opcode::JIfFalse(sz) => {
                    instr.jnz((ip+sz) as u64).expect("JIfFalse");
                }
                Opcode::JBackward(_sz) => {
                    let last_label = instr.bwd().expect("JBACKWARD(LABEL)");
                    instr.jmp(last_label).expect("JBackward");
                }
                _ => todo!("Opcode '{:?}' does not implement to be transformed to instruction yet.", op)
            }
            ip+=1;
        }
        

        instr
    }
}
