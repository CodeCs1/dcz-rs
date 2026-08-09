use std::collections::VecDeque;

use crate::{AST::expr_node::{AccessLevel, ClassFunction, ClassFunctionType, DataType, ExprType, FuncHeader, VariableType}, MessageHandler::{self, MessageType, throw_message}, Value::Value, panic_error, token::{MetaData, TokenData, token_type::TokenType}};
pub mod expr_node;
pub mod ast_checker;
pub mod VarEnvironment;
use expr_node::{Expr, Variable};

macro_rules! create_binary {
    ($self:ident, $name: ident, $lhs: expr, $tok_list: expr, $rhs: expr) => {
        fn $name(&mut $self) -> ExprResult {
            let mut lhs = $lhs?;

            while $self.match_token(&mut $tok_list).is_some() {
                let op = $self.previous();
                let rhs = $rhs?;
                lhs = Expr::new_binary(lhs, op, rhs);
            }
            Ok(lhs)
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

#[derive(Debug,Clone)]
pub enum ParserErrorType {
    ExpectIdentifier(String),
    MissingIdentifier(String),
    Invaild(String)
}

impl std::fmt::Display for ParserErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f,"{}",match self {
            Self::ExpectIdentifier(msg_expect) => format!("Expect: {}", msg_expect),
            Self::Invaild(msg_invaild) => format!("Invaild: {}",msg_invaild),
            Self::MissingIdentifier(msg_missing) => format!("Missing: {}", msg_missing)
        })
    }
}

#[derive(Debug)]
pub struct ParserError {
    error_type: ParserErrorType,
    filename: String,
    token_data: TokenData
}
impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",
            MessageHandler::throw_message(&self.filename, MessageType::Error, self.token_data.line, self.token_data.start,self.error_type.clone()))
    }
}

impl std::error::Error for ParserError {
    fn description(&self) -> &str {
        "Error!"
    }
}

type ParserResult<T> = Result<T,ParserError>;
type ExprResult = ParserResult<Expr>;

impl ParserError {
    pub fn new(error_type:ParserErrorType,filename: String, token_data:TokenData) -> Self {
        Self {error_type,filename, token_data}
    }
}

pub struct AST {
    filename: String,
    token: Vec<TokenData>,
    current: usize,
}

impl AST {
    pub fn new(meta_data: MetaData) -> Self {
        Self { token: meta_data.clone().tok_data, current:0, filename: meta_data.clone().filename }

    }

