use crate::token::{Literal, Token};

#[derive(Debug)]
pub enum Expr {
    Conditional {
        condition: Box<Expr>,
        then: Box<Expr>,
        r#else: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: Literal,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}
