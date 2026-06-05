/*
    This AST Checker one will do:
     + passes on some basic optimization including:
        Constant folding
        Unused variable and extern function (This known as deadcode elimination)
        Limit some data type
     + Basic type checking
*/

use super::expr_node::{DataType, Expr, Variable};
use crate::AST::VarEnvironment::VarEnvironment;
use crate::AST::expr_node::{ClassFunction, FuncHeader, VariableType};
use crate::MessageHandler::{MessageType, throw_message};
use crate::{Value::Value, panic_error};
use std::collections::{BTreeMap, HashMap};
use std::process::exit;

// macro_rules! check_overflow {
//     ($dt: ident, $vi64: ident, $s: ident) => {
//         if $vi64 > $dt::MAX as f64 {
//             throw_message(format!("{}", $s).as_str(), MessageType::Warning, 1, 0, format!("datatype overflow, rolling back from {} to {}",
//             $vi64, $vi64%$dt::MAX as f64).as_str());
//         }
//     };
// }

#[derive(Debug, Clone, PartialEq)]
pub struct FAST {
    // AST formatter
    pub expr: Expr,
    pub is_used: bool,
}

pub struct Checker<'a> {
    ast: &'a Vec<Expr>,
    pseudo_variable_stack: VarEnvironment,
    pseudo_function_stack: BTreeMap<String, Option<FAST>>,
    /// Hashmap (Function Header, is Used)
    extern_function_stack: BTreeMap<String, (FuncHeader, bool)>,
    macro_define: HashMap<String, Vec<Expr>>,
    filename: String,
    parent_function: Option<FuncHeader>,
    is_return: bool,
}

fn check_literal_type(
    init: Option<Box<Expr>>,
    vt: VariableType,
    _filename: String,
) -> Result<(Option<Box<Expr>>, VariableType), String> {
    let mut init_v = None;

    let var_type = vt;
    if matches!(var_type, VariableType::NonPointer(DataType::Void))
        || matches!(var_type, VariableType::Constant(DataType::Void))
    {
        return Err(format!(
            "'void' cannot be used like normal datatype\nUse different data type or add '*' at the end of 'void' data type."
        ));
    }

    if let Some(mut v) = init {
        let mut v = v.visit()?;

        if !matches!(v, Expr::Literal(_)) || !var_type.clone().is_pointer_of(DataType::Void) {
            return Ok((Some(Box::new(v)), var_type));
        }

        let to_v = v.to_value();

        if to_v.clone().is_string() {
            if !matches!(var_type, VariableType::Pointer(_)) {
                return Err(format!(
                    "Cannot convert from {:?} to string literal (variable MUST be pointer and data type MUST be char)",
                    var_type.clone().get_datatype()
                ));
            }
            if !var_type.clone().is_pointer_of(DataType::I8)
                && !var_type.clone().is_pointer_of(DataType::U8)
            {
                return Err(format!(
                    "Cannot convert from {:?}* to string literal (data type MUST be char)",
                    var_type.clone().get_datatype()
                ));
            }
        }

        if var_type.clone().is_pointer() {
            return Ok((Some(Box::new(v)), var_type.clone()));
        }

        if let Expr::Literal(mut val) = v {
            val.cast(var_type.clone().get_datatype())?;
            v = Expr::Literal(val);
        }

        init_v = Some(Box::new(v));
    }
    Ok((init_v, var_type))
}

pub fn catch_error(r: Result<Option<FAST>, String>, filename: String) -> Option<FAST> {
    match r {
        Ok(f) => f,
        Err(s) => {
            panic_error!(&filename, 1, 0, &s);
        }
    }
}

impl<'a> Checker<'a> {
    pub fn new(ast: &'a Vec<Expr>, file: String) -> Self {
        Self {
            ast: ast,
            pseudo_variable_stack: VarEnvironment::new(),
            pseudo_function_stack: BTreeMap::new(),
            extern_function_stack: BTreeMap::new(),
            macro_define: HashMap::new(),
            filename: file,
            parent_function: None,
            is_return: false,
        }
    }

