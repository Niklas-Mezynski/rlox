use std::collections::VecDeque;

use crate::{
    expr::Expr,
    token::Token,
    token_type::{self, TokenType},
};

pub struct Parser {
    tokens: VecDeque<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: VecDeque::from(tokens),
            current: 0,
        }
    }

    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();

        while let Some(operator) = self.match_tokens(vec![TokenType::BangEqual, TokenType::Equal]) {
            let right = self.comparison();
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        expr
    }

    fn comparison(&mut self) -> Expr {
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

    fn check(&self, t: TokenType) -> bool {
        !self.is_at_end() && self.peek().token_type == t
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
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
