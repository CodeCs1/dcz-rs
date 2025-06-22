use crate::token::{token_type::TokenType, TokenData};
pub mod expr_node;
pub mod ast_checker;
pub mod ir_expr;



use expr_node::Expr;

macro_rules! create_binary {
    ($self:ident, $name: ident, $lhs: expr, $tok_list: expr, $rhs: expr) => {
        fn $name(&mut $self) -> Box<Expr> {
            let mut lhs = $lhs;

            while $self.match_token(&mut $tok_list) {
                let op = $self.previous();
                let rhs = $rhs;
                lhs = Box::new(Expr::Binary(lhs, op, rhs));
            }

            lhs
        }
    };
}



pub struct AST {
    token: Vec<TokenData>,
    current: usize,
}

impl AST {
    pub fn new(token: Vec<TokenData>) -> Self {
        Self { token: token, current:0 }

    }
    
    fn is_eof(&self) -> bool {
        self.token[self.current].tok_type == TokenType::EOF
    }

    fn peek(&self) -> TokenData {
        self.token[self.current].clone()
    }

    fn previous(&self) -> TokenData {
        self.token[self.current-1].clone()
    }

    fn advance(&mut self) -> TokenData {
        if !self.is_eof() { self.current+=1; }
        self.previous()
    }

    fn check(&self, t: TokenType) -> bool {
        if self.is_eof() { return false; }
        self.peek().tok_type == t

    }

    fn match_token(&mut self, types: &mut Vec<TokenType>) -> bool {
        types.iter_mut().any(|f| {
            if self.check(f.clone()) {
                self.advance();
                true
            } else {
                false
            }
        })
    }

    fn consume(&mut self, tok_type: TokenType, message: &str) {
        if self.check(tok_type) { self.advance(); }
        else { 
            let p = self.peek();
            panic!("{} at {}:{}", message, p.line, p.end); 
        }
    }

    fn primary(&mut self) -> Box<Expr> {
        if self.match_token(&mut vec![TokenType::Number, TokenType::String]) {
            return Box::new(Expr::Literal(self.previous().value));
        }
        if self.match_token(&mut vec![TokenType::LeftParen]) {
            let expr = self.expr();
            self.consume(TokenType::RightParen, "Expect ')'");
            return Box::new(Expr::Grouping(expr));
        }

        if self.match_token(&mut vec![TokenType::Identifier, TokenType::Keywords, TokenType::DataType]) {
            return Box::new(Expr::Identifier(self.previous().identifier));
        }

        if self.match_token(&mut vec![TokenType::NewLine]) {
            return Box::new(Expr::None);
        }

        panic!("Expect Expression at {}:{}", self.peek().line, self.peek().end);
    }

    fn unary(&mut self) -> Box<Expr> {
        if self.match_token(&mut vec![TokenType::Not, TokenType::Minus]) {
            let op = self.previous();
            let expr = self.unary();
            return Box::new(Expr::Unary(op, expr));
        }
        self.primary()
    }
    
    // very rust
    create_binary!(self, factor, self.unary(), vec![TokenType::Star, TokenType::Slash], self.unary());
    create_binary!(self, term, self.factor(), vec![TokenType::Plus, TokenType::Minus], self.factor());
    create_binary!(self, compare, self.term(), vec![TokenType::Less, TokenType::LessEqual, TokenType::Greater, TokenType::GreaterEqual], self.term());
    create_binary!(self, shift, self.compare(), vec![TokenType::ShiftLeft, TokenType::ShiftRight], self.compare());
    create_binary!(self, equal, self.shift(), vec![TokenType::EqualEqual, TokenType::NotEqual], self.shift());
    create_binary!(self, logical, self.equal(), vec![TokenType::Or, TokenType::And], self.equal());
    create_binary!(self, bool_logical, self.logical(), vec![TokenType::OrBool, TokenType::AndBool], self.logical());

    fn expr(&mut self) -> Box<Expr> {
        self.bool_logical()
    }

    fn statement(&mut self) -> Box<Expr> {
        let expr = self.expr();
        if ! matches!(*expr, Expr::None) {
            self.consume(TokenType::Semicolon, "Expect semicolon");
            Box::new(
                Expr::Statement(expr)
            )
        } else {
            Box::new(Expr::None)
        }
    }


    fn var_decl(&mut self) -> Box<Expr> {
        // char* a = "hello world";
        
        //check if current token is not data type
        
        if self.peek().tok_type != TokenType::DataType {
            return self.statement();
        }
        let data_type = self.primary().to_datatype().expect("VarDecl");
        let is_pointer = self.match_token(&mut vec![TokenType::Star]);

        let name = self.primary();
        if !matches!(*name, Expr::Identifier(_)) {
            panic!("Invaild variable name");
        }

        let mut init = None;

        if self.match_token(&mut vec![TokenType::Equal]) {
            init = Some(self.expr());
        }

        self.consume(TokenType::Semicolon, "Expect ';'");

        Box::new(
            Expr::VarDecl(data_type, is_pointer, name.ident_to_string(), init)
            )
    }


    pub fn parse(&mut self) -> Vec<Expr> {
        let mut expr_vec: Vec<Expr> = Vec::new();
        
        while !self.is_eof() {

            if self.match_token(&mut vec![TokenType::Macro]) {
                let mut vect: Vec<Expr> = Vec::new();
                let macro_name = self.previous().identifier;
                while !self.check(TokenType::NewLine) && !self.is_eof() {
                    vect.push(*self.expr());
                }
                expr_vec.push(Expr::Macro(macro_name, vect));
            }
            else {
                let expr = *self.var_decl();
                match expr {
                    Expr::None => {}
                    _ => expr_vec.push(expr)
                }
            }
        }

        expr_vec
    }
}
