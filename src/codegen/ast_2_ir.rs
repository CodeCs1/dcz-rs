use std::sync::{atomic::AtomicBool, Arc};

use crate::AST::expr_node::Expr;

use super::{ir::{Ir, IrBuilder}, ir_opcode::{ConstantPool, Opcode}};

pub struct Ast2Ir {
    expr: Vec<Expr>,
    pub const_pool: ConstantPool
}


fn visit_expr(e: Expr) ->Vec<Opcode> {
    static IN_BLOCK: AtomicBool = AtomicBool::new(false);
    match e {
        Expr::Statement(st) => {
            visit_expr(*st)
        }
        Expr::Grouping(expr) => {
            visit_expr(*expr)
        }
        Expr::Binary(lhs,op ,rhs) => {
            let lhs_op = visit_expr(*lhs);
            let mut rhs_op = visit_expr(*rhs);

            let mut v=Vec::from(lhs_op);
            v.append(&mut rhs_op);
            v.push(match op.tok_type {
                crate::token::token_type::TokenType::Plus => Opcode::Add,
                crate::token::token_type::TokenType::Minus => Opcode::Sub,
                crate::token::token_type::TokenType::Star => Opcode::Mul,
                crate::token::token_type::TokenType::Slash => Opcode::Div,
                _ => unimplemented!()
            });
            v
        }
        Expr::VarDecl(_, is_p, s, init) => {
            if is_p {
                unimplemented!("Pointer declare not yet implemented.");
            }
            let mut v = vec![Opcode::Nop];
            if init.is_some() {
                v = visit_expr(*init.unwrap());
            }
            let mut v = Vec::from(v);
            if !IN_BLOCK.load(std::sync::atomic::Ordering::Relaxed) {
                v.push(Opcode::StoreName(s));
            } else {
                v.push(Opcode::StoreLocal(s));
            }
            v
        },
        Expr::Identifier(n) => {
            vec![Opcode::LoadName(n)]
        }
        Expr::Block(bl) => {

            IN_BLOCK.store(true, std::sync::atomic::Ordering::SeqCst);
            let mut v = Vec::new();
            v.push(Opcode::Begin);
            bl.iter().for_each(|f| { v.append(&mut visit_expr(f.clone()) ); });
            IN_BLOCK.store(false, std::sync::atomic::Ordering::SeqCst);

            v.push(Opcode::ClearLocal);
            v
        }
        Expr::Unary(op, rhs) => {
            let rhs_op = visit_expr(*rhs);
            let mut v = Vec::from(rhs_op);

            v.push(match op.tok_type {
                crate::token::token_type::TokenType::Minus => Opcode::Neg,
                crate::token::token_type::TokenType::Not => Opcode::Not,
                _ => unimplemented!()
            });
            v
        }
        Expr::Literal(v) => vec![Opcode::Constant(v)],
        e => todo!("This expr '{:?}' does not implemented yet.", e)
    }
}
impl Ast2Ir {
    pub fn new(vect: Vec<Expr>) -> Self{
        Ast2Ir { expr: vect, const_pool: ConstantPool::new() }
    }

    pub fn to_ir(&mut self) -> Ir {
        let mut irb = IrBuilder::new();

        for x in self.expr.iter_mut() {
            irb=irb.append_from_vec(&mut visit_expr(x.clone()));
        }
        self.const_pool = irb.get_const_pool();

        irb.clone().build()
    }
}
