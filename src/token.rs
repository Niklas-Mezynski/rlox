use crate::token_type::TokenType;

#[derive(Debug, PartialEq)]
pub enum Literal {
    String(String),
    Number(f64),
    Nil,
    Boolean(bool),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize) -> Self {
        Token {
            token_type,
            lexeme,
            line,
        }
    }
}