    fn visit(&mut self, expr: Expr) -> Result<Option<FAST>, String> {
        let e = expr.clone();
        match expr {
            Expr::Statement(e) => self.visit(*e),
            Expr::Return(v) => {
                self.is_return = true;
                let ret = if let Some(ret_v) = v {
                    self.visit(*ret_v)?
                } else {
                    None
                };
                self.is_return = false;
                Ok(Some(FAST {
                    expr: Expr::Return({
                        if ret.is_none() {
                            None
                        } else {
                            Some(Box::new(ret.unwrap().expr))
                        }
                    }),
                    is_used: true,
                }))
            }
            Expr::Callee(n, args) => {
                let name = n.ident_to_string();
                if !self.pseudo_function_stack.contains_key(&name) {
                    if !self.extern_function_stack.contains_key(&name) {
                        return Err(format!("Function '{}' not declared!", name));
                    } else {
                        // set used extern function to true
                        if let Some(extern_func) = self.extern_function_stack.get_mut(&name) {
                            *extern_func = (extern_func.0.clone(), true)
                        }
                    }
                }

                let args = self.check_ast(args)?;

                Ok(Some(FAST {
                    expr: Expr::Callee(n, args),
                    is_used: true,
                }))
            }
            Expr::Var(n) => {
                if let Ok(r) = self.pseudo_variable_stack.get_mut(&n) {
                    r.is_used = true;
                    Ok(
                        Some(
                            FAST { 
                                expr: r.init.as_ref().unwrap().clone(),
                                is_used: true
                            }
                        )
                    )
                } else {
                    // maybe it's in macro define hashmap
                    if let Some(idx) = self.macro_define.iter().find(|f| f.0 == n.as_str()) {
                        Ok(Some(FAST {
                            expr: idx.1[0].clone(),
                            is_used: true,
                        }))
                    } else {
                        // yea, just give up already =P
                        Err(format!("Variable '{}' not declared!", n))
                    }
                }
            }
            Expr::WhileStmt(condition, expr) => {
                let cond = self.visit(*condition)?.unwrap();
                let expr = self.visit(*expr)?.unwrap();
                Ok(Some(FAST {
                    expr: Expr::WhileStmt(Box::new(cond.expr), Box::new(expr.expr)),
                    is_used: true,
                }))
            }

            Expr::VarDecl(vt, n, init) => {
                let (init_v, mut data_type) = match check_literal_type(init, vt, self.filename.clone())
                {
                    Ok(v) => (v.0, v.1),
                    Err(s) => {
                        panic_error!(&self.filename, 1, 1, s.as_str());
                    }
                };

                if self
                    .pseudo_variable_stack.contains(&n)
                {
                    return Err(format!("Variable '{}' already defined", n));
                }

                let is_init = match init_v {
                    Some(v) => {
                        let k = self.visit(v.as_ref().clone())?.unwrap();
                        if let Expr::Var(v) = &k.expr {
                            data_type = self.pseudo_variable_stack[v].vData.clone();
                        }

                        let variable = Variable::new(
                            data_type.clone(),
                            n.clone(),
                            Some(k.clone().expr),
                            false
                        );
                        self.pseudo_variable_stack.new_value(n.clone(),variable.clone());
                        data_type = variable.vData;
                        Some(Box::new(k.clone().expr))
                    },
                    None => None
                };
                
                Ok(Some(FAST {
                    expr: Expr::VarDecl(data_type, n, is_init),
                    is_used: false,
                }))
            }

            Expr::Assign(n, v) => {
                let Ok(var) = self.pseudo_variable_stack.get(&n) else {
                    return Err(format!("Undefined variable {}", n));
                };
                if var.clone().is_constant() {
                    return Err(format!("Constant variable '{}' cannot be assignable!", n));
                }
                let val = check_literal_type(Some(v), var.clone().vData, self.filename.clone())?;
                
                Ok(Some(FAST {
                    expr: Expr::Assign(n, val.0.unwrap()),
                    is_used: true,
                }))
            }

            Expr::Literal(mut v) => {
                if self.is_return {
                    let r = self.parent_function.clone().unwrap();
                    let dt = r.return_type.unwrap();
                    v.cast(dt.get_datatype())?;
                }
                Ok(Some(FAST {
                    expr: Expr::Literal(v),
                    is_used: true,
                }))
            }
            Expr::FuncStmt(f, body) => {
                //self.visit(*b)
                //add to pseudo_variable_stack

                self.pseudo_function_stack.insert(f.name.clone(), None);

                //temporary add args into variable stack
                let mut env = VarEnvironment::new_with_enclosing(self.pseudo_variable_stack.clone());
                for arg in f.args.iter() {
                    env.new_value(arg.clone().name,arg.clone());
                }
                self.parent_function = Some(f.clone());
                self.pseudo_variable_stack.save();
                self.pseudo_variable_stack.load(env);
                let fast = FAST {
                    expr: Expr::FuncStmt(f.clone(), Box::new(self.visit(*body)?.unwrap().expr)),
                    is_used: true,
                };
                if let Some(f) = self.pseudo_function_stack.get_mut(&(f.name)) {
                    *f = Some(fast.clone());
                }

                //and then remove it from variable stack
                self.pseudo_variable_stack.load_last();
                self.parent_function = None;

                Ok(Some(fast))
            }
            Expr::Block(b) => {
                let env = VarEnvironment::new_with_enclosing(
                    self.pseudo_variable_stack.clone()
                );
                self.pseudo_variable_stack.save();
                self.pseudo_variable_stack.load(env);
                let bl = self.check_ast(b)?;
                self.pseudo_variable_stack.load_last();
                Ok(Some(FAST {
                    expr: Expr::Block(bl),
                    is_used: true,
                }))
            }
            Expr::Binary(mut lhs, op, mut rhs) => {
                let lhs = self.visit(lhs.visit()?)?.unwrap();
                let rhs = self.visit(rhs.visit()?)?.unwrap();

                let e = Expr::Binary(Box::new(lhs.expr), op, Box::new(rhs.expr)).visit()?;

                Ok(Some(FAST {
                    expr: e,
                    is_used: true,
                }))
            }
            Expr::IfStmt(cond, then_bl, else_bl) => {
                let cond = self.visit(*cond)?.unwrap();
                if let Expr::Literal(cond_val) = cond.expr {
                    if cond_val.val == Value::Integer(0) {
                        if matches!(*else_bl, Expr::None) {
                            return Ok(Some(FAST {
                                expr: Expr::None,
                                is_used: false,
                            }));
                        } else {
                            return Ok(Some(FAST {
                                expr: *else_bl,
                                is_used: true,
                            }));
                        }
                    } else {
                        return Ok(Some(FAST {
                            expr: *then_bl,
                            is_used: true,
                        }));
                    }
                }
                let then_bl = self.visit(*then_bl)?.unwrap();
                let else_bl = if !matches!(*else_bl, Expr::None) {
                    self.visit(*else_bl)?.unwrap()
                } else {
                    FAST {
                        expr: Expr::None,
                        is_used: false,
                    }
                };
                Ok(Some(FAST {
                    expr: Expr::IfStmt(
                        Box::new(cond.expr),
                        Box::new(then_bl.expr),
                        Box::new(else_bl.expr),
                    ),
                    is_used: true,
                }))
            }
            Expr::Extern(b) => {
                //add this into pseudo function stack (used by callee)

                self.extern_function_stack
                    .insert(b.clone().name, (b, false));

                Ok(Some(FAST {
                    expr: e,
                    is_used: false,
                }))
            }
            Expr::Macro(name, expr) => {
                match name.as_str() {
                    "define" => {
                        let name = expr[0].ident_to_string();
                        let expr_slide = self.check_ast(Vec::from(&expr[1..]))?;
                        self.macro_define.insert(name, expr_slide);
                    }
                    o => unimplemented!("{:?}", o),
                }
                Ok(None)
            }
            Expr::Class(name, func) => {
                let mut func_stmt = Vec::new();
                for x in func {
                    func_stmt.push(ClassFunction {
                        function: self
                            .visit(x.function)?
                            .unwrap_or(FAST {
                                expr: Expr::None,
                                is_used: false,
                            })
                            .expr,
                        func_type: x.func_type,
                        access_level: x.access_level,
                    });
                }
                Ok(Some(FAST {
                    expr: Expr::Class(name, func_stmt),
                    is_used: true,
                }))
            }
            Expr::Cast(new_datatype, mut expr) => {
                let mut new_expr = expr.visit()?;
                if let Expr::Literal(mut value) = new_expr {
                    value.cast(new_datatype.clone().get_datatype())?;
                    new_expr = Expr::Literal(value);
                    return Ok(Some(FAST {
                        expr: new_expr,
                        is_used: true,
                    }));
                }
                Ok(Some(FAST {
                    expr: Expr::Cast(new_datatype, Box::new(new_expr)),
                    is_used: true,
                }))
            }
            o => todo!("Expression {:?} does not implemented yet!", o),
        }
    }

