use token_type::TokenType;

use crate::{DataSection::DataSection, Value::Value};
pub mod token_type;

#[derive(Debug, PartialEq, Clone)]
pub struct TokenData {
    pub tok_type: TokenType,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub identifier: String,
    pub value: Value
}


pub struct Token {
    code: String,
    pub tok_data: Vec<TokenData>,
    current: usize,
    start: usize,
    line: usize,
    pub data: DataSection
}

fn fmt_escape(string: String) -> String {
    let mut s = String::new();

    let mut i =0;
    while i < string.len() {
        let mut c = string.chars().nth(i).expect("Out of bound or smth");
        if c == '\\' {
            let e = string.chars().nth(i+1).expect("missing escape character");
            c=match e {
                '0' => 0x00 as char,
                'a' => 0x07 as char,
                'b' => 0x08 as char,
                't' => 0x09 as char,
                'n' => 0x0A as char,
                'v' => 0x0b as char,
                'f' => 0x0c as char,
                'r' => 0x0d as char,
                '\\' => '\\',
                _ => c
            };
            i+=2
        } else {
            i+=1;
        }
        s.push(c);
    }
    s
}


impl Token {
    pub fn new(code: String) -> Self {
        Self { code: code, tok_data: Vec::new(), current:0,start:0, line:1, data: DataSection::new() }
    }

    fn is_eof(&self) -> bool {
        self.current >= self.code.len()
    }

    fn add_token(&mut self, tok_type: TokenType, identifier: String) {
    self.tok_data.push(TokenData { tok_type: tok_type, start: self.start, end: self.current, identifier: identifier.clone(), line: self.line , value: Value::new(identifier)});
    }    
    fn add_str_token(&mut self, identifier: String) {
        self.tok_data.push(TokenData { tok_type: TokenType::String, start: self.start, end: self.current, identifier: identifier.clone(), line: self.line , value: Value::new(identifier[1..identifier.len()-1].to_string())});
    }
    fn add_obj_token(&mut self, tok_type: TokenType, identifier: String) {
        self.tok_data.push(TokenData { tok_type: tok_type, start: self.start, end: self.current, identifier: identifier.clone(), line: self.line , value: Value::new_obj(identifier.trim().to_string())});
    }
    fn add_symbol(&mut self, token_type: TokenType) {
        self.add_token(token_type, String::new());
    }
    fn advance(&mut self) -> char {
        self.current+=1;
        self.code.chars().nth(self.current-1).unwrap()
    }

    fn peek(&self) -> char {
        if self.is_eof() { return '\0'; }
        self.code.chars().nth(self.current).unwrap()
    }

    fn match_chr(&mut self, expect: char) -> bool {
        if self.is_eof() { return false; }
        if self.code.chars().nth(self.current).unwrap() != expect { return false; }

        self.current+=1;

        true
    }
    fn match_str(&mut self, expect:&'static str) -> bool{
        for i in expect.chars() {
            if ! self.match_chr(i) { return false; }
        }
        true
    }

    fn peek_next(&self) -> char {
        if self.current+1 >= self.code.len() { return '\0'; }
        self.code.chars().nth(self.current+1).expect("peek_next: null char returned")
    }

