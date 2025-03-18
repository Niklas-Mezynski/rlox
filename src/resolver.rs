use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{error, expr::Expr, stmt::Stmt, token::Token};

#[derive(PartialEq)]
enum FunctionType {
    None,
    Function,
}

pub struct Resolver {
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
}

impl Resolver {
    pub fn new() -> Resolver {
        Resolver {
            scopes: vec![],
            current_function: FunctionType::None,
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn is_empty(&self) -> bool {
        self.scopes.is_empty()
    }

    fn peek_mut(&mut self) -> &mut HashMap<String, bool> {
        self.scopes
            .last_mut()
            .expect("Scope stack was checked to be non-empty")
    }

    fn peek(&self) -> &HashMap<String, bool> {
        self.scopes
            .last()
            .expect("Scope stack was checked to be non-empty")
    }

    fn declare(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        let scope = self.peek_mut();

        if scope.contains_key(&name.lexeme) {
            error::error_token(name, "Already a variable with this name in this scope.");
        }

        scope.insert(name.lexeme.to_owned(), false);
    }

    fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        self.peek_mut().insert(name.lexeme.to_owned(), true);
    }

    fn resolve_local(&mut self, name: &Token) -> Option<usize> {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                return Some(i);
            }
        }
        None
    }

    fn resolve_function(
        &mut self,
        params: &[Token],
        body: &Rc<RefCell<Vec<Stmt>>>,
        function_type: FunctionType,
    ) {
        let enclosing_function = std::mem::replace(&mut self.current_function, function_type);

        self.begin_scope();
        for param in params {
            self.declare(param);
            self.define(param);
        }
        body.borrow_mut().resolve(self);
        self.end_scope();

        self.current_function = enclosing_function;
    }
}

pub trait Resolvable<T> {
    // fn resolve(self, environment: Rc<RefCell<Environment>>) -> Result<T, RuntimeEvent>;
    fn resolve(self, resolver: &mut Resolver) -> T;
}

impl Resolvable<()> for &mut Stmt {
    fn resolve(self, resolver: &mut Resolver) {
        match self {
            Stmt::Block { statements } => {
                resolver.begin_scope();
                statements.resolve(resolver);
                resolver.end_scope();
            }
            Stmt::Var { name, initializer } => {
                resolver.declare(name);
                if let Some(initializer) = initializer {
                    initializer.resolve(resolver);
                }
                resolver.define(name);
            }
            Stmt::Function { name, params, body } => {
                resolver.declare(name);
                resolver.define(name);

                // resolveFunction
                resolver.resolve_function(params, body, FunctionType::Function);
            }
            Stmt::Expression { expr } => {
                expr.resolve(resolver);
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                condition.resolve(resolver);
                then_branch.resolve(resolver);
                if let Some(else_branch) = else_branch {
                    else_branch.resolve(resolver);
                }
            }
            Stmt::Print { expr } => {
                expr.resolve(resolver);
            }
            Stmt::Return { keyword, value } => {
                if resolver.current_function == FunctionType::None {
                    error::error_token(keyword, "Can't return from top-level code.");
                }

                if let Some(value) = value {
                    value.resolve(resolver);
                }
            }
            Stmt::While { condition, body } => {
                condition.resolve(resolver);
                body.resolve(resolver);
            }
            Stmt::Class { name, methods } => {
                resolver.declare(name);
                resolver.define(name);
                // TODO: resolve methods
            }
        }
    }
}

impl Resolvable<()> for &mut Vec<Stmt> {
    fn resolve(self, resolver: &mut Resolver) {
        for statement in self {
            statement.resolve(resolver);
        }
    }
}

impl Resolvable<()> for &mut Expr {
    fn resolve(self, resolver: &mut Resolver) {
        match self {
            Expr::Variable { name, depth } => {
                if !resolver.is_empty() && resolver.peek().get(&name.lexeme) == Some(&false) {
                    error::error_token(name, "Can't read local variable in its own initializer.");
                }

                *depth = resolver.resolve_local(name);
            }
            Expr::Assign { name, value, depth } => {
                value.resolve(resolver);
                *depth = resolver.resolve_local(name);
            }
            Expr::Binary {
                left,
                operator: _,
                right,
            } => {
                left.resolve(resolver);
                right.resolve(resolver);
            }
            Expr::Call {
                callee,
                closing_paren: _,
                arguments,
            } => {
                callee.resolve(resolver);
                for argument in arguments {
                    argument.resolve(resolver);
                }
            }
            Expr::Grouping { expression } => {
                expression.resolve(resolver);
            }
            Expr::Literal { value: _ } => {}
            Expr::Logical {
                left,
                operator: _,
                right,
            } => {
                left.resolve(resolver);
                right.resolve(resolver);
            }
            Expr::Unary { operator: _, right } => {
                right.resolve(resolver);
            }
            Expr::Conditional {
                condition,
                then,
                r#else,
            } => {
                condition.resolve(resolver);
                then.resolve(resolver);
                r#else.resolve(resolver);
            }
        }
    }
}
