#![allow(dead_code)]

use crate::{
    Value::{self, TypedValue},
    token::TokenData,
};

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Void,
    CustomType(String),
    Unknown
}

impl DataType {
    pub fn size(&self) -> u32 {
        match self {
            DataType::U8 | DataType::I8 => 1,
            DataType::U16 | DataType::I16 => 2,
            DataType::U32 | DataType::I32 | DataType::F32 => 4,
            DataType::U64 | DataType::I64 | DataType::F64 => 8,
            _ => 0,
        }
    }
}

impl From<String> for DataType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "i8"   => DataType::I8,
            "i16"  => DataType::I16,
            "i32"  => DataType::I32,
            "i64"  => DataType::I64,
            "u8"   => DataType::U8,
            "u16"  => DataType::U16,
            "u32"  => DataType::U32,
            "u64"  => DataType::U64,
            "f32"  => DataType::F32,
            "f64"  => DataType::F64,
            "void" => DataType::Void,
            ""     => DataType::Unknown,
            _  => DataType::CustomType(value)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Variable {
    pub vData: Option<VariableType>,
    pub name: String,
    pub init: Option<Expr>,
}

impl Variable {
    pub fn new(vData: Option<VariableType>, name: String, init: Option<Expr>) -> Self {
        // try to guess data type from init value

        Self {
            vData: vData,
            name,
            init,
        }
    }
    pub fn is_constant(self) -> bool {
        let Some(vdata) = self.vData else { return false };
        vdata.is_constant()
    }
}

#[derive(Clone, PartialEq)]
pub struct FuncHeader {
    pub name: String,
    pub args: Vec<Variable>,
    pub return_type: Option<VariableType>,
}

impl std::fmt::Debug for FuncHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}({:?}) -> {:?}", self.name, self.args, self.return_type)
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum AccessLevel {
    Private,
    Public,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassFunctionType {
    Function,
    Initializer,
    Deconstructor,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassFunction {
    pub access_level: AccessLevel,
    pub func_type: ClassFunctionType,
    pub function: Expr,
}

#[derive(Clone, PartialEq)]
pub enum VariableType {
    NonPointer(DataType),
    Pointer(DataType),
    Constant(DataType),
}

impl Default for VariableType {
    fn default() -> Self {
        Self::NonPointer(DataType::Unknown)
    }
}

impl std::fmt::Display for VariableType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constant(dt) => write!(f, "const {:#?}", dt),
            Self::NonPointer(dt) => write!(f, "{:#?}", dt),
            Self::Pointer(dt) => write!(f, "{:#?}*", dt),
        }
    }
}

impl std::fmt::Debug for VariableType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constant(dt) => write!(f, "const {:#?}", dt),
            Self::NonPointer(dt) => write!(f, "{:#?}", dt),
            Self::Pointer(dt) => write!(f, "{:#?}*", dt),
        }
    }
}

