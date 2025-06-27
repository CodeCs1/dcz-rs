#![allow(dead_code)]

use crate::{token::TokenData, Value};


#[derive(Debug, Clone)]
pub enum DataType {
    Char,
    Short,
    Int,
    Long,
    Float,
    Suu // replace for double data type
}

#[derive(Debug, Clone)]
pub enum Expr {
    /// Binary Expression (Expr, Operator, Expr)
    Binary(Box<Expr>, TokenData, Box<Expr>),
    Literal(Value::Value),
    Unary(TokenData, Box<Expr>),
    Grouping(Box<Expr>),
    Macro(String,Vec<Expr>),
    Identifier(String),
    Var(String),
    Statement(Box<Expr>),
    Block(Vec<Expr>),
    Assign(String, Box<Expr>),

    IfStmt(Box<Expr>, Box<Expr>, Box<Expr>),
    WhileStmt(Box<Expr>, Box<Expr>),
    FuncStmt(String, Vec<Expr>, Box<Expr>),
    
    Callee(Box<Expr>, Vec<Expr>),

    /// Var declare Statement VarDecl(dt, is_pointer, name, initializer)
    VarDecl(DataType, bool,  String, Option<Box<Expr>>),

    None
}

impl<'a> Expr
{
    pub fn visit(&mut self) -> Expr {
        match self {
            Expr::Literal(_) => self.clone(),
            Expr::Macro(_,_) => Expr::None,
            Expr::Grouping(expr) => expr.visit().clone(),
            Expr::Binary(lhs, op, rhs) => {
                let lhs = lhs.visit();
                let rhs = rhs.visit();

                let mut result = Value::Value::Null;

                if matches!(lhs, Expr::Literal(_)) && matches!(rhs, Expr::Literal(_)) {
                    result=match op.tok_type {
                        crate::token::token_type::TokenType::Plus => lhs.to_value()+rhs.to_value(),
                        crate::token::token_type::TokenType::Minus => lhs.to_value()-rhs.to_value(),
                        crate::token::token_type::TokenType::Star => lhs.to_value()*rhs.to_value(),
                        crate::token::token_type::TokenType::Slash => lhs.to_value()/rhs.to_value(),
                        crate::token::token_type::TokenType::Less => Value::Value::Number((lhs.to_value()<rhs.to_value()) as i64),
                        crate::token::token_type::TokenType::Greater => Value::Value::Number((lhs.to_value()>rhs.to_value()) as i64),
                        crate::token::token_type::TokenType::LessEqual => Value::Value::Number((lhs.to_value()<=rhs.to_value()) as i64),
                        crate::token::token_type::TokenType::GreaterEqual => Value::Value::Number((lhs.to_value()>=rhs.to_value()) as i64),
                        crate::token::token_type::TokenType::EqualEqual => Value::Value::Number((lhs.to_value()==rhs.to_value()) as i64),
                        _ => {
                            unimplemented!()
                        }
                    };
                }

                Expr::Literal(result).clone()

            },
            Expr::Unary(op, rhs) => {
                let rhs = rhs.visit();
                Expr::Literal(match op.tok_type {
                    crate::token::token_type::TokenType::Minus => -rhs.to_value(),
                    crate::token::token_type::TokenType::Not => !rhs.to_value(),
                    _ => unimplemented!()
                })
            }
            _ => todo!()
        }
    }
    pub fn to_value(&self) -> Value::Value {
        match self {
            Expr::Literal(v) => v.clone(),
            _ => Value::Value::Null
        }
    }
    pub fn ident_to_string(&self) -> String {
        match self {
            Expr::Identifier(s) => s.clone(),
            Expr::Var(s) => s.clone(),
            _ => "".to_string()
        }
    }
    pub fn to_datatype(&self) -> Result<DataType, String> {
        match self {
            Expr::Identifier(n) => {
                match n.as_str() {
                    "char" => Ok(DataType::Char),
                    "short" => Ok(DataType::Short),
                    "int" => Ok(DataType::Int),
                    "long" => Ok(DataType::Long),
                    "float" => Ok(DataType::Float),
                    "suu" => Ok(DataType::Suu),
                    _ => Err(format!("Invaild data type {}", n)),
                }
            }
            _ => Err(format!("Expr type expect to be identifier, got {:#?}", self))
        }
    }
}
