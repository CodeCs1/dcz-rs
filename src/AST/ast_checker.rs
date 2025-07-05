use crate::{MessageHandler::message_handler, Value::Value};

use super::expr_node::{DataType, Expr};


#[derive(Debug)]
pub struct FAST { // AST formatter
    pub expr: Expr,
    pub is_used: bool
}

pub struct Checker<'a> {
    ast: &'a Vec<Expr>,
    pseudo_variable_stack: Vec<(DataType, String)> // DataType, Name
}


fn check_literal_type(init: Option<Box<Expr>>, dt: DataType) -> Option<Box<Expr>> {
    let mut init_v =None;
    if let Some(mut v) = init {
        let mut v = v.visit();

        let vi64 = v.clone().to_value().to_literal();                   
        match dt {
            DataType::Char => {
                if vi64 > u8::MAX as i64 {
                    message_handler::throw_message("source", message_handler::MessageType::Warning, 1, 0, format!("char overflow, rolling back from {} to {}",
                    vi64, vi64%u8::MAX as i64).as_str());
                }
                v=Expr::Literal(
                    crate::Value::Value::Number(
                    vi64%u8::MAX as i64
                    )
                );
            },
            DataType::Short => {
                if vi64 > i16::MAX as i64 {
                    message_handler::throw_message("source", message_handler::MessageType::Warning, 1, 0, format!("short overflow, rolling back from {} to {}",
                    vi64, vi64%i16::MAX as i64).as_str());
                }
                v=Expr::Literal(
                    crate::Value::Value::Number(
                        v.to_value().to_literal()%i16::MAX as i64
                    )
                )
            },
            DataType::Int => {
                if vi64 > i32::MAX as i64 {
                    message_handler::throw_message("source", message_handler::MessageType::Warning, 1, 0, format!("int overflow, rolling back from {} to {}",
                        vi64, vi64%i32::MAX as i64).as_str());
                }
                v=Expr::Literal(
                    crate::Value::Value::Number(
                    vi64 % (i32::MAX) as i64
                    )
                )
            }
            o => todo!("Data Type {:?} not yet implemented", o)
        }
        init_v = Some(Box::new(v));
    }
    init_v
}


impl<'a> Checker<'a> {

    pub fn new(ast: &'a Vec<Expr>) -> Self {
        Self { ast: ast, pseudo_variable_stack: Vec::new() }
    }

    fn visit(&mut self, expr: Expr) -> Result<FAST, String> {
        let e = expr.clone();
        match expr {
            Expr::Statement(e) => self.visit(*e),
            // bypass checking
            Expr::Return(_v) => Ok(FAST { expr: e, is_used: true }),
            Expr::Callee(_n, _e) => Ok(FAST {expr: e, is_used: true}),
            Expr::Var(_n) => Ok(FAST { expr: e, is_used: true }),

            Expr::WhileStmt(_s, _r) => Ok(FAST { expr: e, is_used: true}),

            Expr::VarDecl(dt, is_p, n, init) => {
                let init_v = check_literal_type(init, dt.clone());
                self.pseudo_variable_stack.push((dt.clone(),n.clone()));
                Ok(FAST { expr: Expr::VarDecl(dt, is_p, n, init_v), is_used: false })
            },


            Expr::Assign(n, v) => {
                if !self.pseudo_variable_stack.iter().any(|(_,vname)| *vname == n) {
                    message_handler::throw_message("source", message_handler::MessageType::Error, 1, 1, format!("Undefined variable {}", n).as_str());
                    Err("Error occured while checking source code".to_string())
                } else {
                    let (dt, _) = self.pseudo_variable_stack.iter().find(|(_,name)| *name == n).unwrap();
                    let init_v = check_literal_type(Some(v), dt.clone()).unwrap();
                    Ok(FAST { expr: Expr::Assign(n, init_v), is_used: true })
                }
            }

            Expr::Literal(_v) => Ok(FAST { expr: e, is_used: true }),
            Expr::FuncStmt(n, args, b, _ret) => {
                //self.visit(*b)
                Ok(FAST {
                    expr: Expr::FuncStmt(n, args, Box::new(self.visit(*b)?.expr),_ret),
                    is_used: false
                })
            }
            Expr::Block(b) => {
                let bl = b.iter().map(|f| self.visit(f.clone()).expect("Error").expr).collect::<Vec<Expr>>();
                Ok(FAST { expr: Expr::Block(bl),
                    is_used: true
                })
            }
            o => todo!("Expression {:?} does not implemented yet!", o)
        }
    }

    pub fn check(&mut self) -> Result<Vec<FAST>, String> {

        let mut res = Vec::new();
        res.append(&mut self.ast.iter().map(|f| self.visit(f.clone()).expect("Error")).collect::<Vec<FAST>>());

        Ok(res)
    }

}
