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
    Unknown,
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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Variable {
    pub vData: VariableType,
    pub name: String,
    pub init: Option<Expr>,
    pub is_used: bool,
}

impl Variable {
    pub fn new(vData: VariableType, name: String, init: Option<Expr>, is_used: bool) -> Self {
        // try to guess data type from init value
        println!("{}", vData);
        let mut vData_new = vData;

        if init.is_some() {
            //let expr = init.clone().unwrap();
            // match expr {
            //     Expr::Literal(v) => {
            //         vData_new=match vData_new {
            //             VariableType::Constant(_) => VariableType::Constant(v.val_type),
            //             VariableType::NonPointer(_) => VariableType::NonPointer(v.val_type),
            //             VariableType::Pointer(_) => VariableType::Pointer(v.val_type)
            //         }
            //     }
            //     _ => {}
            // }
        }

        Self {
            vData: vData_new,
            name,
            init,
            is_used,
        }
    }
    pub fn is_constant(self) -> bool {
        self.vData.is_constant()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuncHeader {
    pub name: String,
    pub args: Vec<Variable>,
    pub return_type: Option<VariableType>,
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
    Block(Expr),
    Assign(String, Expr),

    IfStmt(Expr, Expr, Expr),
    WhileStmt(Expr, Expr),
    /// FuncStmt(name, args, body, return_type)
    FuncStmt(FuncHeader, Expr),
    Callee(Expr, Vec<Expr>),

    /// Var declare Statement VarDecl(dt, is_pointer, is_constant, name, initializer)
    VarDecl(VariableType, String, Option<Expr>),
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

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    expr_type: Box<ExprType>,
    line: usize,
    at: usize,
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

// impl ExprType {
//     pub fn visit(&mut self) -> Result<ExprType, String> {
//         match self {
//             ExprType::Literal(_) => Ok(self.clone()),
//             ExprType::Macro(_, _) => Ok(ExprType::None),
//             ExprType::Grouping(expr) => expr.visit().clone(),
//             ExprType::Binary(lhs, op, rhs) => {
//                 let lhs = lhs.visit()?;
//                 let rhs = rhs.visit()?;

//                 let result = if let Expr::Literal(lhs_val) = lhs.clone() && let Expr::Literal(rhs_val) = rhs.clone() {
//                     let v = match op.tok_type {
//                         crate::token::token_type::TokenType::Plus => {
//                             lhs_val.val+rhs_val.val
//                         }
//                         crate::token::token_type::TokenType::Minus => {
//                             lhs_val.val-rhs_val.val
//                         }
//                         crate::token::token_type::TokenType::Star => {
//                             lhs_val.val*rhs_val.val
//                         }
//                         crate::token::token_type::TokenType::Slash => {
//                             lhs_val.val/rhs_val.val
//                         }
//                         crate::token::token_type::TokenType::Less => {
//                             Ok(Value::Value::Boolean(lhs_val.val<rhs_val.val))
//                         }
//                         crate::token::token_type::TokenType::LessEqual => {
//                             Ok(Value::Value::Boolean(lhs_val.val<=rhs_val.val))
//                         }
//                         crate::token::token_type::TokenType::GreaterEqual => {
//                             Ok(Value::Value::Boolean(lhs_val.val>=rhs_val.val))
//                         }
//                         crate::token::token_type::TokenType::EqualEqual => {
//                             Ok(Value::Value::Boolean(lhs_val.val==rhs_val.val))
//                         }
//                         crate::token::token_type::TokenType::ShiftLeft => {
//                             lhs_val.val<<rhs_val.val
//                         }
//                         crate::token::token_type::TokenType::ShiftRight => {
//                             lhs_val.val>>rhs_val.val
//                         }
//                         crate::token::token_type::TokenType::Or => lhs_val.val|rhs_val.val,
//                         crate::token::token_type::TokenType::And => lhs_val.val & rhs_val.val,
//                         _ => {
//                             unimplemented!()
//                         }
//                     }?;
//                     Ok(Expr::Literal(TypedValue::new(v,false)))
//                 } else {
//                     Ok(Expr::Binary(Box::new(lhs), op.clone(), Box::new(rhs)))
//                 };

//                 result
//             }
//             Expr::Unary(op, rhs) => {
//                 let rhs = rhs.visit()?;
//                 if let Expr::Literal(lit) = rhs {
//                     let v = match op.tok_type {
//                         crate::token::token_type::TokenType::Minus => -lit.val,
//                         crate::token::token_type::TokenType::Not => !lit.val,
//                         _ => unimplemented!(),
//                     }?;
//                     Ok(Expr::Literal(TypedValue::new(v,false)))
//                 } else {
//                     Ok(self.clone())
//                 }
//             }
//             Expr::VarDecl(_, _, _) => Ok(Expr::None),
//             Expr::Statement(st) => st.visit(),
//             Expr::Cast(_, _) |Expr::Var(_) | Expr::Callee(_, _) => Ok(self.clone()),
//             o => todo!("Expr visit does not implement {:?} yet ", o),
//         }
//     }
//     pub fn to_value(&self) -> Value::Value {
//         match self {
//             Expr::Literal(v) => v.clone().val,
//             Expr::List(l) => Value::Value::List(l.to_vec()),
//             _ => Value::Value::Null,
//         }
//     }
//     pub fn ident_to_string(&self) -> String {
//         match self {
//             Expr::Identifier(s) => s.clone(),
//             Expr::Var(s) => s.clone(),
//             _ => "".to_string(),
//         }
//     }
//     pub fn get_function(&self) -> (String, Vec<Variable>, Box<Expr>, Option<VariableType>) {
//         match self {
//             Expr::FuncStmt(func_header, body) => (
//                 func_header.name.clone(),
//                 func_header.args.clone(),
//                 body.clone(),
//                 func_header.return_type.clone(),
//             ),
//             Expr::Extern(f) => (
//                 f.name.clone(),
//                 f.args.clone(),
//                 Box::new(Expr::None),
//                 f.return_type.clone(),
//             ),
//             e => unimplemented!("{:?}", e),
//         }
//     }

//     pub fn to_datatype(&self) -> Result<DataType, String> {
//         match self {
//             Expr::Identifier(n) => match n.as_str() {
//                 "i8" => Ok(DataType::I8),
//                 "i16" => Ok(DataType::I16),
//                 "i32" => Ok(DataType::I32),
//                 "i64" => Ok(DataType::I64),

//                 "u8" => Ok(DataType::U8),
//                 "u16" => Ok(DataType::U16),
//                 "u32" => Ok(DataType::U32),
//                 "u64" => Ok(DataType::U64),

//                 "f32" => Ok(DataType::F32),
//                 "f64" => Ok(DataType::F64),
//                 "void" => Ok(DataType::Void),
//                 _ => Ok(DataType::Unknown),
//             },
//             Expr::Literal(v) => Ok(v.clone().val_type),
//             _ => Err(format!(
//                 "Expr type expect to be identifier, got {:#?}",
//                 self
//             )),
//         }
//     }
// }
