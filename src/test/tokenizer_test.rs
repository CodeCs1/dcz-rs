#[cfg(test)]
mod tests {
    use crate::{
        token::{
            TokenData,Token,token_type::TokenType
        },
        Value::Value
    };

    #[test]
    fn tokenizer_test_simple() {
        let tok = Token::new("(()".to_string(), None).tokenize();
        assert_eq!(&format!("{:?}",tok.tok_data), "\
            [TokenData { tok_type: LeftParen, start: 0, \
            end: 1, line: 1, identifier: \"\", \
            value: Null, sub_tok: None }, \
            TokenData { tok_type: LeftParen, \
            start: 1, end: 2, line: 1, \
            identifier: \"\", value: Null, sub_tok: None }, \
            TokenData { tok_type: RightParen, start: 2, end: 3, \
            line: 1, identifier: \"\", value: Null, sub_tok: None }, \
            TokenData { tok_type: EOF, start: 2, end: 3, line: 1, \
            identifier: \"\", value: Null, sub_tok: None }]");
    }
    #[test]
    fn tokenizer_test_string() {
        let tok = Token::new("\"Hello World\"".to_string(), None).tokenize();
        assert_eq!(&format!("{:?}",tok.tok_data), "\
            [TokenData { \
            tok_type: String, start: 0, end: 13, line: 1, \
            identifier: \"\\\"Hello World\\\"\", \
            value: Str(\"Hello World\"), sub_tok: None }, \
            TokenData { tok_type: EOF, start: 0, end: 13, \
            line: 1, identifier: \"\", value: Null, sub_tok: None }]");
    }
    #[test]
    fn tokenizer_test_identifier_keyword() {
        let tok = Token::new("abcxyz".to_string(), None).tokenize();
        assert_eq!(&format!("{:?}",tok.tok_data), "[TokenData { \
            tok_type: Identifier, start: 0, end: 6, line: 1, \
            identifier: \"abcxyz\", value: Object(\"abcxyz\"), \
            sub_tok: None }, TokenData { tok_type: EOF, start: 0, \
            end: 6, line: 1, identifier: \"\", value: Null, sub_tok: None }]\
            ");
    }
}
