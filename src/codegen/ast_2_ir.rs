use std::sync::atomic::AtomicBool;

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
            v.push(Opcode::BinOp(op));
            v
        }
        Expr::VarDecl(data_type, is_p,_, s, init) => {
            let mut v = Vec::new();
            if init.is_some() {
                v = visit_expr(*init.unwrap());
            }
            let mut v = Vec::from(v);
            if !IN_BLOCK.load(std::sync::atomic::Ordering::Relaxed) {
                v.push(Opcode::StoreGlobal(data_type,is_p,s));
            } else {
                v.push(Opcode::StoreLocal(data_type,is_p,s));
            }
            v
        },
        Expr::Var(n) => {
            vec![Opcode::LoadName(n)]
        }
        Expr::Block(bl) => {

            IN_BLOCK.store(true, std::sync::atomic::Ordering::SeqCst);
            let mut v = Vec::new();
            v.push(Opcode::Begin);
            bl.iter().for_each(|f| { v.append(&mut visit_expr(f.clone()) ); });
            IN_BLOCK.store(false, std::sync::atomic::Ordering::SeqCst);

            v.push(Opcode::End);
            v
        }
        Expr::IfStmt(cond,then , elsecase) => {
            let mut v = Vec::from(visit_expr(*cond));
            let mut then_v = visit_expr(*then);
            let mut else_v = Vec::new();
            if !matches!(*elsecase, Expr::None) {
                else_v.append(&mut visit_expr(*elsecase));
                then_v.push(Opcode::Jmp(else_v.len()));
            }
            v.push(Opcode::JIfFalse(then_v.len()));
            v.append(&mut then_v);
            v.append(&mut else_v);
            v
        }
        Expr::Assign(n, v) => {
            let mut v = Vec::from(visit_expr(*v));
            v.push(Opcode::Agn(n));
            v
        },
        Expr::WhileStmt(cond, body) => {
            let mut v = Vec::from(visit_expr(*cond));
            let cond_len = v.len();
            if cond_len == 1 {
                let mut body = visit_expr(*body);
                let body_len = body.len();
                v.append(&mut body);
                v.push(Opcode::JBackward(body_len));
            } else {
                let mut body = visit_expr(*body);
                let body_len = body.len();
                v.push(Opcode::JIfFalse(body_len+1));
                v.append(&mut body);
                v.push(Opcode::JBackward(body_len+cond_len+2));
            }

            v
        },
        Expr::FuncStmt(n, args , body,r_d) => {
            let mut v = Vec::new();
            let expr = visit_expr(*body);

            v.push(Opcode::MakeFunc(expr.len(),n));

            args.iter().for_each(|(d,n)| {
                v.push(Opcode::StoreParam(d.clone(), n.clone()))
            });

            v.append(&mut expr.clone());

            let ret_last = &expr[expr.len()-2];

            if r_d.is_some() {
                if !matches!(ret_last, Opcode::Return(_)) {
                    v.push(Opcode::Invaild)
                }
            } else {
                if matches!(ret_last, Opcode::Return(_)) {
                    v.push(Opcode::Invaild)
                }
            }

            v.push(Opcode::EndFunc);
            //v.push(Opcode::StoreName(n));
            v
        },
        Expr::Callee(n, args) => {
            let mut v = Vec::new();
            for x in &args {
                v.push(Opcode::StoreArg(x.to_value()));
            }
            v.push(Opcode::Call(n.ident_to_string()));
            v
        },
        Expr::Return(val_ret) => {
            if val_ret.is_some() {
                return vec![Opcode::Return(Some(val_ret.unwrap().to_value()))];
            }

            vec![Opcode::Return(None)]

            
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
impl Ast2Ir  {
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