impl VariableType {
    pub fn new(datatype: DataType, is_ptr: bool, is_constant: bool) -> Self {
        if is_ptr {
            Self::Pointer(datatype)
        } else if is_constant {
            Self::Constant(datatype)
        } else {
            Self::NonPointer(datatype)
        }
    }
    pub fn is_pointer_of(self, datatype: DataType) -> bool {
        match self {
            Self::Pointer(p) => p == datatype,
            _ => false,
        }
    }
    pub fn is_pointer(self) -> bool {
        matches!(self, Self::Pointer(_))
    }
    pub fn get_datatype(self) -> DataType {
        match self {
            Self::Constant(dt) => dt,
            Self::NonPointer(dt) => dt,
            Self::Pointer(dt) => dt,
        }
    }
    pub fn is_constant(self) -> bool {
        matches!(self, Self::Constant(_))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprType {
    /// Binary Expression (Expr, Operator, Expr)
    Binary(Expr, TokenData, Expr),
    Literal(TypedValue),
    Unary(TokenData, Expr),
    Grouping(Expr),
    Macro(String, Expr),
    Identifier(String),
    Var(String),
    Statement(Expr),
    Block(Vec<Expr>),
    Assign(String, Expr),

    IfStmt(Expr, Expr, Option<Expr>),
    WhileStmt(Expr, Expr),
    /// FuncStmt(name, args, body, return_type)
    FuncStmt(FuncHeader, Expr),
    Callee(Expr, Vec<Expr>),

    /// Var declare Statement VarDecl(dt, is_pointer, is_constant, name, initializer)
    VarDecl(Variable),
    List(Vec<Value::Value>),
    Return(Option<Expr>),

    /// Extern declare statement
    Extern(FuncHeader),
    /// class statement
    Class(String, Vec<ClassFunction>),
    /// casting expr
    /// NewDataType, isPointer, Expression
    Cast(VariableType, Expr),

    None,
}

#[derive(Clone, PartialEq)]
pub struct Expr {
    expr_type: Box<ExprType>,
    pub line: usize,
    pub at: usize,
}

pub enum TerminalType {
    Literal,
    Identifier,
    Var
}

impl Expr {
    pub fn new(expr_type:ExprType, line: usize, at: usize) -> Self {
        Self { expr_type:Box::new(expr_type), line, at }
    }
    pub fn get_expr_type(self) -> Option<Box<ExprType>> {
        if matches!(*self.expr_type, ExprType::None) { None }
        else { Some(self.expr_type) }
    }
    pub fn new_group(inner_expr: Expr) -> Self {
        Self {
            expr_type: Box::new(ExprType::Grouping(inner_expr.clone())),
            line: inner_expr.line,
            at: inner_expr.at
        }
    }

    pub fn new_terminal(td:TokenData, term_type: TerminalType) -> Self {
        Self {
            expr_type: Box::new(
                match term_type {
                    TerminalType::Identifier => ExprType::Identifier(td.identifier),
                    TerminalType::Literal => ExprType::Literal(TypedValue::new(td.value,false)),
                    TerminalType::Var => ExprType::Var(td.identifier)
                }
            ),
            line: td.line,
            at: td.start
        }
    }
    pub fn new_callee(expr: Expr, args: Vec<Expr>) -> Self {
        Self {
            expr_type: Box::new(ExprType::Callee(expr.clone(), args)),
            line: expr.line,
            at: expr.at
        }
    }
    pub fn new_unary(op: TokenData, expr: Expr) -> Self {
        Self { expr_type: Box::new(
            ExprType::Unary(
                op.clone(),
                expr
            )
        ), line: op.line, at: op.start
        }
    }
    pub fn new_binary(lhs: Expr, op: TokenData, rhs: Expr) -> Self {
        let line = lhs.line;
        let at=lhs.at;
        Self {
            expr_type: Box::new(
                ExprType::Binary(lhs, op, rhs)
            ),
            line,
            at
        }
    }
    pub fn function(self) -> Option<(FuncHeader, Expr)> {
        let ExprType::FuncStmt(func_header, body) = *self.expr_type else { return None };
        Some((func_header, body))

    }
    pub fn identifier(self) -> Option<String> {
        let ExprType::Identifier(s) = *self.expr_type else { return None };
        Some(s)
    }

    pub fn literal(self) -> Option<TypedValue> {
        let ExprType::Literal(s) = *self.expr_type else {return None};
        Some(s)
    }

    pub fn var(self) -> Option<String> {
        let ExprType::Var(v) = *self.expr_type else {return None };
        Some(v)
    }
    pub fn if_vaild_then<F>(self, mut f: F)
        where F: FnMut(Expr){
            if !matches!(*self.expr_type,ExprType::None) {
                f(self);
            }
    }
    pub fn if_vaild_then_pass<F, T>(self, mut f: F, pass_what: T)
        where F: FnMut(T){
            if !matches!(*self.expr_type,ExprType::None) {
                f(pass_what);
            }
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f,"{:?}",*self.expr_type)
    }
}

impl std::fmt::Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"[{}:{}]:{:?}", self.line,self.at,*self.expr_type)
    }
}

impl Default for Expr {
    fn default() -> Self {
        Self {
            expr_type: Box::new(ExprType::None),
            line: 0,
            at: 0,
        }
    }
}
