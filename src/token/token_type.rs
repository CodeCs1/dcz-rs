
#[derive(Debug, PartialEq, Clone,Eq)]
pub enum TokenType {
    Identifier,
    Keywords,
    DataType,
    Number,
    Macro,
    String,
    Char,

    Plus,
    Minus,
    Star,
    Slash,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Modulo,
    Equal,
    Semicolon,
    And,
    Or,
    Not,
    Comma,
    Colon,
    NotEqual,
    EqualEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    ShiftLeft,
    ShiftRight,
    AndBool,
    OrBool,
    PointTo,

    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,
    PlusPlus,
    MinusMinus,

    EOF
}

impl TokenType {
    pub fn from(c: &[char]) -> Self {
        match c {
            ['-', '>'] => TokenType::PointTo,
            ['-', '='] => TokenType::MinusEqual,
            ['-', ' '] | ['-'] => TokenType::Minus,

            ['+', '='] => TokenType::PlusEqual,
            ['+', '+'] => TokenType::PlusPlus,
            ['+', ' '] | ['+'] => TokenType::Plus,

            ['*', '='] => TokenType::StarEqual,
            ['*', ' '] | ['*'] => TokenType::Star,

            ['/', '='] => TokenType::SlashEqual,
            ['/', ' '] | ['/'] => TokenType::Slash,

            ['=', '='] => TokenType::EqualEqual,
            ['=', ' '] | ['='] => TokenType::Equal,

            ['!', '='] => TokenType::NotEqual,
            ['!', ' '] | ['!'] => TokenType::Not,

            ['>', '='] => TokenType::GreaterEqual,
            ['>', ' '] | ['>'] => TokenType::Greater,
            ['>', '>'] => TokenType::ShiftRight,

            ['<', '='] => TokenType::LessEqual,
            ['<', ' '] | ['<'] => TokenType::Less,
            ['<', '<'] => TokenType::ShiftLeft,
            ['&', '&'] => TokenType::AndBool,
            ['&',  ' '] | ['&'] => TokenType::And,
            ['|', '|'] => TokenType::OrBool,
            ['|',  ' '] | ['|'] => TokenType::Or,
            _ => todo!()
        }
    }
}