    fn check_ast(&mut self, ast: Vec<Expr>) -> Result<Vec<Expr>, String> {
        let mut res = Vec::new();
        let filename = &self.filename.clone();
        // map and remove 'None' Option value
        let mut original_fast = ast
            .iter()
            .map(|f| catch_error(self.visit(f.clone()), filename.to_string()))
            .filter(|f| f.is_some())
            .map(|f| f.unwrap())
            .collect::<Vec<FAST>>();

        for func_f in &self.pseudo_function_stack {
            original_fast
                .iter()
                .find(|f| {
                    if matches!(f.expr, Expr::FuncStmt(_, _)) {
                        f.expr.get_function().0 == *func_f.0
                    } else {
                        false
                    }
                })
                .replace(&func_f.1.clone().unwrap_or(FAST {
                    expr: Expr::None,
                    is_used: false,
                }));
        }

        println!("{:#?}",self.pseudo_variable_stack);

        /*
        for var_decl in &self.pseudo_variable_stack {
            if let Some(idx) = original_fast.iter().position(|f| {
                if let Expr::VarDecl(_, name, _) = &f.expr {
                    *name == var_decl.name && var_decl.is_used
                } else {
                    false
                }
            }) {
                original_fast[idx].is_used = true;
            }
        }*/

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
            &mut original_fast
                .into_iter()
                .filter(|f| f.is_used)
                .map(|f| f.expr)
                .collect(),
        );

        Ok(res)
    }

    pub fn check(&mut self) -> Result<Vec<Expr>, String> {
        self.check_ast(self.ast.to_vec())
    }
}