    pub fn tokenize(&mut self) {
        while !self.is_eof() {
            self.start = self.current;
            let curr_char = self.advance();
            match curr_char {
                '(' => self.add_symbol(TokenType::LeftParen),
                ')' => self.add_symbol(TokenType::RightParen),
                '{' => self.add_symbol(TokenType::LeftBrace),
                '}' => self.add_symbol(TokenType::RightBrace),
                '[' => self.add_symbol(TokenType::LeftBracket),
                ']' => self.add_symbol(TokenType::RightBracket),
                '+' => self.add_symbol(TokenType::Plus),
                '-' => self.add_symbol(TokenType::Minus),
                '*' => self.add_symbol(TokenType::Star),
                '/' => self.add_symbol(TokenType::Slash),
                ';' => self.add_symbol(TokenType::Semicolon),
                '=' => {
                    if self.match_chr('=') {
                        self.add_symbol(TokenType::EqualEqual);
                    } else {
                        self.add_symbol(TokenType::Equal);
                    }
                },
                '>' => {
                    if self.match_chr('=') {
                        self.add_symbol(TokenType::GreaterEqual);
                    } else if self.match_chr('>') {
                        self.add_symbol(TokenType::ShiftRight);
                    } else {
                        self.add_symbol(TokenType::Greater);
                    }
                }
                '&' => {
                    if self.match_chr('&') {
                        self.add_symbol(TokenType::AndBool);
                    } else {
                        self.add_symbol(TokenType::And);
                    }
                }
                '|' => {
                    if self.match_chr('|') {
                        self.add_symbol(TokenType::OrBool);
                    } else {
                        self.add_symbol(TokenType::Or);
                    }
                }
                '<' => {
                    if self.match_chr('=') {
                        self.add_symbol(TokenType::LessEqual);
                    } else if self.match_chr('<') {
                        self.add_symbol(TokenType::ShiftLeft);
                    } else {
                        self.add_symbol(TokenType::Less);
                    }
                }
                '!' => {
                    if self.match_chr('=') {
                        self.add_symbol(TokenType::NotEqual);
                    } else {
                        self.add_symbol(TokenType::Not);
                    }
                }
                '%' => self.add_symbol(TokenType::Modulo),
                ',' => self.add_symbol(TokenType::Comma),
                '#' => {
                    if self.match_chr('#') {
                        while self.match_str("##") && ! self.is_eof() {
                            if self.peek() == '\n' { self.line+=1; }
                            self.advance();
                        }
                    } else if self.match_chr('!') {
                        while self.peek() != ' ' && !self.is_eof() {
                            self.advance();
                        }

                        self.add_obj_token(TokenType::Macro, self.code[self.start+2..self.current].to_string());

                    } else {
                        while self.peek() != '\n' && ! self.is_eof() {
                            self.advance();
                        }
                    }
                }

                '"' => {
                    while self.peek() != '"' && !self.is_eof() {
                        if self.peek() == '\n' {
                            panic!("Invaild string format.")
                        }
                        self.advance();
                    }

                    if self.is_eof() {
                        panic!("Unterminated string");
                    }
                    self.advance();
                    let mut sub_str = self.code[self.start..self.current].to_string();

                    sub_str = fmt_escape(sub_str);

                    self.add_str_token( sub_str.clone());

                    // add str to data section
                    self.data.append_string(sub_str);
                }
                '0'..='9' => {
                    while self.peek().is_digit(10) { self.advance(); }
                    
                    if self.peek() == '.' && self.peek_next().is_digit(10) {
                        self.advance();
                        while self.peek().is_digit(10) { self.advance(); }
                    }
                    self.add_token(TokenType::Number, self.code[self.start..self.current].to_string());
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let kw = vec![
                        //block code
                        "func",
                        "if",
                        "else",
                        "while",
                        "for",
                        "let",
                        "extern"
                    ];

                    let data_type_kw = vec![
                        // data type
                        "int",
                        "float",
                        "suu",
                        "char",
                        "short",
                        "long"
                    ];

                    while self.peek().is_alphanumeric()  { self.advance(); }

                    let str_text = &self.code[self.start..self.current];

                    if kw.iter().any(|f| *f==str_text) {
                        //self.add_symbol(TokenType::Keywords);
                        self.add_obj_token(TokenType::Keywords, str_text.to_string());
                    } else if data_type_kw.iter().any(|f| *f==str_text)  {
                        self.add_obj_token(TokenType::DataType, str_text.to_string());
                    } else {
                        //self.add_symbol(TokenType::Identifier);
                        self.add_obj_token(TokenType::Identifier, str_text.to_string());
                    }
                }
                '\r' | '\t' | ' ' => {},
                '\n' => { 
                    self.add_symbol(TokenType::NewLine);
                    self.line+=1;
                },
                _ => {
                    panic!("Undefined token {}.", curr_char);
                }
            }
        }
        self.current = 0;
        self.start =0;
        self.add_symbol(TokenType::EOF);
    }
}
