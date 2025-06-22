use super::expr_node::Expr;


pub struct Checker<'a> {
    ast: &'a Vec<Expr>
}


impl<'a> Checker<'a> {

    pub fn new(ast: &'a Vec<Expr>) -> Self {
        Self { ast: ast }
    }

    pub fn check(&mut self) -> Result<(), String> {

        let mut res = Ok(());

        self.ast.iter().for_each(|f| {
            match f {
                Expr::Macro(s, v) => {
                    match s.as_str() {
                        "syscall" | "def" => {},
                        _ => res = Err(format!("Invaild Macro name '{}'", s))
                    }
                    v.clone().iter_mut().for_each(|x| {
                        x.visit();
                    });
                },
                _ => {
                    ()
                }
            }
        });

        res
    }

}
