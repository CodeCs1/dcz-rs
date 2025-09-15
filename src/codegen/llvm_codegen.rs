use crate::{codegen::llvm::{Builder, FnValue, LlvmValue, Module, Type}, token::token_type::TokenType, Value::Value, AST::expr_node::{DataType, Expr, Func_Header}};

pub struct LLVMCodegen <'llvm>{
    exprs: Vec<Expr>,
    builder: Builder<'llvm>,
    module: &'llvm Module,
}

pub enum TypeValue<'llvm> {
    LLVMValue(LlvmValue<'llvm>),
    FnValue(FnValue<'llvm>),
    None
}

impl<'llvm> From<TypeValue<'llvm>> for LlvmValue<'llvm> {
    fn from(value: TypeValue<'llvm>) -> Self {
        if let TypeValue::LLVMValue(l) = value {
            l
        } else {
            unimplemented!()
        }
    }
}
impl<'llvm> From<TypeValue<'llvm>> for FnValue<'llvm> {
    fn from(value: TypeValue<'llvm>) -> Self {
        if let TypeValue::FnValue(l) = value {
            l
        } else {
            unimplemented!()
        }
    }
}


impl<'llvm> LLVMCodegen<'llvm> {
    pub fn compile(expr: Vec<Expr>, module: &'llvm Module) -> Self {
        Self {
            exprs: expr,
            builder: Builder::new(module),
            module: module
        }
    }

    pub fn codegen_all(&'llvm self) -> Vec<TypeValue<'llvm>> {
        self.exprs.clone().into_iter().map(|f| {
            self.codegen(f)
        }).collect::<Vec<TypeValue<'llvm>>>()
    }

    fn dczdt_2_llvmdt(&self, dt: DataType) -> Type<'llvm> {
        match dt {
            DataType::Void => {
                self.module.type_void()
            }
            DataType::Int => {
                self.module.type_i32()
            }
            _ => {
                todo!()
            }
        }
    }

    fn extern_codegen(&self,f: Func_Header) -> TypeValue {
        let mut args_dt = Vec::new();
        for x in f.clone().args {
            args_dt.push( self.dczdt_2_llvmdt(x.0));
        }
        let fun = self.module.type_fn(&mut args_dt, 
            {
                if let Some(r) = f.return_type {
                    self.dczdt_2_llvmdt(r)
                }else {
                    self.dczdt_2_llvmdt(DataType::Void)
                }
            });
        let fn_v = self.module.add_fn(&f.name,fun);
        for idx in 0..fn_v.args() {
            fn_v.arg(idx).set_name(&f.args[idx].1);
        }
        TypeValue::FnValue(fn_v)
    } 

    fn codegen(&'llvm self,e: Expr) -> TypeValue<'llvm> {
        match e {
            Expr::Literal(v) => {
                match v {
                    Value::Number(n) => {
                        TypeValue::LLVMValue( self.module.type_i32().const_i32(n as i32))
                    }
                    _ => todo!()
                }
                
            },
            Expr::Binary(lhs, op, rhs) => {
                let lhs = self.codegen(*lhs);
                let rhs = self.codegen(*rhs);

                match op.tok_type {
                    TokenType::Plus => {
                        TypeValue::LLVMValue(
                            self.builder.iadd(lhs.into(), rhs.into())
                        )
                    },
                    _ => {
                        todo!()
                    }
                }
            }
            Expr::Extern(f) => {
                self.extern_codegen(f)
            }
            Expr::Callee(name, args) => {
                if let Some(func) = self.module.get_fn(&name.ident_to_string()) {
                    //func.dump();
                    if func.args() != args.len() {
                        panic!("Incorrect # arguments passed");
                    }
                    let mut args = args
                            .iter()
                            .map(|arg| self.codegen(arg.clone()).into()).collect::<
                            Vec<LlvmValue<'_>>
                            >();

                    TypeValue::LLVMValue(self.builder.call(func, &mut args))
                } else {
                    panic!("Error!");
                }
            }
            Expr::Block(v) => {
                for x in v.iter() {
                    self.codegen(x.clone());
                }
                TypeValue::None
            }
            Expr::Return(v) => {
                if let Some(e) = v {
                    TypeValue::LLVMValue(
                        self.builder.ret(self.codegen(*e).into())
                    )
                } else {
                    todo!("Not support empty return");
                }
            }
            Expr::FuncStmt(header, block) => {
                let f = match self.module.get_fn(&header.name) {
                    Some(f) => f,
                    None => self.extern_codegen(header).into()
                };
                if f.basic_blocks() > 0 {
                    panic!("Function cannot be redefined.");
                }

                let bb = self.module.new_basic_block(f);
                self.builder.pos_at_end(bb);

                self.codegen(*block);
                assert!(f.verify());
                TypeValue::FnValue(f)
            }
            o => {
                todo!("{:?} not implemented for llvm", o)
            }
        }
    }
}