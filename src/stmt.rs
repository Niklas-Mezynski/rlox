use std::{cell::RefCell, rc::Rc};

use crate::{expr::Expr, token::Token};

#[derive(Debug)]
pub enum Stmt {
    Block {
        statements: Vec<Stmt>,
    },
    Class {
        name: Token,
        superclass: Option<Expr>,
        methods: Vec<Stmt>, // Where statements must be functions
    },
    Expression {
        expr: Expr,
    },
    Function {
        name: Rc<Token>,
        params: Rc<Vec<Token>>,
        body: Rc<RefCell<Vec<Stmt>>>,
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
    Return {
        keyword: Token,
        value: Option<Expr>,
    },
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
}
