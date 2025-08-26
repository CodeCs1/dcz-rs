use crate::{panic_error, MessageHandler::message_handler, Value::Value};
use crate::MessageHandler::message_handler::{throw_message, MessageType};
use super::expr_node::{DataType, Expr};
use std::process::exit;

#[derive(Debug, Clone,PartialEq)]
pub struct FAST { // AST formatter
    pub expr: Expr,
    pub is_used: bool
}

struct VariableData {
    dt: DataType,
    name: String,
    is_const: bool,
    is_ptr: bool,
    init: Expr,
    is_used: bool,
}

pub struct Checker<'a> {
    ast: &'a Vec<Expr>,
    pseudo_variable_stack: Vec<VariableData>, // DataType, Name, is_const, is_ptr, init_v
    pseudo_function_stack: Vec<FAST>
}


fn check_literal_type(init: Option<Box<Expr>>, dt: DataType) -> (Option<Box<Expr>>, DataType) {
    let mut init_v =None;

    let mut data_type = dt.clone();

    if let Some(mut v) = init {
        let mut v = v.visit();

        if !matches!(v, Expr::Literal(_)) { return (Some(Box::new(v)), DataType::Unknown); }

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

            DataType::Long=> {
                if vi64 > i64::MAX as f64 {
                    message_handler::throw_message("source", message_handler::MessageType::Warning, 1, 0, format!("suu (double) overflow, rolling back from {} to {}",
                        vi64, vi64%i64::MAX as f64).as_str());
                }
                v=Expr::Literal(
                    crate::Value::Value::Number(
                    vi64 as i64 % i64::MAX
                    )
                )
            }
            DataType::Unknown => {
                data_type=v.to_value().to_datatype();
            }
            o => todo!("Data Type {:?} not yet implemented", o)
        }
        init_v = Some(Box::new(v));
    }
    (init_v,data_type)
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
                if !self.pseudo_function_stack.iter().any(|f| {
                    let func = f.expr.get_function();
                    n.ident_to_string() == func.0
                }) {
                    return Err(format!("Function '{}' not declared!", n.ident_to_string()));
                }

                Ok(FAST {expr: e, is_used: true})
            },
            Expr::Var(n) => {
                if let Some(idx) = self.pseudo_variable_stack.iter().position(|f| {
                    f.name == n
                }) {
                    self.pseudo_variable_stack[idx].is_used = true;
                    Ok(FAST { expr: self.pseudo_variable_stack[idx].init.clone(), is_used: true })
                } else {
                    Err(format!("Variable '{}' not declared!", n))
                }
            }
            Expr::WhileStmt(_s, _r) => Ok(FAST { expr: e, is_used: true}),

            Expr::VarDecl(dt, is_p,is_const, n, init) => {
                let (init_v,data_type) = check_literal_type(init, dt.clone());

                if self.pseudo_variable_stack.iter().find(|f| {
                    f.name == n
                }).is_some() {
                    return Err(format!("Variable '{}' already defined", n));
                }
                let k = self.visit(*init_v.clone().unwrap());

                self.pseudo_variable_stack.push(
                    VariableData { dt: dt, name: n.clone(), is_const: is_const, is_ptr: is_p, init: k.clone()?.expr, is_used: false }
                );
                Ok(FAST { expr: Expr::VarDecl(data_type, is_p,is_const, n, Some(Box::new(k?.expr))), is_used: false })
            },


            Expr::Assign(n, v) => {
                if !self.pseudo_variable_stack.iter().any(|v| *v.name == n) {
                    Err(format!("Undefined variable {}", n))
                } else {
                    let assign = self.pseudo_variable_stack.iter().find(|v| v.name == n).unwrap();

                    if assign.is_const {
                        return Err(format!("Constant variable '{}' cannot be assignable!", n));
                    }
                    let init_v = check_literal_type(Some(v), assign.dt.clone()).0.unwrap();
                    Ok(FAST { expr: Expr::Assign(n, init_v), is_used: true })
                }
            }

            Expr::Literal(_v) => Ok(FAST { expr: e, is_used: true }),
            Expr::FuncStmt(n, args, b, ret) => {
                //self.visit(*b)
                //add to pseudo_variable_stack

                let f=FAST {
                    expr: Expr::FuncStmt(n, args, Box::new(self.visit(*b)?.expr),ret),
                    is_used: true
                };

                self.pseudo_function_stack.push(f.clone());

                Ok(f)
            }
            Expr::Block(b) => {
                let bl = self.check_ast(b)?;
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
            Expr::IfStmt(cond,then_bl ,else_bl ) => {
                let cond = self.visit(*cond)?;
                if matches!(cond.expr, Expr::Literal(_)) {
                    if cond.expr.to_value() == Value::Number(0) {
                        if matches!(*else_bl, Expr::None) {
                            return Ok(
                                FAST{
                                    expr: Expr::None,
                                    is_used: false
                                }
                            );
                        } else {
                            return Ok(FAST { expr: *else_bl, is_used: true });
                        }
                    } else {
                        return Ok(FAST { expr: *then_bl, is_used: true });
                    }
                }
                let then_bl = self.visit(*then_bl)?;
                let else_bl = if !matches!(*else_bl, Expr::None) {
                    self.visit(*else_bl)?
                } else {
                    FAST {expr: Expr::None, is_used: false}
                };
                Ok(FAST {
                    expr: Expr::IfStmt(Box::new(cond.expr), Box::new(then_bl.expr), Box::new(else_bl.expr)),
                    is_used: true
                })
            }
            o => todo!("Expression {:?} does not implemented yet!", o)
        }
    }

    fn check_ast(&mut self, ast: Vec<Expr>) -> Result<Vec<Expr>, String> {
        let mut res = Vec::new();
        let mut original_fast = ast.iter().map(|f|
            catch_error(self.visit(f.clone()))).collect::<Vec<FAST>>();

        for func_f in &self.pseudo_function_stack {
            if let Some(idx) = original_fast.iter().position(|f|{
                f.expr.get_function().0 == func_f.expr.get_function().0
            }) {
                original_fast[idx] = func_f.clone();
            }
        }

        for var_decl in &self.pseudo_variable_stack {
            if let Some(idx) = original_fast.iter().position(|f|{
                if let Expr::VarDecl(_, _, _, name, _) = &f.expr {
                    *name == var_decl.name && var_decl.is_used
                } else {
                    false
                }
            }) {
                original_fast[idx].is_used=true;
            }
        }

        res.append(
            &mut 
            original_fast.into_iter().filter(|f| {
                f.is_used
            }).map(|f| {
                f.expr
            }).collect()
        );

        Ok(res)
    }

    pub fn check(&mut self) -> Result<Vec<Expr>, String> {
        self.check_ast(self.ast.to_vec())
    }

}
