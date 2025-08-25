#[cfg(test)]
mod test {
    use crate::{token::{token_type::TokenType, Token, TokenData}, Value::Value};

    #[test]
    fn tokenizer_test_simple() {
        let mut t = Token::new("(()".to_string());
        let meta_data = t.tokenize();
        assert_eq!(meta_data.tok_data, vec![
            TokenData {
                tok_type: TokenType::LeftParen,
                start:0,
                end:1,
                identifier: "".to_string(),
                line: 1,
                value: Value::Null,
                sub_tok: None
            },
            TokenData {
                tok_type: TokenType::LeftParen,
                start:1,
                end:2,
                identifier: "".to_string(),
                line: 1,
                value: Value::Null,
                sub_tok: None
            },
            TokenData {
                tok_type: TokenType::RightParen,
                start: 2,
                end: 3,
                identifier: "".to_string(),
                line: 1,
                value: Value::Null,
                sub_tok: None
            },
            TokenData {
                tok_type: TokenType::EOF,
                start: 0,
                end: 0,
                identifier: "".to_string(),
                line: 1,
                value: Value::Null,
                sub_tok: None
            }
        ])
    }
    #[test] 
    fn tokenizer_test_string() {
        let mut t = Token::new("\"Hello World\"".to_string());
        let meta_data = t.tokenize();
        assert_eq!(meta_data.tok_data, vec![
            TokenData {
                tok_type: TokenType::String,
                start: 0,
                end: 13,
                identifier: "\"Hello World\"".to_string(),
                line: 1,
                value: Value::Str("Hello World".to_string()),
                sub_tok: None
            },
            TokenData {
                tok_type: TokenType::EOF,
                start: 0,
                end: 0,
                identifier: "".to_string(),
                line: 1,
                value: Value::Null,
                sub_tok: None
            }
        ])
    }
    #[test] 
    fn tokenizer_test_identifier_keyword() {
        let mut t = Token::new("abcxyz".to_string());
        let meta_data1 = t.tokenize();
        assert_eq!(meta_data1.tok_data, vec![
            TokenData {
                tok_type: TokenType::Identifier,
                start: 0,
                end: 6,
                identifier: "abcxyz".to_string(),
                line: 1,
                value: Value::Object("abcxyz".to_string()),
                sub_tok: None
            },
            TokenData {
                tok_type: TokenType::EOF,
                start: 0,
                end: 0,
                identifier: "".to_string(),
                line: 1,
                value: Value::Null,
                sub_tok: None
            }
        ]);
        let mut t1 = Token::new("suu number".to_string());
        let meta_data2 = t1.tokenize();
        assert_eq!(meta_data2.tok_data, vec![
            TokenData {
                tok_type: TokenType::DataType,
                start: 0,
                end: 3,
                identifier: "suu".to_string(),
                line: 1,
                value: Value::Object("suu".to_string()),
                sub_tok: None
            },
            TokenData {
                tok_type: TokenType::Identifier,
                start: 4,
                end: 10,
                identifier: "number".to_string(),
                line: 1,
                value: Value::Object("number".to_string()),
                sub_tok: None
            },
            TokenData {
                tok_type: TokenType::EOF,
                start: 0,
                end: 0,
                identifier: "".to_string(),
                line: 1,
                value: Value::Null,
                sub_tok: None
            }
        ])
    }

    #[test]
    fn value_test() {
        let v = Value::new("1".to_string());
        assert_eq!(*v.value().unwrap().downcast_ref::<i64>().unwrap(), 1)
    }

}
