use crate::{expr::Expr, token::Token};

#[derive(Debug)]
pub enum Stmt {
    Block {
        statements: Vec<Stmt>,
    },
    Expression {
        expr: Expr,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Print {
        expr: Expr,
    },
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
}
