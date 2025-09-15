#![allow(dead_code)]

use crate::{token::TokenData, Value::{self}};


#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Char,
    Short,
    Int,
    Long,
    Float,
    Suu, // replace for double data type
    Void,
    Unknown
}

impl DataType {
    pub fn size(&self) -> u32 {
        match self {
            DataType::Char => 1,
            DataType::Short => 2,
            DataType::Int | DataType::Float => 4,
            DataType::Long | DataType::Suu => 8,
            _ => 0
        }
    }
}

#[derive(Debug, Clone,PartialEq)]
pub struct Func_Header {
    pub name: String,
    pub args: Vec<(DataType,String)>,
    pub return_type: Option<DataType>,
    pub is_ptr_dt: bool
}

#[derive(Debug, Clone,PartialEq)]
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
    /// FuncStmt(name, args, body, return_type)
    FuncStmt(Func_Header, Box<Expr>),
    Callee(Box<Expr>, Vec<Expr>),

    /// Var declare Statement VarDecl(dt, is_pointer, is_constant, name, initializer)
    VarDecl(DataType, bool, bool, String, Option<Box<Expr>>),
    List(Vec<Value::Value>),
    Return(Option<Box<Expr>>),

    /// Extern declare statement
    Extern(Func_Header),

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


                let result = if matches!(lhs, Expr::Literal(_)) && matches!(rhs, Expr::Literal(_)) {
                    match op.tok_type {
                        crate::token::token_type::TokenType::Plus => Expr::Literal(lhs.to_value()+rhs.to_value()),
                        crate::token::token_type::TokenType::Minus => Expr::Literal(lhs.to_value()-rhs.to_value()),
                        crate::token::token_type::TokenType::Star => Expr::Literal(lhs.to_value()*rhs.to_value()),
                        crate::token::token_type::TokenType::Slash => Expr::Literal(lhs.to_value()/rhs.to_value()),
                        crate::token::token_type::TokenType::Less => Expr::Literal(Value::Value::Number((lhs.to_value()<rhs.to_value()) as i64)),
                        crate::token::token_type::TokenType::Greater => Expr::Literal(Value::Value::Number((lhs.to_value()>rhs.to_value()) as i64)),
                        crate::token::token_type::TokenType::LessEqual => Expr::Literal(Value::Value::Number((lhs.to_value()<=rhs.to_value()) as i64)),
                        crate::token::token_type::TokenType::GreaterEqual => Expr::Literal(Value::Value::Number((lhs.to_value()>=rhs.to_value()) as i64)),
                        crate::token::token_type::TokenType::EqualEqual => Expr::Literal(Value::Value::Number((lhs.to_value()==rhs.to_value()) as i64)),
                        crate::token::token_type::TokenType::ShiftLeft => Expr::Literal(Value::Value::Number((lhs.to_value()<<rhs.to_value()).to_literal())),
                        crate::token::token_type::TokenType::ShiftRight => Expr::Literal(Value::Value::Number((lhs.to_value()>>rhs.to_value()).to_literal())),
                        crate::token::token_type::TokenType::Or => Expr::Literal(Value::Value::Number((lhs.to_value()|rhs.to_value()).to_literal())),
                        _ => {
                            unimplemented!()
                        }
                    }
                } else {
                    Expr::Binary(Box::new(lhs), op.clone(),Box::new(rhs))
                };

                result

            },
            Expr::Unary(op, rhs) => {
                let rhs = rhs.visit();
                Expr::Literal(match op.tok_type {
                    crate::token::token_type::TokenType::Minus => -rhs.to_value(),
                    crate::token::token_type::TokenType::Not => !rhs.to_value(),
                    _ => unimplemented!()
                })
            },
            Expr::VarDecl(_,_,_,_,_) => {
                Expr::None
            }
            Expr::Var(_) => self.clone(),
            Expr::Statement(st) => st.visit(),
            Expr::Callee(_, _) => self.clone(),
            o => todo!("Expr visit does not implemented {:?} yet ", o)
        }
    }
    pub fn to_value(&self) -> Value::Value {
        match self {
            Expr::Literal(v) => v.clone(),
            Expr::List(l) => Value::Value::List(l.to_vec()),
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
    pub fn get_function(&self) -> (String, Vec<(DataType, String)>, Box<Expr>, Option<DataType>) {
        match self {
            Expr::FuncStmt(func_header, body ) => {
                (func_header.name.clone(),
                 func_header.args.clone(),
                body.clone(),
                func_header.return_type.clone())
            }
            Expr::Extern(f) => {
                (f.name.clone(),
                f.args.clone(),
                Box::new(Expr::None),
                f.return_type.clone())
            }
            _ => unimplemented!()
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
                    "void" => Ok(DataType::Void),
                    _ => Ok(DataType::Unknown)
                }
            }
            _ => Err(format!("Expr type expect to be identifier, got {:#?}", self))
        }
    }
}
