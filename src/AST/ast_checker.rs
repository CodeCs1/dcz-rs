use crate::{panic_error, MessageHandler::message_handler, Value::Value};
use crate::MessageHandler::message_handler::{throw_message, MessageType};
use super::expr_node::{DataType, Expr};
use std::process::exit;

#[derive(Debug, Clone)]
pub struct FAST { // AST formatter
    pub expr: Expr,
    pub is_used: bool
}

pub struct Checker<'a> {
    ast: &'a Vec<Expr>,
    pseudo_variable_stack: Vec<(DataType, String, bool)>, // DataType, Name, is_const
    pseudo_function_stack: Vec<FAST>
}


fn check_literal_type(init: Option<Box<Expr>>, dt: DataType) -> Option<Box<Expr>> {
    let mut init_v =None;

    if let Some(mut v) = init {
        let mut v = v.visit();

        let to_v= v.to_value();
        let vi64 = if to_v.clone().is_literal() {
            to_v.to_literal() as f64
        } else if to_v.clone().is_double() {
            to_v.to_double() as f64
        }
        else {
            (to_v.to_char() as u64) as f64
        };


        match dt {
            DataType::Char => {
                if vi64 > u8::MAX as f64 {
                    message_handler::throw_message("source", message_handler::MessageType::Warning, 1, 0, format!("char overflow, rolling back from {} to {}",
                    vi64, vi64%u8::MAX as f64).as_str());
                }
                v=Expr::Literal(
                    crate::Value::Value::Number(
                    (vi64%256 as f64) as i64
                    )
                );
            },
            DataType::Short => {
                if vi64 > i16::MAX as f64 {
                    message_handler::throw_message("source", message_handler::MessageType::Warning, 1, 0, format!("short overflow, rolling back from {} to {}",
                    vi64, vi64%i16::MAX as f64).as_str());
                }
                v=Expr::Literal(
                    crate::Value::Value::Number(
                        (v.to_value().to_literal()%i16::MAX as i64) as i64
                    )
                )
            },
            DataType::Int => {
                if vi64 > i32::MAX as f64 {
                    message_handler::throw_message("source", message_handler::MessageType::Warning, 1, 0, format!("int overflow, rolling back from {} to {}",
                        vi64, vi64%i32::MAX as f64).as_str());
                }
                v=Expr::Literal(
                    crate::Value::Value::Number(
                    (vi64 % i32::MAX as f64) as i64
                    )
                )
            },
            DataType::Suu => {
                if vi64 > f64::MAX as f64 {
                    message_handler::throw_message("source", message_handler::MessageType::Warning, 1, 0, format!("suu (double) overflow, rolling back from {} to {}",
                        vi64, vi64%f64::MAX as f64).as_str());
                }
                v=Expr::Literal(
                    crate::Value::Value::Double(
                    vi64 % f64::MAX as f64
                    )
                )
            }
            o => todo!("Data Type {:?} not yet implemented", o)
        }
        init_v = Some(Box::new(v));
    }
    init_v
}

pub fn catch_error(r: Result<FAST, String>) -> FAST {
    match r {
        Ok(f) => f,
        Err(s) => {
            panic_error!("stdin", 1, 0, &s);
        }
    }
}

impl<'a> Checker<'a> {

    pub fn new(ast: &'a Vec<Expr>) -> Self {
        Self { ast: ast, pseudo_variable_stack: Vec::new(), pseudo_function_stack: Vec::new() }
    }

    fn visit(&mut self, expr: Expr) -> Result<FAST, String> {
        let e = expr.clone();
        match expr {
            Expr::Statement(e) => self.visit(*e),
            // bypass checking
            Expr::Return(_v) => Ok(FAST { expr: e, is_used: true }),
            Expr::Callee(n, _e) => { 
                let a = self.pseudo_function_stack.iter().find(|f| {
                    let (func_name,_,_,_) = f.expr.get_function();
                    n.ident_to_string() == func_name
                });
                if a.is_none() {
                   return Err(format!("Function '{}' not declared!", n.ident_to_string()));
                }
                Ok(FAST {expr: e, is_used: true}) 
            },
            Expr::Var(n) => {
                if self.pseudo_variable_stack.iter().find(|f| {
                    f.1 == n
                }).is_none() {
                    return Err(format!("Variable '{}' not declared!", n))
                }
                Ok(FAST { expr: e, is_used: true })
            }
            Expr::WhileStmt(_s, _r) => Ok(FAST { expr: e, is_used: true}),

            Expr::VarDecl(dt, is_p,is_const, n, init) => {
                let init_v = check_literal_type(init, dt.clone());
                self.pseudo_variable_stack.push((dt.clone(),n.clone(), is_const));
                Ok(FAST { expr: Expr::VarDecl(dt, is_p,is_const, n, init_v), is_used: false })
            },


            Expr::Assign(n, v) => {
                if !self.pseudo_variable_stack.iter().any(|(_,vname,_)| *vname == n) {
                    Err(format!("Undefined variable {}", n))
                } else {
                    let (dt, _,is_const) = self.pseudo_variable_stack.iter().find(|(_,name,_)| *name == n).unwrap();

                    if *is_const {
                        return Err(format!("Constant variable '{}' cannot be assignable!", n));
                    }
                    let init_v = check_literal_type(Some(v), dt.clone()).unwrap();
                    Ok(FAST { expr: Expr::Assign(n, init_v), is_used: true })
                }
            }

            Expr::Literal(_v) => Ok(FAST { expr: e, is_used: true }),
            Expr::FuncStmt(n, args, b, ret) => {
                //self.visit(*b)
                //add to pseudo_variable_stack

                let f=FAST {
                    expr: Expr::FuncStmt(n, args, Box::new(self.visit(*b)?.expr),ret),
                    is_used: false
                };

                self.pseudo_function_stack.push(f.clone());

                Ok(f)
            }
            Expr::Block(b) => {
                let bl = b.iter().map(|f| catch_error(self.visit(f.clone())).expr).collect::<Vec<Expr>>();
                Ok(FAST { expr: Expr::Block(bl),
                    is_used: true
                })
            }
            Expr::Binary(mut lhs, op, mut rhs) => {
                let lhs = self.visit(lhs.visit());
                let rhs = self.visit(rhs.visit());

                let e = Expr::Binary(Box::new(lhs?.expr),op,Box::new(rhs?.expr)).visit();

                Ok(
                    FAST {
                        expr: e,
                        is_used: true
                    }
                )
            }
            o => todo!("Expression {:?} does not implemented yet!", o)
        }
    }

    pub fn check(&mut self) -> Result<Vec<FAST>, String> {

        let mut res = Vec::new();
        res.append(&mut self.ast.iter().map(|f| catch_error(self.visit(f.clone()))).collect::<Vec<FAST>>());

        Ok(res)
    }

}
