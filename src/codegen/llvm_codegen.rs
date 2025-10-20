use std::collections::HashMap;

use crate::{codegen::llvm::{Builder, FnValue, LlvmValue, Module, Type}, token::token_type::TokenType, Value::Value, AST::expr_node::{DataType, Expr, Func_Header}};

pub struct LLVMCodegen <'llvm>{
    exprs: Vec<Expr>,
    builder: Builder<'llvm>,
    module: &'llvm Module,
}

#[derive(Copy,Clone,Debug)]
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
            unimplemented!("{:?}", value)
        }
    }
}
impl<'llvm> From<TypeValue<'llvm>> for FnValue<'llvm> {
    fn from(value: TypeValue<'llvm>) -> Self {
        if let TypeValue::FnValue(l) = value {
            l
        } else {
            unimplemented!("{:?}", value)
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

    pub fn get_module(&'llvm self) -> &'llvm Module {
        self.module
    }

    pub fn codegen_all(&'llvm self) -> Vec<TypeValue<'llvm>> {
        let mut hm = HashMap::new();
        self.exprs.clone().into_iter().map(|f| {
            self.codegen(f,&mut hm)
        }).collect::<Vec<TypeValue<'llvm>>>()
    }

    fn dczdt_2_llvmdt(&self, dt: DataType, is_ptr: bool) -> Type<'llvm> {
        match dt {
            DataType::Void => {
                let mut t = self.module.type_void();
                if is_ptr {
                    t.to_pointer();
                }
                t
            }
            DataType::Int => {
                let mut t = self.module.type_i32();
                if is_ptr {
                    t.to_pointer();
                }
                t
            }
            DataType::Long => {
                self.module.type_i64()
            }
            DataType::Char => {

                let mut t = self.module.type_char();
                if is_ptr {
                    t.to_pointer();
                }
                t
            }
            _ => {
                todo!("{:?}",dt)
            }
        }
    }

    fn extern_codegen(&self,f: Func_Header) -> TypeValue<'llvm> {
        let mut args_dt = Vec::new();
        for x in f.clone().args {
            args_dt.push( self.dczdt_2_llvmdt(x.dt,x.is_ptr));
        }
        let fun = self.module.type_fn(&mut args_dt,
            {
                if let Some(r) = f.return_type {
                    self.dczdt_2_llvmdt(r,f.is_ptr_dt)
                }else {
                    self.dczdt_2_llvmdt(DataType::Void,false)
                }
            });
        let fn_v = self.module.add_fn(&f.name,fun);
        for idx in 0..fn_v.args() {
            fn_v.arg(idx).set_name(&f.args[idx].name);
        }
        TypeValue::FnValue(fn_v)
    }

    fn codegen(&'llvm self,
    e: Expr,
    variable: &mut HashMap<
        String,
        (LlvmValue<'llvm>,Type<'llvm>)
    >) -> TypeValue<'llvm> {
        match e {
            Expr::Statement(statement) => self.codegen(*statement, variable),
            Expr::Literal(v) => {
                match v {
                    Value::Number(n) => {

                        TypeValue::LLVMValue( self.module.type_i32().const_i32(n as i32))
                    }
                    Value::Str(s)=> {
                        TypeValue::LLVMValue(self.builder.global_string(&s))
                    }
                    _ => todo!()
                }

            },
            Expr::Binary(lhs, op, rhs) => {
                let lhs = self.codegen(*lhs,variable);
                let rhs = self.codegen(*rhs,variable);

                match op.tok_type {
                    TokenType::Plus => {
                        TypeValue::LLVMValue(
                            self.builder.iadd(lhs.into(), rhs.into())
                        )
                    },
                    TokenType::EqualEqual => {
                        TypeValue::LLVMValue(
                            self.builder.cmp(lhs.into(),llvm_sys_201::LLVMIntPredicate::LLVMIntEQ,rhs.into())
                        )
                    }
                    TokenType::Less => {
                        TypeValue::LLVMValue(
                            self.builder.isub(lhs.into(),rhs.into())
                        )
                    }
                    TokenType::Minus => {
                        TypeValue::LLVMValue(
                            self.builder.cmp(lhs.into(),llvm_sys_201::LLVMIntPredicate::LLVMIntSLT,rhs.into())
                        )
                    }
                    o => {
                        todo!("Unsupport Token: {:#?}", o)
                    }
                }
            }
            Expr::Extern(f) => {
                self.extern_codegen(f)
            }
            Expr::VarDecl(dt,is_ptr ,_is_const ,name ,init ) => {
                /*
                    %{name}_ptr = alloca <type>
                    store <type> <val>, ptr %{name}_ptr
                */


                let ptr_name = name.clone() + "_ptr";
                let alloca= self.builder.alloca(self.dczdt_2_llvmdt(dt.clone(), is_ptr), &ptr_name);
                if let Some(v) = init {
                    let vf: LlvmValue<'llvm> = self.codegen(*v,variable).into();
                    self.builder.store(vf, alloca);
                }
                variable.insert(name,
                    (
                        alloca,
                        self.dczdt_2_llvmdt(dt, is_ptr)
                    ));
                TypeValue::None
            }
            Expr::Var(n) => {
                let v = variable.get(&n).unwrap();
                let l=self.builder.load(n.as_str(),v.1, v.0);
                TypeValue::LLVMValue(l)
            }
            Expr::Callee(name, args) => {
                if let Some(func) = self.module.get_fn(&name.ident_to_string()) {
                    //func.dump();
                    if func.args() != args.len() {
                        panic!("Incorrect # arguments passed");
                    }
                    let mut args = args
                            .iter()
                            .map(|arg| self.codegen(arg.clone(),variable).into()).collect::<
                            Vec<LlvmValue<'_>>
                            >();

                    TypeValue::LLVMValue(self.builder.call(func, &mut args,"\0"))
                } else {
                    panic!("Error!");
                }
            }
            Expr::Block(v) => {
                for x in v.iter() {
                    self.codegen(x.clone(),variable);
                }
                TypeValue::None
            }
            Expr::Return(v) => {
                if let Some(e) = v {
                    TypeValue::LLVMValue(
                        self.builder.ret(self.codegen(*e,variable).into())
                    )
                } else {
                    todo!("Not support empty return");
                }
            }
            Expr::FuncStmt(header, block) => {
                let f = match self.module.get_fn(&header.name) {
                    Some(f) => f,
                    None => self.extern_codegen(header.clone()).into()
                };
                if f.basic_blocks() > 0 {
                    panic!("Function cannot be redefined.");
                }

                let bb = self.module.new_basic_block(f);
                self.builder.pos_at_end(bb);

                for i in 0..f.args() {
                    let args_var = &header.args[i];
                    let ptr_name = args_var.name.clone() + "_ptr";
                    let args_type = self.dczdt_2_llvmdt(args_var.dt.clone(), args_var.is_ptr);
                    let alloca= self.builder.alloca(args_type, &ptr_name);

                    self.builder.store(f.arg(i), alloca);
                    variable.insert(args_var.name.clone(),(
                        alloca,
                        args_type
                    ));
                }

                self.codegen(*block,variable);
                if header.return_type.is_none() {
                    self.builder.retvoid();
                }
                assert!(f.verify());
                TypeValue::FnValue(f)
            }
            Expr::IfStmt(cond,then_block ,else_block ) => {
                let c:LlvmValue<'llvm> = self.codegen(*cond, variable).into();

                let func = self.builder.get_insert_block().get_parent();

                let then_bb = self.module.new_basic_block(func);
                let else_bb = self.module.create_basic_block("else");
                let end_bb = self.module.create_basic_block("end");

                self.builder.cond_br(c, then_bb,{
                    if !matches!(*else_block,Expr::None) { else_bb }
                    else {end_bb}
                });

                self.builder.pos_at_end(then_bb);

                self.codegen(*then_block, variable);
                self.builder.br(end_bb);

                if !matches!(*else_block, Expr::None) {
                    func.append_basic_block(else_bb);
                    self.builder.pos_at_end(else_bb);
                    self.codegen(*else_block, variable);
                    self.builder.br(end_bb);
                }

                func.append_basic_block(end_bb);
                self.builder.pos_at_end(end_bb);



                TypeValue::None
            }
            o => {
                todo!("{:?} not implemented for llvm", o)
            }
        }
    }
}