    pub fn get_filename(self) -> String {
        self.filename
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

    fn match_token(&mut self, types: &mut Vec<TokenType>) -> Option<TokenData> {
        if types.iter().any(|&f| self.check(f)) {
            Some(self.advance())
        } else {
            None
        }
    }

    fn consume(&mut self, tok_type: TokenType, message: ParserErrorType) -> Result<(), ParserError> {
        if self.check(tok_type) { self.advance(); }
        else {
            return Err(self.error(message, self.previous()))
        }
        Ok(())
    }
    fn error(&self, message: ParserErrorType, tok_data: TokenData) -> ParserError {
        ParserError::new(message, self.filename.clone(), tok_data)
    }

    fn primary(&mut self) -> ExprResult {
        if self.match_token(&mut vec![TokenType::Number, TokenType::String, TokenType::Char]).is_some() {
            return Ok(Expr::new_terminal(self.previous(),expr_node::TerminalType::Literal))
        }
        if self.match_token(&mut vec![TokenType::LeftParen]).is_some() {
            let expr = self.expr()?;
            self.consume(TokenType::RightParen, ParserErrorType::ExpectIdentifier("Expect ')'".into()))?;
            return Ok(Expr::new_group(expr));
        }
        if self.match_token(&mut vec![TokenType::Keywords, TokenType::DataType]).is_some() {
            return Ok(Expr::new_terminal(self.previous(),expr_node::TerminalType::Identifier));
        }
        if self.match_token(&mut vec![TokenType::Identifier]).is_some() {
            return Ok(Expr::new_terminal(self.previous(),expr_node::TerminalType::Var));
        }
        if self.match_token(&mut vec![TokenType::LeftBracket]).is_some() {
            return self.list();
        }

        Err(self.error(ParserErrorType::ExpectIdentifier(
            format!("Expect expression of {:?}", self.peek().tok_type)
        ),self.peek()))
    }

    fn callee(&mut self) -> ExprResult {
        let mut primary = self.primary()?;
        if self.match_token(&mut vec![TokenType::LeftParen]).is_some() {
            let mut arg_v = Vec::new();
            while !self.check(TokenType::RightParen) {
                arg_v.push(self.expr()?);
                if !self.check(TokenType::RightParen) {
                    self.consume(TokenType::Comma,
                        ParserErrorType::ExpectIdentifier("Expect ',' in parameter declare".into()))?;
                }
            }
            self.consume(TokenType::RightParen, ParserErrorType::ExpectIdentifier("Expect ')' after callee".into()))?;
            primary = Expr::new_callee(primary, arg_v);
        }
        Ok(primary)
    }

    fn unary(&mut self) -> ExprResult {
        if self.match_token(&mut vec![TokenType::Not, TokenType::Minus]).is_some() {
            let op = self.previous();
            let expr = self.unary()?;
            Ok(Expr::new_unary(op, expr))
        } else{
            self.callee()
        }
    }

    // very rust
    create_binary!(self, factor, self.unary(), vec![TokenType::Star, TokenType::Slash], self.unary());
    create_binary!(self, term, self.factor(), vec![TokenType::Plus, TokenType::Minus], self.factor());
    create_binary!(self, compare, self.term(), vec![TokenType::Less, TokenType::LessEqual, TokenType::Greater, TokenType::GreaterEqual], self.term());
    create_binary!(self, shift, self.compare(), vec![TokenType::ShiftLeft, TokenType::ShiftRight], self.compare());
    create_binary!(self, equal, self.shift(), vec![TokenType::EqualEqual, TokenType::NotEqual], self.shift());
    create_binary!(self, logical, self.equal(), vec![TokenType::Or, TokenType::And], self.equal());
    create_binary!(self, bool_logical, self.logical(), vec![TokenType::OrBool, TokenType::AndBool], self.logical());

    fn casting(&mut self) -> ExprResult {
        // let k = <int>34.1; <- this will be cast in compile time
        // let k = <A*>0x12; <- this will be cast in runtime, where A is sturct or class
        if !self.check(TokenType::Less) { return self.assignment() }

        self.advance(); // eat '<'

        let cast_dt = self.primary()?.identifier().expect("Unknown cast data type");
        let is_pointer = if self.peek().tok_type == TokenType::Star {
            self.advance();
            true
        } else {false};
        self.consume(
            TokenType::Greater,
            ParserErrorType::ExpectIdentifier("Expect '>' in casting".into())
        )?;

        //return Box::new(Expr::Cast(VariableType::new(cast_dt,is_pointer,false), self.expr()))
        Ok(Expr::default())
    }

    fn expr(&mut self) -> ExprResult {
        self.casting()
    }

    fn assignment(&mut self) -> ExprResult {
        let expr = self.bool_logical()?;

        if self.match_token(&mut vec![TokenType::Equal]).is_some() {
            let v = self.assignment()?;
            let Some(n) = expr.clone().var() else {
                return Err(self.error(
                    ParserErrorType::Invaild("assignment object".into()),
                    self.peek(),
                ));
            };
            return Ok(Expr::new(ExprType::Assign(n, v),expr.line,expr.at));
        }
        Ok(expr)
    }

    fn while_stmt(&mut self) -> ExprResult {
        let expr = self.expr()?;
        let body = self.statement()?;
        Ok(Expr::new(ExprType::WhileStmt(expr.clone(), body), expr.line, expr.at))
    }

    fn get_data_type(&mut self) -> ParserResult<VariableType> {
        //i32*, i32&, A*, A&
        let Some(name) = self.primary()?.identifier() else {
            return Err(self.error(
                ParserErrorType::Invaild("Data type is not an indentifier".into()),
                self.peek()));
        };

        let is_ptr = self.match_token(&mut vec![TokenType::Star]).is_some();

        Ok(if is_ptr {
            VariableType::Pointer(DataType::from(name))
        } else {
            VariableType::NonPointer(DataType::from(name))
        })
    }

    fn func_header(&mut self) -> ParserResult<FuncHeader>{
        let f = self.primary()?;
        let Some(func_name) = f.clone().var() else {
            return Err(
                self.error(
                    ParserErrorType::Invaild(format!("Invaild function name, got: {:?}", f)),
                    self.peek()
                )
            );
        };

        self.consume(
            TokenType::LeftParen,
            ParserErrorType::ExpectIdentifier("Expect '(' in declare func".into()))?;
        let mut arg_v = Vec::new();

        while !self.check(TokenType::RightParen) {
            let name = self.primary()?.var().unwrap();
            self.consume(TokenType::Colon,
                ParserErrorType::MissingIdentifier("Missing colon in declare function arguments".into()))?;
            let dt = self.get_data_type()?;
            arg_v.push(Variable::new(Some(dt),name,None));
            if !self.check(TokenType::RightParen) {
                self.consume(TokenType::Comma, ParserErrorType::ExpectIdentifier("Expect ',' in arguments declare".into()))?;
            }
        }

        self.consume(TokenType::RightParen, ParserErrorType::ExpectIdentifier("Expect ')' in declare func".into()))?;

        let return_type = self.match_token(&mut vec![TokenType::PointTo]).and_then(|_| {
            self.get_data_type().ok()
        });

        Ok(FuncHeader { name: func_name, args: arg_v, return_type: return_type })
    }

    fn func_stmt(&mut self) -> ExprResult {
        let func_header= self.func_header()?;
        let body = self.block()?;
        Ok(Expr::new(ExprType::FuncStmt(func_header, body.clone()), body.line, body.at))
    }

    fn list(&mut self) -> Result<Expr,ParserError>{
        Ok(Expr::default())
        // let mut data_type = DataType::Unknown;
        // let mut l: Vec<Value> = Vec::new();
        // while !self.check(TokenType::RightBracket) {
        //     let v = self.primary();
        //     l.push(v.to_value());
        //     if l.len() == 1 {
        //         data_type=v.to_value().to_datatype();
        //     }else {
        //         if v.to_value().to_datatype() != data_type {
        //             let p = self.peek();
        //             MessageHandler::throw_message(
        //                 &self.filename,
        //                 crate::MessageHandler::MessageType::Error,
        //                 p.line, p.start, &format!("List item must be same as {:?}", data_type));
        //             exit(1);
        //         }
        //     }
        //     if !self.check(TokenType::RightBracket) {
        //         self.consume(TokenType::Comma, "Expect ',' in list item declaration");
        //     }
        // }
        // self.consume(TokenType::RightBracket, "Expect ']' in list declaration");

        // Box::new (
        //     ExprType::List(l)
        // )
    }

    fn class_stmt(&mut self) -> ExprResult {
        let Some(class_name) = self.primary()?.var() else {
            return Err(self.error(
                ParserErrorType::Invaild("Invaild class name".into()),
                self.previous()
            ))
        };

        self.consume(TokenType::LeftBrace, ParserErrorType::ExpectIdentifier("Expect '{' in class definition".into()))?;
        let mut func_list = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_eof() {
            let access =
                    match self.advance().identifier.as_str() {
                        "public" => AccessLevel::Public,
                        "private" => AccessLevel::Private,
                        _ => AccessLevel::None
                    };
            let is_func = self.peek().identifier == "func";
            if is_func {
                 self.advance();
            }
            let class_func_type =
                if is_func {
                    ClassFunctionType::Function
                } else if self.check(TokenType::Tilde) {
                    self.advance();
                    ClassFunctionType::Deconstructor
                } else {
                    ClassFunctionType::Initializer
                };
            let f = self.func_stmt()?;
            let f_name = f.clone().function().unwrap().0.name;
            if f_name != class_name &&
                (class_func_type == ClassFunctionType::Initializer ||
                class_func_type == ClassFunctionType::Deconstructor) {
                    return Err(self.error(ParserErrorType::Invaild("Invaild constructor/decontructor function name".into()), self.peek()));
                }
                func_list.push(ClassFunction{
                        function: f,
                        func_type: class_func_type,
                        access_level: access
                    });
                self.consume(TokenType::RightBrace, ParserErrorType::ExpectIdentifier("Expect '}' in class definition".into()))?;
        }
        Ok(Expr::new(ExprType::Class(class_name, func_list), 0,0))
    }

    fn statement(&mut self) -> ExprResult {

        if self.check(TokenType::LeftBrace) {
            return self.block();
        }
        check_keyword!(self, "if",self.if_stmt());
        check_keyword!(self, "while", self.while_stmt());
        check_keyword!(self, "func", self.func_stmt());
        check_keyword!(self, "extern", self.extern_func());
        check_keyword!(self, "return", self.return_keyw());
        check_keyword!(self, "class", self.class_stmt());

        let var_expr = self.var_decl()?;
        let var_type = var_expr.clone().get_expr_type().and_then(|f| {
            self.consume(TokenType::Semicolon, ParserErrorType::ExpectIdentifier("Expect semicolon".into())).ok()?;
            Some(*f)
        }).unwrap_or(ExprType::None);
        Ok(Expr::new(var_type, var_expr.line, var_expr.at))
    }

    fn return_keyw(&mut self) -> ExprResult {
        // return 3;
        let v = if !self.check(TokenType::Semicolon) {
            Some(self.expr()?)
        } else {None};

        self.consume(TokenType::Semicolon,
            ParserErrorType::ExpectIdentifier("Expect ';' after return keyword".into()))?;

        Ok(Expr::new(ExprType::Return(v),0,0))
    }

    fn extern_func(&mut self) -> ExprResult {
        //extern <func_header>;

        if self.advance().identifier != "func" {
            return Err(self.error(ParserErrorType::MissingIdentifier("missing 'func'".into()),self.peek()))
        }

        let func_header = self.func_header()?;

        self.consume(TokenType::Semicolon, ParserErrorType::ExpectIdentifier("Expect ';' after extern function".into()))?;

        Ok(Expr::new(
            ExprType::Extern(func_header),
            0,0
        ))
    }


    fn if_stmt(&mut self) -> ExprResult {
        let condition = self.expr()?;
        let then_block = self.statement()?;
        let else_block = if self.peek().identifier == "else" {
            self.advance();
            Some(self.statement()?)
        } else {None};
        Ok(Expr::new(ExprType::IfStmt(condition.clone(), then_block, else_block), condition.line, condition.at))
    }

    fn block(&mut self) -> Result<Expr, ParserError> {
        /*
         * {
         *  int a = 0;
         *
         * }
         * */

         self.consume(TokenType::LeftBrace,
             ParserErrorType::ExpectIdentifier(
                 format!("Expect '{{' in declare block")
             ))?;
        let mut block = Vec::new();
        let start_block = self.previous();
        while ! self.check(TokenType::RightBrace) && !self.is_eof() {
            block.push(self.statement()?);
        }
        self.consume(TokenType::RightBrace, ParserErrorType::ExpectIdentifier("Expect '}' after declare block".into()))?;
        Ok(Expr::new(ExprType::Block(block.clone()), start_block.line, start_block.start))
    }


    fn var_decl(&mut self) -> ExprResult {
        /*
         * let mutable_value = 4;
         * const constant_value = 6;
         * let mutable_value_with_known_type: u32 = function_that_return_u32();
         * let mutable_value_with_identifier_type: Rectangle = { x = 4, y = 4, w = 5, h = 5 };
         */
        let peek = self.peek();
        if peek.identifier != "let"
        && peek.identifier != "const" {
            return self.expr();
        }
        let is_const = self.advance().identifier=="const";
        let Some(name) = self.primary()?.var() else {
            return Err(self.error(
                ParserErrorType::Invaild(format!("Invaild variable name, got: {:?}", self.peek())),
                self.peek()
            ));
        };

        let vt: Option<VariableType> = if self.match_token(&mut vec![TokenType::Colon]).is_some() {
            let data_type = DataType::from(
                self.primary()?.identifier().unwrap()
            );

            let is_pointer = self.match_token(&mut vec![TokenType::Star]).is_some();
            Some(VariableType::new(data_type, is_pointer, is_const))
        } else {
            None
        };

        let init: Option<Expr> = if self.match_token(&mut vec![TokenType::Equal]).is_some() {
            Some(self.expr()?)
        } else {None};

        Ok(Expr::new(ExprType::VarDecl(Variable::new(vt, name, init)),peek.line,peek.start))
    }


    pub fn parse(&mut self) -> Result<Vec<Expr>, ParserError> {
        let mut expr_vec: Vec<Expr> = Vec::new();

        while !self.is_eof() {
            if self.match_token(&mut vec![TokenType::Macro]).is_some() {
                let mut vect: Vec<Expr> = Vec::new();
                if let Some(st) = self.previous().sub_tok {
                    let mut macro_queue = VecDeque::from(st);

                    let macro_name = macro_queue.pop_front().unwrap();
                    let mut sub_ast = AST::new(MetaData {filename: self.filename.clone(), tok_data: Vec::from(macro_queue)});
                    while !sub_ast.is_eof() {
                        vect.push(sub_ast.expr()?);
                    }
                    //expr_vec.push(ExprType::Macro(macro_name.identifier, vect));
                }
            }
            else {
                self.statement()?.if_vaild_then(|f| {
                    expr_vec.push(f);
                });
            }
        }

        Ok(expr_vec)
    }
}
