use std::collections::VecDeque;

use crate::{
    expr::Expr,
    token::{Literal, Token},
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
        let mut expr = self.term();

        while let Some(operator) = self.match_tokens(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let right = self.term();
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.comparison();

        while let Some(operator) = self.match_tokens(vec![TokenType::Minus, TokenType::Plus]) {
            let right = self.comparison();
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.comparison();

        while let Some(operator) = self.match_tokens(vec![TokenType::Slash, TokenType::Star]) {
            let right = self.comparison();
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        expr
    }

    fn unary(&mut self) -> Expr {
        if let Some(operator) = self.match_tokens(vec![TokenType::Bang, TokenType::Minus]) {
            let right = self.unary();
            return Expr::Unary {
                operator,
                right: Box::new(right),
            };
        }

        self.primary()
    }

    fn primary(&mut self) -> Expr {
        if let Some(operator) = self.match_token(TokenType::False) {
            return Expr::Literal {
                value: Literal::Boolean(false),
            };
        }
        if let Some(operator) = self.match_token(TokenType::True) {
            return Expr::Literal {
                value: Literal::Boolean(true),
            };
        }
        if let Some(operator) = self.match_token(TokenType::Nil) {
            return Expr::Literal {
                value: Literal::Nil,
            };
        }
        if let Some(num) = self.match_number() {
            return Expr::Literal {
                value: Literal::Number(num),
            };
        }
        if let Some(string) = self.match_string() {
            return Expr::Literal {
                value: Literal::String(string),
            };
        }

        if let Some(operator) = self.match_token(TokenType::LeftParen) {
            let expr = self.expression();
            self.consume(TokenType::RightParen, "Expect ')' after expression.");

            return Expr::Grouping {
                expression: Box::new(expr),
            };
        }

        todo!()
    }

    fn consume(&mut self, t: TokenType, error_msg: &str) {
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

    fn match_number(&mut self) -> Option<f64> {
        if self.pre_check() && matches!(self.peek().token_type, TokenType::Number(_)) {
            let token = self.advance();
            if let TokenType::Number(num) = token.token_type {
                return Some(num);
            }
            unreachable!()
        }

        None
    }

    fn match_string(&mut self) -> Option<String> {
        if self.pre_check() && matches!(self.peek().token_type, TokenType::String(_)) {
            let token = self.advance();
            if let TokenType::String(string) = token.token_type {
                return Some(string);
            }
            unreachable!()
        }

        None
    }

    fn check(&self, t: TokenType) -> bool {
        self.pre_check() && self.peek().token_type == t
    }

    fn pre_check(&self) -> bool {
        !self.is_at_end()
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
