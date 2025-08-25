use std::collections::HashMap;

use iced_x86::code_asm::*;
use crate::{Value::Value, AST::expr_node::DataType, VM::stack::Stack};

use super::ir_opcode::Opcode;

pub struct Codegen{
    pub call_location: Vec<(String, usize)>,
    pub assign_location: Vec<(String, Value)>, // name, value
    pub func_location: Vec<(String, CodeAssembler)>, // name, opcodes
    pseudo_stack: Vec<Stack>,
    pseudo_variable_stack: HashMap<String, AsmMemoryOperand>,
    code_sz: usize,
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
        Self { call_location: Vec::new(), assign_location: Vec::new(), pseudo_stack: Vec::new(), func_location: Vec::new(), code_sz:0, pseudo_variable_stack: HashMap::new() }
    }

    pub fn instr(&mut self, opcodes: Vec<Opcode>, func_ip: usize) -> CodeAssembler {
        let mut instr = CodeAssembler::new(64).expect("can't create code assembler class");

        let (
            mut local_pad_offset, 
            mut param_offset, 
            mut ip, 
            mut arg_c,
            mut param_c
        ) = (0,0,0,0,0);

        while ip < opcodes.len() {
            let op = &opcodes[ip];
            match op {
                Opcode::End => {}
                Opcode::Nop => instr.nop().expect("NOP"),
                Opcode::Constant(v) => self.pseudo_stack.push(Stack::Value(v.clone())),
                Opcode::LoadName(n) => {
                    if self.func_location.iter().any(|(func_name,_)| func_name == n) {
                        self.pseudo_stack.push(Stack::Value(Value::Str(n.clone())));
                    } else {
                        todo!()
                    }
                }
                Opcode::StoreArg(v) => {
                    // calling convention
                    // Linux CC
                    // x32: bx, cx, dx, si, di, bp, (push)
                    // x64: di, si, dx, cx, r8, r9, (push)


                    if instr.bitness() == 64 {
                        let reg = [rdi, rsi, rdx, rcx, r8,r9];
                        let v = v.clone().to_literal();
                        let rg32 = if v < u32::max_value() as i64 {
                            match arg_c {
                                0 => Some(edi),
                                1 => Some(esi),
                                2 => Some(edx),
                                3 => Some(ecx),
                                4 => Some(r8d),
                                5 => Some(r9d),
                                _ => None
                            }
                        } else { None };
                        if arg_c <= reg.len() {
                            if let Some(r32) = rg32 {
                                instr.mov(r32, v as u32).expect("STORE_ARG");
                            } else {
                                instr.mov(reg[arg_c], v as u64).expect("STORE_ARG");
                            }
                        }
                    }
                    arg_c+=1

                },
                Opcode::Agn(n) => {
                    let (_,p) = self.pseudo_variable_stack.iter().find(|(f,_)| *f == n).unwrap();
                    let v = self.pseudo_stack.pop().expect("Empty variable").as_value();


                    instr.mov(p.clone(),v.to_literal() as i32).expect("AGN (MOV)");

                }
                Opcode::Call(n) => {
                    let instr_len = instr.assemble(0).expect("INSTR_LEN").len()+9;
                    instr.call(instr_len as u64).expect("CALL");
                    self.call_location.push((n.clone(),self.code_sz+func_ip+1));
                },
                Opcode::Invaild => {
                    instr.ud2().expect("INVAILD");
                }
                Opcode::Return(v) => {
                    if let Some(v) = v {
                        if matches!(v, Value::Number(_)) {
                            instr.mov(eax, v.clone().to_literal() as u32).expect("RETURN (MOV)");
                        } else if matches!(v, Value::Char(_)) {
                            instr.mov(eax, v.clone().to_char() as u32).expect("RETURN (MOV)");
                        } else { todo!() }
                    }
                }
                Opcode::StoreParam(d, _n) => {
                    let pad = padding(d.clone(), param_offset);
                    let off = param_offset+pad;
                    param_offset=off+d.clone().size();
 
                    let p = match *d {
                        DataType::Int | DataType::Float => {
                            dword_ptr(rbp-param_offset)
                        }
                        DataType::Char => {
                            byte_ptr(rbp-param_offset)
                        }
                        DataType::Short => {
                            word_ptr(rbp-param_offset)
                        }
                        DataType::Long | DataType::Suu => {
                            qword_ptr(rbp-param_offset)
                            //qword_ptr(rbp)
                        }
                        DataType::Unknown => todo!()
                    };

                    if instr.bitness() == 64 {
                        let reg = [rdi, rsi, rdx, rcx, r8,r9];
                        let rg32 = if matches!(d, DataType::Int) {
                            match param_c {
                                0 => Some(edi),
                                1 => Some(esi),
                                2 => Some(edx),
                                3 => Some(ecx),
                                4 => Some(r8d),
                                5 => Some(r9d),
                                _ => None
                            }
                        } else { None };
                        if param_c <= reg.len() {
                            if let Some(r32) = rg32 {
                                instr.mov(p, r32).expect("STORE_ARG");
                            } else {
                                instr.mov(p, reg[param_c]).expect("STORE_ARG");
                            }
                        }
                    }
                    param_c+=1
                }
                Opcode::Push(v) => {
                    instr.push(v.clone().to_literal() as u32).expect("PUSH");
                }
                Opcode::StoreLocal(d,is_p,n) => { //store local
                    let value = self.pseudo_stack.pop().unwrap();
                    let pad = padding(d.clone(), local_pad_offset);
                    let off = local_pad_offset+pad;
                    local_pad_offset = off+d.clone().size();


                    let d = if !*is_p { d.clone() } else { DataType::Long };

                    let p = match d {
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
                        DataType::Unknown => todo!()
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

                    self.pseudo_variable_stack.insert(n.clone(), p);
                }
                Opcode::EndFunc => {
                }
                Opcode::StoreGlobal(_d,_is_p,_n) => {
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
                    let mut body_func = self.instr(v,5);
                    let body_vec = body_func.instructions().to_vec();

                    for x in body_vec {
                        code_func.add_instruction(x).expect("MAKE_FUNC - ADD_INSTR");
                    }
                    code_func.pop(rbp).expect("END_FUNC(POP)");
                    code_func.ret().expect("END_FUNC(RET)");

                    self.func_location.push((name.clone(),code_func));
                    self.code_sz += body_func.assemble(0).expect("ok").len()+6;
                    ip += sz;
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
