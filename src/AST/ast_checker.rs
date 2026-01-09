/*
    This AST Checker one will do:
     + passes on some basic optimization including:
        Constant folding
        Unused variable and extern function
        Limit some data type
     + Basic type checking

*/


use crate::AST::expr_node::{ClassFunction, FuncHeader, VariableData};
use crate::{panic_error, Value::Value};
use crate::MessageHandler::{throw_message, MessageType};
use super::expr_node::{DataType, Expr, Variable};
use std::collections::{BTreeMap, HashMap};
use std::process::exit;

macro_rules! rolling_back_dt {
    ($dt: ident, $vi64: ident, $v: ident) => {
        if $vi64 > $dt::MAX as f64 {
            throw_message("source", MessageType::Warning, 1, 0, format!("datatype overflow, rolling back from {} to {}",
            $vi64, $vi64%$dt::MAX as f64).as_str());
        }
        $v=Expr::Literal(
            crate::Value::Value::Number(
            ($vi64%$dt::MAX as f64) as i64
            )
        );
    };
}


#[derive(Debug, Clone,PartialEq)]
pub struct FAST { // AST formatter
    pub expr: Expr,
    pub is_used: bool
}


pub struct Checker<'a> {
    ast: &'a Vec<Expr>,
    pseudo_variable_stack: Vec<Variable>, // DataType, Name, is_const, is_ptr, init_v
    pseudo_function_stack: BTreeMap<String, Option<FAST>>,
    /// Hashmap (Function Header, is Used)
    extern_function_stack: BTreeMap<String, (FuncHeader, bool)>,
    macro_define: HashMap<String, Vec<Expr>>,
    filename: String
}



fn check_literal_type(init: Option<Box<Expr>>, dt: DataType, is_ptr: bool) ->
    Result<(Option<Box<Expr>>, DataType, bool),String>
    {
    let mut init_v =None;

    let mut data_type = dt.clone();

    if matches!(data_type, DataType::Void) && !is_ptr {
        return Err(format!("'void' cannot be used like normal datatype\nUse different data type or add '*' at the end of 'void' data type."));
    }

    if let Some(mut v) = init {
        let mut v = v.visit();

        if !matches!(v, Expr::Literal(_)) ||
        (matches!(dt, DataType::Void) && is_ptr) { return Ok((Some(Box::new(v)), dt, is_ptr)); }


        let to_v= v.to_value();

        if to_v.clone().is_string() {
            if matches!(dt, DataType::Unknown) { // 'let' keyword
                return Ok((Some(Box::new(v)), DataType::I8, true));
            }
            if !is_ptr {
                return Err(format!("Cannot convert from {:?} to string literal (variable MUST be pointer and data type MUST be char)", data_type.clone()));
            }
            if is_ptr && (!matches!(dt, DataType::I8)  && !matches!(dt, DataType::U8)) {
                return Err(format!("Cannot convert from {:?}* to string literal (data type MUST be char)", dt.clone()));
            }
        }

        if is_ptr {
            return Ok((Some(Box::new(v)), dt, true));
        }

        let vi64 = if to_v.clone().is_literal() {
            to_v.to_literal() as f64
        } else if to_v.clone().is_double() {
            to_v.to_double() as f64
        }
        else {
            (to_v.to_char() as u64) as f64
        };


        match dt {
            DataType::I8 => {
                rolling_back_dt!(i8, vi64,v);
            },
            DataType::I16 => {
                rolling_back_dt!(i16, vi64,v);
            },
            DataType::I32 => {
                rolling_back_dt!(i32, vi64,v);
            },
            DataType::I64 => {
                rolling_back_dt!(i64, vi64,v);
            },

            DataType::U8 => {
                rolling_back_dt!(u8, vi64,v);
            },
            DataType::U16 => {
                rolling_back_dt!(u16, vi64,v);
            },
            DataType::U32 => {
                rolling_back_dt!(u32, vi64,v);
            },
            DataType::U64 => {
                rolling_back_dt!(u64, vi64,v);
            },

            DataType::Unknown => {
                data_type=v.to_value().to_datatype();
            },
            o => todo!("Data type \"{:#?}\" not yet implemented!.", o)
        }
        init_v = Some(Box::new(v));
    }
    Ok((init_v,data_type, is_ptr))
}

pub fn catch_error(r: Result<Option<FAST>, String>) -> Option<FAST> {
    match r {
        Ok(f) => f,
        Err(s) => {
            panic_error!("s", 1, 0, &s);
        }
    }
}

impl<'a> Checker<'a> {

    pub fn new(ast: &'a Vec<Expr>, file: String) -> Self {
        Self {
            ast: ast,
            pseudo_variable_stack: Vec::new(),
            pseudo_function_stack: BTreeMap::new(),
            extern_function_stack: BTreeMap::new(),
            macro_define: HashMap::new(),
            filename: file
        }
    }

