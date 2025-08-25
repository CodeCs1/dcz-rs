use std::{fs::File,io::Read, path::Path, process::exit};

use token_type::TokenType;

use crate::{DataSection::DataSection, MessageHandler::message_handler::{self, throw_message}, Value::Value};
pub mod token_type;

#[derive(Debug, PartialEq, Clone)]
pub struct TokenData {
    pub tok_type: TokenType,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub identifier: String,
    pub value: Value,
    pub sub_tok: Option<Vec<TokenData>>
}

// Source file metadata
#[derive(Debug, Clone)]
pub struct MetaData {
    pub filename: String,
    pub tok_data: Vec<TokenData>,
    pub data: DataSection,
}


pub struct Token {
    code: String,
    current: usize,
    at: usize,
    start: usize,
    line: usize,
    data: DataSection,
    pub source_file_name:String,
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
        Self { code: code, current:0,start:0, line:1, data: DataSection::new(), source_file_name: "stdin".to_string(), at:0 }
    }

    pub fn FromIO(p: &Path, mut fileio: File) -> Result<Self,Box<dyn std::error::Error>> {
        let mut file_content = String::new();
        fileio.read_to_string(&mut file_content)?;
        Ok(Self { code: file_content,current:0,start:0, line:1, data: DataSection::new(), source_file_name: p.display().to_string(), at:0 })
    }

    fn is_eof(&self) -> bool {
        self.current >= self.code.len()
    }

    fn ToTokenData_Symbol(&self, tok_type:TokenType) -> TokenData {
        self.To_TokenData_Identifier(tok_type, String::new())
    }

    fn To_TokenData_Identifier(&self, tok_type: TokenType, identifier: String) -> TokenData {
        TokenData { tok_type: tok_type, start: self.start, end: self.current, identifier: identifier.clone(), line: self.line , value: Value::new(identifier), sub_tok: None}
    }    
    fn To_TokenData_String(&self, string_literal: String) -> TokenData {
        TokenData { tok_type: TokenType::String, start: self.start, end: self.current, identifier: string_literal.clone(), line: self.line , value: Value::new(string_literal[1..string_literal.len()-1].to_string()),sub_tok: None}
    }
    fn To_TokenData_Obj(&self, tok_type: TokenType, identifier: String) -> TokenData {
        TokenData { tok_type: tok_type, start: self.start, end: self.current, identifier: identifier.clone(), line: self.line , value: Value::new_obj(identifier.trim().to_string()),sub_tok: None}
    }
    fn To_TokenData_SubToken(&self, tok_type: TokenType, sub_tok: Vec<TokenData>) -> TokenData {
        TokenData { tok_type: tok_type, start: self.start, end: self.current, line: self.line, identifier: String::new(), value: Value::Null, sub_tok: Some(sub_tok) }
    }
    fn advance(&mut self) -> char {
        self.current+=1;
        self.at+=1;
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
        self.at+=1;

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

    fn tokenize_single_char(&mut self) -> Option<TokenData> {
        self.start = self.current;
        let curr_char = self.advance();
        match curr_char {
            '(' => Some(self.ToTokenData_Symbol(TokenType::LeftParen)),
            ')' => Some(self.ToTokenData_Symbol(TokenType::RightParen)),
            '{' => Some(self.ToTokenData_Symbol(TokenType::LeftBrace)),
            '}' => Some(self.ToTokenData_Symbol(TokenType::RightBrace)),
            '[' => Some(self.ToTokenData_Symbol(TokenType::LeftBracket)),
            ']' => Some(self.ToTokenData_Symbol(TokenType::RightBracket)),
            '+' => Some(self.ToTokenData_Symbol(TokenType::Plus)),
            '-' => {
                if self.match_chr('>') {
                    Some(self.ToTokenData_Symbol(TokenType::PointTo))
                }else {
                    Some(self.ToTokenData_Symbol(TokenType::Minus))
                }
            },
            '*' => Some(self.ToTokenData_Symbol(TokenType::Star)),
            '/' => Some(self.ToTokenData_Symbol(TokenType::Slash)),
            ';' => Some(self.ToTokenData_Symbol(TokenType::Semicolon)),
            ':' => Some(self.ToTokenData_Symbol(TokenType::Colon)),
            '=' => {
                if self.match_chr('=') {
                        Some(self.ToTokenData_Symbol(TokenType::EqualEqual))
                    } else {
                        Some(self.ToTokenData_Symbol(TokenType::Equal))
                    }
                },
                '>' => {
                    if self.match_chr('=') {
                        Some(self.ToTokenData_Symbol(TokenType::GreaterEqual))
                    } else if self.match_chr('>') {
                        Some(self.ToTokenData_Symbol(TokenType::ShiftRight))
                    } else {
                        Some(self.ToTokenData_Symbol(TokenType::Greater))
                    }
                }
                '&' => {
                    if self.match_chr('&') {
                        Some(self.ToTokenData_Symbol(TokenType::AndBool))
                    } else {
                        Some(self.ToTokenData_Symbol(TokenType::And))
                    }
                }
                '|' => {
                    if self.match_chr('|') {
                        Some(self.ToTokenData_Symbol(TokenType::OrBool))
                    } else {
                        Some(self.ToTokenData_Symbol(TokenType::Or))
                    }
                }
                '<' => {
                    if self.match_chr('=') {
                        Some(self.ToTokenData_Symbol(TokenType::LessEqual))
                    } else if self.match_chr('<') {
                        Some(self.ToTokenData_Symbol(TokenType::ShiftLeft))
                    } else {
                        Some(self.ToTokenData_Symbol(TokenType::Less))
                    }
                }
                '!' => {
                    if self.match_chr('=') {
                        Some(self.ToTokenData_Symbol(TokenType::NotEqual))
                    } else {
                        Some(self.ToTokenData_Symbol(TokenType::Not))
                    }
                }
                '%' => Some(self.ToTokenData_Symbol(TokenType::Modulo)),
                ',' => Some(self.ToTokenData_Symbol(TokenType::Comma)),
                '#' => {
                    if self.match_chr('#') {
                        while self.match_str("##") && ! self.is_eof() {
                            if self.peek() == '\n' { self.line+=1; }
                            self.advance();
                        }
                        None
                    } else if self.match_chr('!') {
                        let mut sub_token: Vec<TokenData> = Vec::new();

                        while self.peek() != '\n' && !self.is_eof() {
                            let td = self.tokenize_single_char();
                            if td.is_some() {
                                sub_token.push(
                                    td.unwrap()
                                );
                            }
                        }

                        Some(self.To_TokenData_SubToken(TokenType::Macro, sub_token))

                    } else {
                        while self.peek() != '\n' && ! self.is_eof() {
                            self.advance();
                        }
                        None
                    }
                }

                '"' => {
                    while self.peek() != '"' && !self.is_eof() {
                        if self.peek() == '\n' {
                            throw_message(
                            &self.source_file_name, 
                            message_handler::MessageType::Error,
                            self.line as i64, 
                            self.at as i64, 
                            "Invaild string literal format");
                            exit(1);
                        }
                        self.advance();
                    }

                    if self.is_eof() {
                        throw_message(
                            &self.source_file_name, 
                            message_handler::MessageType::Error,
                            self.line as i64, 
                            self.at as i64, 
                            "Unterminated string literal");
                            exit(1);
                    }
                    self.advance();
                    let mut sub_str = self.code[self.start..self.current].to_string();

                    sub_str = fmt_escape(sub_str);

                    // add str to data section
                    self.data.append_string(sub_str.clone());
                    Some(self.To_TokenData_String(sub_str.clone()))
                }

                '0'..='9' => {

                    let mut radix = 10;

                    if curr_char == '0' {
                        radix = match self.peek() {
                            'x' => { self.advance(); 16 },
                            'b' => { self.advance(); 2 },
                            'o' => { self.advance(); 8 },
                            _ => { 10 }
                        };
                    }

                    while self.peek().is_digit(radix) { self.advance(); }
                    
                    
                    if self.peek() == '.' && self.peek_next().is_digit(10) && radix==10 {
                        self.advance();
                        while self.peek().is_digit(10) { self.advance(); }
                    }
                    

                    Some(self.To_TokenData_Identifier(TokenType::Number, self.code[self.start..self.current].to_string()))
                }
                '\'' => {
                    // char support
                    self.advance();
                    if !self.match_chr('\'') {
                        throw_message(
                            &self.source_file_name, 
                            message_handler::MessageType::Error,
                            self.line as i64, 
                            self.at as i64, 
                            "Invaild char format!");
                        exit(1);
                    }
                    Some(self.To_TokenData_Identifier(TokenType::Char, self.code[self.start+1..self.current-1].to_string()))
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
                        "long",
                        "const"
                    ];

                    while self.peek().is_alphanumeric() || self.peek() == '_' { self.advance(); }

                    let str_text = &self.code[self.start..self.current];

                    if kw.iter().any(|f| *f==str_text) {
                        //self.ToTokenData_Symbol(TokenType::Keywords);
                        Some(self.To_TokenData_Obj(TokenType::Keywords, str_text.to_string()))
                    } else if data_type_kw.iter().any(|f| *f==str_text)  {
                        Some(self.To_TokenData_Obj(TokenType::DataType, str_text.to_string()))
                    } else {
                        //self.ToTokenData_Symbol(TokenType::Identifier);
                        Some(self.To_TokenData_Obj(TokenType::Identifier, str_text.to_string()))
                    }
                }
                '\r' | '\t' | ' ' => {None},
                '\n' => { 
                    self.line+=1;
                    self.at = 0;
                    None
                },
                _ => {
                    throw_message(&self.source_file_name, 
                        message_handler::MessageType::Error, 
                        self.at as i64, 
                        self.current as i64,
                        &format!("Unknown token: {}", curr_char)
                    );
                    exit(1);
                }
            }
    }

    pub fn tokenize(&mut self) -> MetaData {
        let mut token_data: Vec<TokenData> = Vec::new();
        while !self.is_eof() {
            let tok_data = self.tokenize_single_char();
            if tok_data.is_some() {
                token_data.push(tok_data.unwrap());
            }
        }
        token_data.push(
            TokenData { 
                tok_type: TokenType::EOF,
                start: self.start, 
                end: self.current, 
                line:self.line, 
                identifier: String::new(), 
                value: Value::Null ,
                sub_tok: None
            });

        self.current = 0;
        self.start =0;

        MetaData { filename: self.source_file_name.clone(), tok_data: token_data, data: self.data.clone() }
    }
}
