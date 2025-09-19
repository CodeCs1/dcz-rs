use std::{collections::VecDeque, process::exit};

use crate::{token::{token_type::TokenType, MetaData, TokenData}, MessageHandler::message_handler::throw_message, Value::Value, AST::expr_node::{DataType, Func_Header}};
pub mod expr_node;
pub mod ast_checker;
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


macro_rules! check_keyword {
    ($self:ident, $keyword: expr, $func: expr) => {
        if $self.peek().identifier == $keyword {
            $self.advance();
            return $func;
        }
    };
}


pub struct AST {
    filename: String,
    token: Vec<TokenData>,
    meta_data: MetaData,
    current: usize,
}

impl AST {
    pub fn new(meta_data: MetaData) -> Self {
        Self { token: meta_data.clone().tok_data, current:0, filename: meta_data.clone().filename, meta_data: meta_data.clone() }

    }
    
    fn is_eof(&self) -> bool {
        self.current >= self.token.len() || self.token[self.current].tok_type == TokenType::EOF 
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
        if self.match_token(&mut vec![TokenType::Number, TokenType::String, TokenType::Char]) {
            return Box::new(Expr::Literal(self.previous().value));
        }
        if self.match_token(&mut vec![TokenType::LeftParen]) {
            let expr = self.expr();
            self.consume(TokenType::RightParen, "Expect ')'");
            return Box::new(Expr::Grouping(expr));
        }

        if self.match_token(&mut vec![TokenType::Keywords, TokenType::DataType]) {
            return Box::new(Expr::Identifier(self.previous().identifier));
        }

        if self.match_token(&mut vec![TokenType::LeftBracket]) {
            return self.list();
        }

        if self.match_token(&mut vec![TokenType::Identifier]) {
            return Box::new(Expr::Var(self.previous().identifier));
        }

        panic!("Expect Expression at {}:{} ({:?})", self.peek().line, self.peek().end, self.peek().tok_type);
    }

    fn callee(&mut self) -> Box<Expr> {
        let mut primary = self.primary();
        if self.match_token(&mut vec![TokenType::LeftParen]) {
            let mut arg_v = Vec::new();
            while !self.check(TokenType::RightParen) {
                arg_v.push(*self.expr());
                if !self.check(TokenType::RightParen) {
                    self.consume(TokenType::Comma, "Expect ',' in parameter declare");
                }
            }
            self.consume(TokenType::RightParen, "Expect ')' after callee");
            primary = Box::new(Expr::Callee(primary, arg_v));
        }
        primary
    }

    fn unary(&mut self) -> Box<Expr> {
        if self.match_token(&mut vec![TokenType::Not, TokenType::Minus]) {
            let op = self.previous();
            let expr = self.unary();
            return Box::new(Expr::Unary(op, expr));
        }
        self.callee()
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
        self.assignment()
    }

    fn assignment(&mut self) -> Box<Expr> {
        let expr = self.bool_logical();

        if self.match_token(&mut vec![TokenType::Equal]) {
            let v = self.assignment();

            if matches!(*expr, Expr::Var(_)) {
                let n = expr.ident_to_string();
                return Box::new(Expr::Assign(n, v));
            }
            panic!("Invaild assignment object");
        }
        return expr;
    }

    fn while_stmt(&mut self) -> Box<Expr> {
        let expr = self.expr();
        let body = self.statement();
        return Box::new(Expr::WhileStmt(expr, body));
    }

