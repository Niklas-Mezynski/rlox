use std::collections::VecDeque;

use crate::expr::Expr;

#[derive(Debug, PartialEq, Clone)]
enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,

    // ...

    // Literals.
    Identifier(String),
    String(String),
    Number(f64),
    // ...
}

struct Token {
    token_type: TokenType,
    lexeme: String,
    line: usize,
}

pub struct Parser {
    tokens: VecDeque<Token>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: VecDeque::from(tokens),
        }
    }

    fn primary(&mut self) -> Expr {
        if let Some(operator) = self.match_token(TokenType::Number(??)) {
            // Help: Can I somehow immediately get the number value from within the token enum?
            let num = 0; // ??
            return Expr::Literal {
                value: Literal::Number(num),
            };
        }

        todo!()
    }

    fn match_tokens(&mut self, types: Vec<TokenType>) -> Option<Token> {
        for t in types {
            if self.check(t) {
                let token = self.advance();
                return Some(token);
            }
        }

        None
    }

    fn match_token(&mut self, t: TokenType) -> Option<Token> {
        if self.check(t) {
            let token = self.advance();
            return Some(token);
        }

        None
    }

    fn check(&self, t: TokenType) -> bool {
        !self.is_at_end() && self.peek().token_type == t
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            return self.peek().clone();
        }

        self.tokens.pop_front().unwrap()
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().token_type, TokenType::Eof)
    }

    fn peek(&self) -> &Token {
        &self.tokens.front().unwrap()
    }
}
