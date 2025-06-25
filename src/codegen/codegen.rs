use crate::AST::expr_node::Expr;
use iced_x86::{code_asm::*, Encoder, Instruction};

pub struct Codegen {
    ast: Vec<Expr>,
    pub blob_location: Vec<usize>
}
fn get_current_instr_opcode(instr: Instruction) -> Vec<u8> {
    let mut en = Encoder::new(64);
    en.encode(&instr, 0).expect("can't encode current instr");
    en.take_buffer()
}
impl Codegen {
    pub fn new(ast: Vec<Expr>) -> Self {
        Self { ast: ast, blob_location: Vec::new() }
    }

    pub fn instr(&mut self) -> CodeAssembler {
        let mut instr = CodeAssembler::new(64).expect("can't create code assembler class");

        let mut location: usize = 0;

        self.ast.iter_mut().for_each(|f| {
            match f {
                Expr::Macro(n, v) => {
                    //rdi	rsi	rdx	r10	r8	r9
                    match n.as_str() {
                        "syscall" => {
                            instr.mov(rax, v[0].to_value().to_literal()).expect("syscall ID MUST be integer");
                            let reg_args = [rdi, rsi, rdx, r10, r8, r9];
                            for i in 1..v.len() {
                                let val = v[i].visit();
                                match val {
                                    Expr::Literal(vl) => {
                                        if vl.value().expect("Null type").is::<i64>() { 
                                            instr.mov(reg_args[i-1], *vl.value().expect("Null type").downcast_ref::<i64>().unwrap()).expect("mov");
                                        } else if vl.value().expect("Null type").is::<String>() {
                                            instr.mov(reg_args[i-1], 0 as u64).expect("mov");
                                            location += get_current_instr_opcode(instr.instructions()[instr.instructions().len()-1]).len(); 
                                            self.blob_location.push(location+2);
                                        } else {
                                            todo!()
                                        }
                                    },
                                    _ => {}
                                }
                                location += get_current_instr_opcode(instr.instructions()[instr.instructions().len()-1]).len(); //TODO: use current instruction without use last element from instructions list
                            }
                            instr.syscall().expect("syscall");
                            location += 2;
                        },
                        _ => todo!()
                    }
                }
                _ => todo!()
            }
        });


        instr
    }
}