    fn func_header(&mut self) -> (Box<Expr>, Vec<(DataType,String,bool)>, (bool, Option<DataType>)){
        let func_name = self.primary();

        self.consume(TokenType::LeftParen, "Expect '(' in declare func");
        let mut arg_v = Vec::new();

        while !self.check(TokenType::RightParen) {
            let dt = self.primary().to_datatype().expect("Expect data type - FuncStmt");
            let is_ptr = self.match_token(&mut vec![TokenType::Star]);
            let name = self.primary().ident_to_string();
            arg_v.push((dt, name, is_ptr));
            if !self.check(TokenType::RightParen) {
                self.consume(TokenType::Comma, "Expect ',' in arguments declare");
            }
        }

        self.consume(TokenType::RightParen, "Expect ')' in declare func");

        let return_type = if self.match_token(&mut vec![TokenType::PointTo]) {
            Some(self.primary().to_datatype().expect("Invaild data type"))
        } else {
            None
        };

        let is_ptr = self.match_token(&mut vec![TokenType::Star]);

        (func_name, arg_v, (is_ptr,return_type))
    }

    fn func_stmt(&mut self) -> Box<Expr> {

        /*
         * func test(suu test_args) -> suu {
         *  return 4;
         * }
         *
         * func test_ptr(suu test_args) -> suu* {
         *  return (suu*)0x123;
         * }
         * */
        let func_header = self.func_header();
        
        self.consume(TokenType::LeftBrace, "Expect '{' in declare func");
        let body = self.block();

        Box::new(
            Expr::FuncStmt(
                Func_Header { 
                    name: func_header.0.ident_to_string(), 
                    args: func_header.1,
                    return_type: func_header.2.1,
                    is_ptr_dt: func_header.2.0
                },
                body
            )
        )
    }

    fn list(&mut self) -> Box<Expr> {
        let mut data_type = DataType::Unknown;
        let mut l: Vec<Value> = Vec::new();
        while !self.check(TokenType::RightBracket) {
            let v = self.primary();
            l.push(v.to_value());
            if l.len() == 1 {
                data_type=v.to_value().to_datatype();
            }else {
                if v.to_value().to_datatype() != data_type {
                    let p = self.peek();
                    throw_message(
                        &self.filename, 
                        crate::MessageHandler::message_handler::MessageType::Error,
                        p.line as i64, p.start as i64, &format!("List item must be same as {:?}", data_type));
                    exit(1);
                }
            }
            if !self.check(TokenType::RightBracket) {
                self.consume(TokenType::Comma, "Expect ',' in list item declaration");
            }
        }
        self.consume(TokenType::RightBracket, "Expect ']' in list declaration");

        Box::new (
            Expr::List(l)
        )
    }

    fn statement(&mut self) -> Box<Expr> {
        
        if self.match_token(&mut vec![TokenType::LeftBrace]) {
            return self.block();
        }
        check_keyword!(self, "if",self.if_stmt());
        check_keyword!(self, "while", self.while_stmt());
        check_keyword!(self, "func", self.func_stmt());
        check_keyword!(self, "extern", self.extern_func());
        check_keyword!(self, "return", self.return_keyw());

        let expr = self.var_decl();
        if ! matches!(*expr, Expr::None) {
            self.consume(TokenType::Semicolon, "Expect semicolon");
            Box::new(
                Expr::Statement(expr)
            )
        } else {
            Box::new(Expr::None)
        }
    }

    fn return_keyw(&mut self) -> Box<Expr> {
        // return 3;
        let mut v = None;
        if !self.check(TokenType::Semicolon) {
            v = Some(self.expr());
        }

        self.consume(TokenType::Semicolon, "Expect ';' after return keyw");
        Box::new(

            Expr::Return(v)

            )
    }

    fn extern_func(&mut self) -> Box<Expr> {
        //extern <func_header>;

        if self.advance().identifier != "func" {
            throw_message(&self.filename,
                crate::MessageHandler::message_handler::MessageType::Error,
                self.peek().line as i64, self.peek().start as i64, 
            "extern declare must be start with 'func' keywords");
            exit(1);
        }

        let func_header = self.func_header();

        self.consume(TokenType::Semicolon, "Expect ';' after extern function");
        
        Box::new (
            Expr::Extern(Func_Header {
                name: func_header.0.ident_to_string(),
                args: func_header.1,
                return_type: func_header.2.1,
                is_ptr_dt: func_header.2.0
            })
        )
    }


