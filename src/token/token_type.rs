
#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Identifier,
    Keywords,
    DataType,
    Number,
    Macro,
    String,

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

    NewLine,


    EOF

}
