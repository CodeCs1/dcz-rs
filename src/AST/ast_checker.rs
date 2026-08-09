/*
    This AST Checker one will do:
     + passes on some basic optimization including:
        Constant folding
        Unused variable and extern function (This known as deadcode elimination)
        Limit some data type
     + Basic type checking
*/

use super::expr_node::{DataType, ExprType,Expr, Variable};
use crate::AST::VarEnvironment::VariableSymbolTableRoot as VariableEnvironment;
use crate::AST::expr_node::{ClassFunction, FuncHeader, VariableType};
use crate::MessageHandler::MessageType::Warning;
use crate::MessageHandler::{MessageType, throw_message};
use crate::token::token_type::TokenType;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, PartialEq)]
pub struct FAST {
    // AST formatter
    pub expr: Expr,
    is_used: bool,
}

impl FAST {
    pub fn new_empty_unused() -> Self {
        Self {
            expr: Expr::default(),
            is_used: false
        }
    }
    pub fn new(expr: Expr, used: bool) -> Self {
        Self {
            expr: expr,
            is_used: used
        }
    }
    pub fn set_new_expr(mut self, expr: Expr) -> Self {
        self.expr = expr;
        self
    }
    pub fn used(mut self) -> Self {
        self.is_used = true;
        self
    }
    pub fn unused(mut self) -> Self {
        self.is_used = false;
        self
    }
}

impl Default for FAST {
    fn default() -> Self {
        Self::new_empty_unused()
    }
}

pub struct Checker<'a> {
    expr_visitor: &'a Vec<Expr>,
    variables: VariableEnvironment<'a>,
    functions: BTreeMap<String, Option<FAST>>,
    /// Hashmap (Function Header, is Used)
    extern_function_stack: BTreeMap<String, (FuncHeader, bool)>,
    macro_define: HashMap<String, Vec<Expr>>,
    filename: String,
    parent_function: Option<FuncHeader>,
    is_return: bool,
}
impl<'a> Checker<'a> {
    pub fn new(ast: &'a Vec<Expr>, file: String) -> Self {
        Self {
            expr_visitor: ast,
            variables: VariableEnvironment::new(),
            functions: BTreeMap::new(),
            extern_function_stack: BTreeMap::new(),
            macro_define: HashMap::new(),
            filename: file,
            parent_function: None,
            is_return: false,
        }
    }

    fn visit(&mut self, expr: &Expr) -> Result<Option<FAST>, String> {
        let Some(expr_type) =  expr.to_owned().get_expr_type() else { return Ok(None) };

        match *expr_type {
            ExprType::Statement(child) => self.visit(&child),
            ExprType::Identifier(_) | ExprType::Literal(_) => Ok(Some(FAST::new(expr.clone(), true))),
            ExprType::Binary(lhs, op, rhs) => {
                let Some(lhs_new) = self.visit(&lhs)? else { panic!("Empty lhs"); };
                let Some(rhs_new) = self.visit(&rhs)? else { panic!("Empty rhs"); };

                let fast_binary = if let (Some(lhs_1),Some(rhs_1)) = (lhs_new.expr.literal(), rhs_new.expr.literal()) {
                    if lhs_1.is_ptr || rhs_1.is_ptr { todo!("Calulation on pointer not yet implemented") }
                    let new_value = match op.tok_type {
                        TokenType::Plus => lhs_1.val+rhs_1.val,
                        TokenType::Minus => lhs_1.val-rhs_1.val,
                        TokenType::Star => lhs_1.val*rhs_1.val,
                        TokenType::Slash => lhs_1.val/rhs_1.val,
                        e => unimplemented!("Token Type {:?} not implemented for math calculation!",e)
                    }?;
                    let new_typed_value = crate::Value::TypedValue::new(new_value,false);
                    Some(FAST::new(
                        Expr::new(ExprType::Literal(new_typed_value), expr.line, expr.at),true))
                } else {Some(FAST::new(expr.clone(),true))};
                Ok(fast_binary)
            }
            ExprType::Var(v_name) => {
                let Some(v) = self.variables.find(&v_name) else {
                    return Err(format!("Variable '{}' not found", v_name));
                };
                Ok(if v.clone().is_constant() {
                    Some(FAST::new(v.init.unwrap(), true))
                } else {
                    Some(FAST::new(expr.clone(), true))
                })
            }

            ExprType::Block(v) => {
                self.variables.add_local_table();
                let mut expr_block:Vec<Expr> = Vec::new();
                for expr in v.iter() {
                    if let Some(fast) = self.visit(expr)? {
                        expr_block.push(fast.expr);
                    }
                }
                self.variables.clear_local_table();
                Ok(Some(
                    FAST::new(Expr::new(ExprType::Block(expr_block), expr.line, expr.at),true)
                ))
            }
            ExprType::VarDecl(var) => {
                if self.variables.find(&var.name).is_some() {
                    return Err(format!("Variable '{}' already defined", var.name));
                }
                let vars = if let Some(init_value) = &var.init {
                    let optimize_init = self.visit(&init_value)?.unwrap().expr;
                    let mut var_new = var;
                    var_new.init = Some(optimize_init);
                    self.variables.add_vars(var_new.clone());
                    Expr::new(ExprType::VarDecl(
                        var_new
                    ), expr.line, expr.at)
                } else {
                    println!("{}",
                        throw_message(&self.filename, Warning, expr.line, expr.at,
                            format!("Variable {} is not initialized.", var.name)));
                    self.variables.add_vars(var);
                    expr.clone()
                };
                Ok(Some( FAST::new(vars, false) ))
            }
            e => Err(format!("Expr not support: {:?}", e))
        }
    }

    pub fn check(&mut self) -> Result<Vec<Expr>, String> {
        let mut exprs:Vec<Expr> = Vec::new();
        for expr in self.expr_visitor.iter() {
            if let Some(fast) = self.visit(expr)? {
                exprs.push(fast.expr);
            }
        }
        Ok(exprs)
    }
}