    fn if_stmt(&mut self) -> Box<Expr> {
        let condition = self.expr();
        let then_block = self.statement();
        let mut else_block = Box::new(Expr::None);
        if self.peek().identifier == "else" {
            self.advance();
            else_block = self.statement();
        }
        Box::new(
            Expr::IfStmt(condition, then_block, else_block)
            )
    }

    fn block(&mut self) -> Box<Expr> {
        /*
         * {
         *  int a = 0;
         *
         * }
         * */

        let mut block = Vec::new();
        while ! self.check(TokenType::RightBrace) && !self.is_eof() {
            let st = *self.statement();
            block.push(st);
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block declare");
        Box::new(
            Expr::Block(block)
        )
    }


    fn var_decl(&mut self) -> Box<Expr> {
        // char* a = "hello world";
        // let a: const = 3;
        
        //check if current token is not data type
        

        if self.peek().tok_type != TokenType::DataType 
        && self.peek().identifier != "let" 
        && self.peek().identifier != "const" {
            return self.expr();
        }

        let is_const = self.peek().identifier=="const";
        
        let mut data_type = if self.peek().tok_type == TokenType::DataType {
            self.primary().to_datatype().expect("VarDecl")
        } else {
            self.advance();
            DataType::Unknown
        };
        let mut is_pointer = self.match_token(&mut vec![TokenType::Star]);

        let name = self.primary();
        if !matches!(*name, Expr::Var(_)) {
            let l =self.peek().line;
            let p =self.peek().start;
            throw_message(
                "stdin", 
                crate::MessageHandler::message_handler::MessageType::Error,
                l as i64, p as i64,"Using keyword as variable name is forbidden!");
            exit(1);
        }

        if self.peek().tok_type == TokenType::Colon {
            if !matches!(data_type, DataType::Unknown) {
                let l =self.peek().line;
                let p =self.peek().start;
                throw_message(
                    &self.filename, 
                    crate::MessageHandler::message_handler::MessageType::Error,
                    l as i64, p as i64,&format!("Can't override to data type: {:?}\nFix this by using 'let' instead.", data_type));
                exit(1);
            }
            self.advance();
            data_type = self.primary().to_datatype().expect("VarDecl(override)");
            is_pointer = self.match_token(&mut vec![TokenType::Star]);
        }

        let mut init = None;

        if self.match_token(&mut vec![TokenType::Equal]) {
            let i = self.expr();
            if matches!(data_type, DataType::Unknown) && matches!(*i, Expr::Literal(_)) {
                let v = i.to_value();
                if !v.clone().is_null() {
                    data_type = if v.clone().is_char() { DataType::Char }
                            else if v.clone().is_double() {DataType::Suu}
                            else if v.clone().is_float() {DataType::Float}
                            else if v.clone().is_literal() {DataType::Long}
                            else {DataType::Unknown}
                }
            }
            init = Some(i);
        }
        Box::new(
            Expr::VarDecl(data_type, is_pointer,is_const, name.ident_to_string(), init)
            )
    }


    pub fn parse(&mut self) -> Vec<Expr> {
        let mut expr_vec: Vec<Expr> = Vec::new();
        
        while !self.is_eof() {
            if self.match_token(&mut vec![TokenType::Macro]) {
                let mut vect: Vec<Expr> = Vec::new();
                if let Some(st) = self.previous().sub_tok {
                    let mut macro_queue = VecDeque::from(st);

                    let macro_name = macro_queue.pop_front().unwrap();
                    let mut sub_ast = AST::new(MetaData {filename: self.filename.clone(), tok_data: Vec::from(macro_queue), data: self.meta_data.data.clone()});
                    while !sub_ast.is_eof() {
                        vect.push(*sub_ast.expr());
                    }
                    expr_vec.push(Expr::Macro(macro_name.identifier, vect));
                }
            }
            else {
                
                let expr = *self.statement();
                match expr {
                    Expr::None => {}
                    _ => expr_vec.push(expr)
                }
            }
        }

        expr_vec
    }
}
