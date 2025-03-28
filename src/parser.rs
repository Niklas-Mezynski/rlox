use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use crate::{
    error,
    expr::Expr,
    stmt::Stmt,
    token::{Literal, Token},
    token_type::TokenType,
};

pub struct Parser {
    tokens: VecDeque<Token>,
    current: usize,
}

#[derive(Debug)]
struct ParseError;

fn error<T>(token: &Token, message: &str) -> Result<T, ParseError> {
    error::error_token(token, message);
    Err(ParseError)
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: VecDeque::from(tokens),
            current: 0,
        }
    }

    pub fn parse(mut self) -> Option<Vec<Stmt>> {
        let mut statements = vec![];
        let mut has_errored = false;

        while !self.is_at_end() {
            match self.declaration() {
                Some(stmt) => {
                    statements.push(stmt);
                }
                None => {
                    // If any statement was not parsable, we don't want to return an AST
                    has_errored = true;
                }
            };
        }

        match has_errored {
            true => None,
            false => Some(statements),
        }
    }

    fn declaration(&mut self) -> Option<Stmt> {
        match self.declaration_impl() {
            Ok(val) => Some(val),
            Err(_) => {
                self.synchronize();
                None
            }
        }
    }

    fn declaration_impl(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(TokenType::Class).is_some() {
            return self.class_declaration();
        }
        if self.match_token(TokenType::Fun).is_some() {
            return self.function("function");
        }
        if self.match_token(TokenType::Var).is_some() {
            return self.var_declaration();
        }
        self.statement()
    }

    fn class_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::Identifier, "Expect class name.")?;

        let superclass = match self.match_token(TokenType::Less) {
            Some(_token) => Some(Expr::Variable {
                name: self.consume(TokenType::Identifier, "Expect superclass name.")?,
                depth: None,
            }),
            None => None,
        };

        self.consume(TokenType::LeftBrace, "Expect '{' before class body.")?;

        let mut methods = vec![];
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            methods.push(self.function("method")?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after class body.")?;

        Ok(Stmt::Class {
            name,
            methods,
            superclass,
        })
    }

    fn function(&mut self, kind: &str) -> Result<Stmt, ParseError> {
        let name = self.consume(
            TokenType::Identifier,
            format!("Expected {} name", kind).as_str(),
        )?;
        self.consume(
            TokenType::LeftParen,
            format!("Expect '(' after {} name.", kind).as_str(),
        )?;

        let mut parameters = vec![];
        if !self.check(TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    let _ = error::<Expr>(self.peek(), "Can't have more than 255 parameters.");
                }

                parameters.push(self.consume(TokenType::Identifier, "Expect parameter name.")?);

                if self.match_token(TokenType::Comma).is_none() {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;

        self.consume(
            TokenType::LeftBrace,
            format!("Expect '{{' before {} body.", kind).as_str(),
        )?;

        let body = self.block()?;

        Ok(Stmt::Function {
            name: Rc::new(name),
            params: Rc::new(parameters),
            body: Rc::new(RefCell::new(body)),
        })
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;

        // self.consume(TokenType::Equal, "Expect a variable assignment ('=').")?;
        let mut initializer = None;
        if self.match_token(TokenType::Equal).is_some() {
            initializer = Some(self.expression()?);
        }

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;

        Ok(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(TokenType::For).is_some() {
            return self.for_statement();
        }
        if self.match_token(TokenType::If).is_some() {
            return self.if_statement();
        }
        if self.match_token(TokenType::Print).is_some() {
            return self.print_statement();
        }
        if let Some(keyword) = self.match_token(TokenType::Return) {
            return self.return_statement(keyword);
        }
        if self.match_token(TokenType::While).is_some() {
            return self.while_statement();
        }
        if self.match_token(TokenType::LeftBrace).is_some() {
            return Ok(Stmt::Block {
                statements: self.block()?,
            });
        }

        self.expression_statement()
    }

    fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;

        // initializer
        let initializer = if self.match_token(TokenType::Semicolon).is_some() {
            None
        } else if self.match_token(TokenType::Var).is_some() {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        // condition
        let condition = match self.check(TokenType::Semicolon) {
            false => self.expression()?,
            true => Expr::Literal {
                value: Literal::Boolean(true),
            },
        };
        self.consume(TokenType::Semicolon, "Expect ';' after loop condition.")?;

        // increment
        let mut increment = None;
        if !self.check(TokenType::RightParen) {
            increment = Some(self.expression()?);
        }
        self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;

        // body
        let mut body = self.statement()?;

        // desugaring for loop into known statements

        if let Some(increment) = increment {
            body = Stmt::Block {
                statements: vec![body, Stmt::Expression { expr: increment }],
            }
        }

        body = Stmt::While {
            condition,
            body: Box::new(body),
        };

        if let Some(initializer) = initializer {
            body = Stmt::Block {
                statements: vec![initializer, body],
            }
        }

        Ok(body)
    }

    fn return_statement(&mut self, keyword: Token) -> Result<Stmt, ParseError> {
        let value = match self.check(TokenType::Semicolon) {
            true => None,
            false => Some(self.expression()?),
        };

        self.consume(TokenType::Semicolon, "Expect ';' after return value.")?;

        Ok(Stmt::Return { keyword, value })
    }

    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after while condition.")?;
        let body = self.statement()?;

        Ok(Stmt::While {
            condition,
            body: Box::new(body),
        })
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        let then_branch = self.statement()?;
        let mut else_branch = None;

        if self.match_token(TokenType::Else).is_some() {
            else_branch = Some(Box::new(self.statement()?));
        }

        Ok(Stmt::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = vec![];

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            // If something goes wrong, we don't care about returning a valid AST
            if let Some(declaration) = self.declaration() {
                statements.push(declaration);
            }
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;

        Ok(statements)
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print { expr: value })
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression { expr })
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.ternary()?;

        if let Some(equals) = self.match_token(TokenType::Equal) {
            let value = self.assignment()?;

            match expr {
                Expr::Variable { name, depth: None } => {
                    return Ok(Expr::Assign {
                        name,
                        value: Box::new(value),
                        depth: None,
                    })
                }
                Expr::Get { object, name } => {
                    return Ok(Expr::Set {
                        object,
                        name,
                        value: Box::new(value),
                    })
                }
                _ => return error(&equals, "Invalid assignment target."),
            }
        }

        Ok(expr)
    }

    fn ternary(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.or()?;

        while let Some(_operator) = self.match_token(TokenType::QuestionMark) {
            let then = self.ternary()?;

            match self.match_token(TokenType::Colon) {
                Some(_) => {
                    let r#else = self.ternary()?;
                    expr = Expr::Conditional {
                        condition: Box::new(expr),
                        then: Box::new(then),
                        r#else: Box::new(r#else),
                    }
                }
                None => error(self.peek(), "Expected ':' for ternary operation")?,
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.and()?;

        while let Some(operator) = self.match_token(TokenType::Or) {
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;

        while let Some(operator) = self.match_token(TokenType::And) {
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while let Some(operator) =
            self.match_tokens(vec![TokenType::BangEqual, TokenType::EqualEqual])
        {
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        while let Some(operator) = self.match_tokens(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        while let Some(operator) = self.match_tokens(vec![TokenType::Minus, TokenType::Plus]) {
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        while let Some(operator) = self.match_tokens(vec![TokenType::Slash, TokenType::Star]) {
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if let Some(operator) = self.match_tokens(vec![TokenType::Bang, TokenType::Minus]) {
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }

        self.call()
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(TokenType::LeftParen).is_some() {
                expr = self.finish_call(expr)?;
            } else if self.match_token(TokenType::Dot).is_some() {
                let name =
                    self.consume(TokenType::Identifier, "Expect property name after '.'.")?;
                expr = Expr::Get {
                    object: Box::new(expr),
                    name,
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        let mut arguments = vec![];

        if !self.check(TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    let _ = error::<Expr>(self.peek(), "Can't have more than 255 arguments.");
                }
                arguments.push(self.expression()?);
                if self.match_token(TokenType::Comma).is_none() {
                    break;
                }
            }
        }

        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;

        Ok(Expr::Call {
            callee: Box::new(callee),
            closing_paren: paren,
            arguments,
        })
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if let Some(_operator) = self.match_token(TokenType::False) {
            return Ok(Expr::Literal {
                value: Literal::Boolean(false),
            });
        }
        if let Some(_operator) = self.match_token(TokenType::True) {
            return Ok(Expr::Literal {
                value: Literal::Boolean(true),
            });
        }
        if let Some(_operator) = self.match_token(TokenType::Nil) {
            return Ok(Expr::Literal {
                value: Literal::Nil,
            });
        }
        if let Some(num) = self.match_number() {
            return Ok(Expr::Literal {
                value: Literal::Number(num),
            });
        }
        if let Some(string) = self.match_string() {
            return Ok(Expr::Literal {
                value: Literal::String(string),
            });
        }

        if let Some(token) = self.match_token(TokenType::Identifier) {
            return Ok(Expr::Variable {
                name: token,
                depth: None,
            });
        }

        if let Some(keyword) = self.match_token(TokenType::Super) {
            self.consume(TokenType::Dot, "Expect '.' after 'super'.")?;
            let method = self.consume(TokenType::Identifier, "Expect superclass method name.")?;
            return Ok(Expr::Super {
                keyword,
                method,
                depth: 0,
            });
        }

        if let Some(token) = self.match_token(TokenType::This) {
            return Ok(Expr::This {
                keyword: token,
                depth: 0,
            });
        }

        if self.match_token(TokenType::LeftParen).is_some() {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;

            return Ok(Expr::Grouping {
                expression: Box::new(expr),
            });
        }

        // unreachable!("At this point the parser must matched a correct primary token")
        error(self.peek(), "Expect expression.")
    }

    // After we hit a parse error, we discard tokens until we can continue parsing (until we encounter a new statement)
    fn synchronize(&mut self) {
        let mut previous = self.advance();

        while !self.is_at_end() {
            if previous.token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    return;
                }
                _ => {}
            }

            previous = self.advance();
        }
    }

    fn consume(&mut self, t: TokenType, error_msg: &str) -> Result<Token, ParseError> {
        if self.check(t) {
            return Ok(self.advance());
        }

        error(self.peek(), error_msg)?
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
            return self.tokens.pop_front().unwrap();
        }

        // Nothing more to consume
        // panic!("Advancing although end is reached");
        self.peek().clone()
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().token_type, TokenType::Eof)
    }

    fn peek(&self) -> &Token {
        self.tokens.front().unwrap()
    }
}
