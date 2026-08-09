#![allow(dead_code)]
use inkwell::{
    builder::Builder, context::Context, module::Module, support::LLVMString, types::{BasicTypeEnum, FloatType, FunctionType, IntType, VoidType}, values::{BasicValue, PointerValue}
};

use crate::AST::expr_node::{DataType, Expr, VariableType,ExprType};

pub struct LLVMCodegen<'llvm> {
    exprs: Vec<Expr>,
    cont: &'llvm Context,
    builder: Builder<'llvm>,
    module: Module<'llvm>,
}

enum InkwellValue<'llvm> {
    IntVal(IntType<'llvm>),
    FloatVal(FloatType<'llvm>),
    VoidVal(VoidType<'llvm>),
    None,
}

impl<'llvm> InkwellValue<'llvm> {
    pub fn const_int(self) -> Option<IntType<'llvm>> {
        if let InkwellValue::IntVal(v) = self {
            return Some(v);
        }
        None
    }
    pub fn const_float(self) -> Option<FloatType<'llvm>> {
        if let InkwellValue::FloatVal(v) = self {
            return Some(v);
        }
        None
    }
    pub fn get_basic_type(self) -> Option<BasicTypeEnum<'llvm>> {
        match self {
            InkwellValue::IntVal(int_type) => Some(BasicTypeEnum::IntType(int_type)),
            InkwellValue::FloatVal(float_type) => Some(BasicTypeEnum::FloatType(float_type)),
            _ => None,
        }
    }
}

#[derive(Debug,Clone)]
pub struct VariableLLVM<'llvm> {
    name: String,
    pointer: PointerValue<'llvm>,
    ty: BasicTypeEnum<'llvm>
}

impl<'llvm> VariableLLVM<'llvm> {
    pub fn new(name: String, pointer: PointerValue<'llvm>, ty: BasicTypeEnum<'llvm>) -> Self {
        Self { name, pointer, ty }
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

impl<'llvm> LLVMCodegen<'llvm> {
    pub fn new(expr: Vec<Expr>, cont: &'llvm Context, name: &String) -> Self {
        Self {
            exprs: expr,
            cont: cont,
            builder: cont.create_builder(),
            module: cont.create_module(name.as_str())
        }
    }

    fn add_data_type(&self, dataType: DataType) -> InkwellValue<'llvm> {
        match dataType {
            DataType::I8 | DataType::U8 => InkwellValue::IntVal(self.cont.i8_type()),
            DataType::I16 | DataType::U16 => InkwellValue::IntVal(self.cont.i16_type()),
            DataType::I32 | DataType::U32 => InkwellValue::IntVal(self.cont.i32_type()),
            DataType::I64 | DataType::U64 => InkwellValue::IntVal(self.cont.i32_type()),
            DataType::F32 => InkwellValue::FloatVal(self.cont.f32_type()),
            DataType::F64 => InkwellValue::FloatVal(self.cont.f64_type()),
            DataType::Void => InkwellValue::VoidVal(self.cont.void_type()),
            _ => InkwellValue::None,
        }
    }

    fn variable_type_to_function_type(&self, vType: VariableType) -> FunctionType<'llvm> {
        match self.add_data_type(vType.get_datatype()) {
            InkwellValue::IntVal(v) => v.fn_type(&[], false),
            InkwellValue::FloatVal(float_type) => float_type.fn_type(&[], false),
            InkwellValue::VoidVal(void_type) => void_type.fn_type(&[], false),
            InkwellValue::None => todo!(),
        }
    }

    pub fn codegen(&self, e: &Expr, variable: &mut Vec<VariableLLVM<'llvm>>) -> Option<Box<dyn BasicValue<'llvm> + 'llvm>> {
        match e.clone().get_expr_type().and_then(|f| Some(*f)).unwrap_or(ExprType::None) {
            ExprType::Literal(v) => {
                let inkwell_value = self.add_data_type(v.val_type.clone());
                match v.val_type {
                    DataType::I8 | DataType::I16 | DataType::I32 | DataType::I64 => Some(Box::new(
                        inkwell_value
                            .const_int()?
                            .const_int(v.val.clone().to_literal().ok()? as u64, true),
                    )),
                    DataType::U8 | DataType::U16 | DataType::U32 | DataType::U64 => Some(Box::new(
                        inkwell_value
                            .const_int()?
                            .const_int(v.val.clone().to_literal().ok()? as u64, false),
                    )),
                    DataType::F32 | DataType::F64 => Some(
                        Box::new(
                            inkwell_value.const_float()?.const_float(v.val.clone().to_real().ok()?)
                        )
                    ),
                    _ => None
                }
            }
            ExprType::Return(v) => {
                let ret_val = &v;
                let k = match ret_val {
                    Some(v) => self.codegen(v,variable),
                    None => None,
                };
                self.builder
                    .build_return(k.as_deref())
                    .ok()?;
                None
            }
            ExprType::Block(blocks) => {
                None
            }
            ExprType::FuncStmt(fh, _body) => {
                let fn_type = match fh.return_type.clone() {
                    Some(vt) => self.variable_type_to_function_type(vt),
                    None => self
                        .variable_type_to_function_type(VariableType::NonPointer(DataType::Void)),
                };
                let f = self.module.add_function(&fh.name, fn_type, None);
                let basic_block = self.cont.append_basic_block(f, "entry");
                self.builder.position_at_end(basic_block);
                //self.codegen(_body,variable);
                None
            },
            ExprType::Var(k) => {
                match variable.iter().find(|&f| return f.get_name() == k) {
                    Some(v) => {
                        self.builder.build_load(
                            v.ty,
                            v.pointer,
                            &v.name
                        ).ok()?;
                    },
                    None => panic!("Variable not found!")
                }
                None
            },
            ExprType::VarDecl(_) => {
                // let ptr = self.builder.build_alloca(basic_type, name).ok()?;
                // let val = self.codegen(init.clone().unwrap().as_mut(),variable)?;
                // self.builder
                //     .build_store(ptr, val.as_basic_value_enum())
                //     .ok()?;
                // variable.push(VariableLLVM::new(name.clone(),ptr,basic_type));
                None
            }
            ExprType::Cast(_n, _e) => None,
            o => unimplemented!("Expr not implemented: {:?}", o),
        }
    }

    pub fn compile(&self) -> Result<(), LLVMString> {
        let mut v = Vec::new();
        for e in self.exprs.iter() {
            self.codegen(&e,v.as_mut());
        }
        self.module.verify()
    }
    pub fn dump(&self) {
        self.module.print_to_stderr();
    }
}