    fn visit(&mut self, expr: Expr) -> Result<Option<FAST>, String> {
        let e = expr.clone();
        match expr {
            Expr::Statement(e) => self.visit(*e),
            Expr::Return(v) =>  {
                let ret = if let Some(ret_v) = v {
                    self.visit(*ret_v)?
                } else {
                    None
                };
                Ok (
                    Some(
                        FAST {
                            expr: Expr::Return({
                                if ret.is_none() {
                                    None
                                } else {
                                    Some(Box::new(ret.unwrap().expr))
                                }
                            }
                            ),
                            is_used: true
                        }
                    )
                )
            },
            Expr::Callee(n, args) => {

                let name = n.ident_to_string();
                if !self.pseudo_function_stack.contains_key(&name) {
                    if !self.extern_function_stack.contains_key(&name) {
                        return Err(format!("Function '{}' not declared!", name));
                    } else {
                        // set used extern function to true
                        if
                        let Some(extern_func)
                        = self.extern_function_stack.get_mut(&name) {
                            *extern_func = (extern_func.0.clone(), true)
                        }
                    }
                }

                let args = self.check_ast(args)?;

                Ok(Some(FAST {expr: Expr::Callee(n, args), is_used: true}))
            },
            Expr::Var(n) => {
                if let Some(idx) = self.pseudo_variable_stack.iter().position(|f| {
                    f.name == n
                }) {
                    self.pseudo_variable_stack[idx].is_used = true;
                    Ok(Some(FAST { expr:e, is_used: true })) // let codegen do the rest
                } else {
                    // maybe it's in macro define hashmap

                    if let Some(idx) = self.macro_define.iter().find(|f| {
                        f.0 == n.as_str()
                    }) {
                        Ok(
                            Some(
                                FAST {
                                    expr: idx.1[0].clone(),
                                    is_used: true
                                }
                            )
                        )
                    } else {
                        // yea, just give up already =P
                        Err(format!("Variable '{}' not declared!", n))
                    }
                }
            }
            Expr::WhileStmt(condition, expr) => {
                let cond = self.visit(*condition)?.unwrap();
                let expr = self.visit(*expr)?.unwrap();
                Ok(Some(
                    FAST {
                        expr: Expr::WhileStmt(
                            Box::new(cond.expr),
                            Box::new(expr.expr)
                        ),
                        is_used: true
                        }
                    )
                )
            },

            Expr::VarDecl(dt, is_p,is_const, n, init) => {
                let (init_v,data_type,is_str) =
                    match check_literal_type(init, dt.clone(), is_p) {
                        Ok(v) => (v.0,v.1,v.2),
                        Err(s) => {
                            panic_error!(&self.filename, 1, 1, s.as_str());
                        }
                    };

                if self.pseudo_variable_stack.iter().find(|f| {
                    f.name == n
                }).is_some() {
                    return Err(format!("Variable '{}' already defined", n));
                }
                let k = self.visit(*init_v.clone().unwrap())?.unwrap();

                self.pseudo_variable_stack.push(
                    //VariableData { dt: dt, name: n.clone(), is_const: is_const, is_ptr: is_p, init: Some(k.clone().expr), is_used: false }

                    Variable { 
                        vData: VariableData { 
                            dt, 
                            isConst: is_const, 
                            isPtr: is_p, 
                            isUnsigned: false 
                        }, 
                        name: n.clone(), 
                        init: Some(k.clone().expr), 
                        is_used: false 
                    }
                );
                Ok(Some(FAST { expr: Expr::VarDecl(data_type, is_str,is_const, n, Some(Box::new(k.expr))), is_used: false }))
            },


            Expr::Assign(n, v) => {
                if !self.pseudo_variable_stack.iter().any(|v| *v.name == n) {
                    Err(format!("Undefined variable {}", n))
                } else {
                    let assign = self.pseudo_variable_stack.iter().find(|v| v.name == n).unwrap();

                    if assign.vData.isConst {
                        return Err(format!("Constant variable '{}' cannot be assignable!", n));
                    }
                    let init_v =
                        if let Ok(v) = check_literal_type(Some(v), assign.vData.dt.clone(), assign.vData.isPtr) {
                            v.0.unwrap()
                        } else {
                            exit(1);
                        };
                    Ok(Some(FAST { expr: Expr::Assign(n, init_v), is_used: true }))
                }
            }

            Expr::Literal(_v) => Ok(Some(FAST { expr: e, is_used: true })),
            Expr::FuncStmt(f,body) => {
                //self.visit(*b)
                //add to pseudo_variable_stack

                self.pseudo_function_stack.insert(f.name.clone(), None);

                //temporary add args into variable stack
                self.pseudo_variable_stack.append(&mut f.args.clone());

                //count for 'return' keyword


                let fast=FAST {
                    expr: Expr::FuncStmt(f.clone(), Box::new(self.visit(*body)?.unwrap().expr)),
                    is_used: true
                };

                if let Some(f) = self.pseudo_function_stack.get_mut(&(f.name)) {
                    *f = Some(fast.clone());
                }

                //and then remove it from variable stack
                for _ in 0..f.args.len() {
                    self.pseudo_variable_stack.pop();
                }

                Ok(Some(fast))
            }
            Expr::Block(b) => {
                let bl = self.check_ast(b)?;
                Ok(Some(FAST { expr: Expr::Block(bl),
                    is_used: true
                }))
            }
            Expr::Binary(mut lhs, op, mut rhs) => {
                let lhs = self.visit(lhs.visit())?.unwrap();
                let rhs = self.visit(rhs.visit())?.unwrap();

                let e = Expr::Binary(Box::new(lhs.expr),op,Box::new(rhs.expr)).visit();

                Ok(
                    Some(FAST {
                        expr: e,
                        is_used: true
                    })
                )
            }
            Expr::IfStmt(cond,then_bl ,else_bl ) => {
                let cond = self.visit(*cond)?.unwrap();
                if matches!(cond.expr, Expr::Literal(_)) {
                    if cond.expr.to_value() == Value::Number(0) {
                        if matches!(*else_bl, Expr::None) {
                            return Ok(
                                Some(FAST{
                                    expr: Expr::None,
                                    is_used: false
                                })
                            );
                        } else {
                            return Ok(Some(FAST { expr: *else_bl, is_used: true }));
                        }
                    } else {
                        return Ok(Some(FAST { expr: *then_bl, is_used: true }));
                    }
                }
                let then_bl = self.visit(*then_bl)?.unwrap();
                let else_bl = if !matches!(*else_bl, Expr::None) {
                    self.visit(*else_bl)?.unwrap()
                } else {
                    FAST {expr: Expr::None, is_used: false}
                };
                Ok(Some(FAST {
                    expr: Expr::IfStmt(Box::new(cond.expr), Box::new(then_bl.expr), Box::new(else_bl.expr)),
                    is_used: true
                }))
            }
            Expr::Extern(b) => {
                //add this into pseudo function stack (used by callee)

                self.extern_function_stack.insert(b.clone().name, (b,false));

                Ok(
                    Some(FAST {
                        expr:e,
                        is_used: false
                    })
                )
            }
            Expr::Macro(name, expr) => {
                match name.as_str() {
                    "define" => {
                        let name = expr[0].ident_to_string();
                        let expr_slide = self.check_ast(
                            Vec::from(&expr[1..])
                        )?;
                        self.macro_define.insert(
                            name,
                            expr_slide
                        );
                    }
                    o => unimplemented!("{:?}", o)
                }
                Ok(None)
            }
            Expr::Class(name, func) => {
                let mut func_stmt = Vec::new();
                for x in func {
                    func_stmt.push(
                        ClassFunction {
                            function:
                                self.visit(x.function)?
                                .unwrap_or(FAST { expr: Expr::None, is_used:false })
                                .expr,
                            func_type: x.func_type,
                            access_level: x.access_level
                        }
                    );
                }
                Ok(
                    Some(
                        FAST {
                            expr: Expr::Class(name, func_stmt),
                            is_used: true
                        }
                    )
                )
            },
            o => todo!("Expression {:?} does not implemented yet!", o)
        }
    }

    fn check_ast(&mut self, ast: Vec<Expr>) -> Result<Vec<Expr>, String> {
        let mut res = Vec::new();
        // map and remove 'None' Option value
        let mut original_fast = ast.iter()
            .map(|f| catch_error(self.visit(f.clone())))
            .filter(|f| f.is_some())
            .map(|f| f.unwrap())
            .collect::<Vec<FAST>>();

        for func_f in &self.pseudo_function_stack {
            original_fast.iter().find(|f| {
                if matches!(f.expr, Expr::FuncStmt(_, _)) {
                    f.expr.get_function().0 == *func_f.0
                } else {
                    false
                }
            }).replace(&func_f.1.clone().unwrap_or(FAST {expr: Expr::None, is_used: false}));
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

        for (name, func_header) in self.extern_function_stack.iter() {
            if let Some(idx) = original_fast.iter().position(|f| {
                if let Expr::Extern(f) = &f.expr {
                    f.name == *name && func_header.1
                } else {
                    false
                }
            }) {
                original_fast[idx].is_used = true;
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
