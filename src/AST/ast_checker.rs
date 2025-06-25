use super::expr_node::Expr;


#[derive(Debug)]
pub struct FAST<'a> { // AST formatter
    expr: &'a Expr,
    is_used: bool
}

pub struct Checker<'a> {
    ast: &'a Vec<Expr>
}


impl<'a> Checker<'a> {

    pub fn new(ast: &'a Vec<Expr>) -> Self {
        Self { ast: ast }
    }

    pub fn check(&mut self) -> Result<Vec<FAST>, String> {

        let mut res = Vec::new();


        self.ast.iter().for_each(|f| {
            let mut err=Ok(());
            match f {
                Expr::Macro(s, v) => {
                    match s.as_str() {
                        "syscall" | "def" => {},
                        _ => err=Err(format!("Invaild Macro name '{}'", s))
                    }
                    v.clone().iter_mut().for_each(|x| {
                        x.visit();
                    });
                },
                Expr::VarDecl(d,_is_p ,_n ,init) => {
                    // suu a = 1.2;
                    ()
                }
                _ => {
                    ()
                }
            }
            if err.is_ok() {
                res.push(FAST { expr: f, is_used: false });
            }
        });

        Ok(res)
    }

}
