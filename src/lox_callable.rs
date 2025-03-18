use std::{cell::RefCell, collections::VecDeque, fmt::Debug, rc::Rc};

use crate::{
    environment::Environment,
    interpreter::{Evaluatable, LoxValue, RuntimeError, Stringifyable},
    stmt::Stmt,
    token::Token,
};

#[derive(Debug)]
pub struct FunctionStmt {
    pub name: Rc<Token>,
    pub params: Rc<Vec<Token>>,
    pub body: Rc<Vec<Stmt>>,
}

#[derive(Debug)]
pub enum LoxCallable {
    ClockFunction,
    Function(FunctionStmt),
}

impl LoxCallable {
    fn arity(&self) -> usize {
        match self {
            LoxCallable::ClockFunction => 0,
            LoxCallable::Function(declaration) => declaration.params.len(),
        }
    }

    pub fn call(
        &self,
        arguments: Vec<Rc<LoxValue>>,
        call_token: &Token,
        environment: Rc<RefCell<Environment>>,
    ) -> Result<Rc<LoxValue>, RuntimeError> {
        if self.arity() != arguments.len() {
            return Err(RuntimeError::new(
                call_token.to_owned(),
                format!(
                    "Expected {} arguments but got {}.",
                    self.arity(),
                    arguments.len()
                ),
            ));
        }

        match self {
            LoxCallable::ClockFunction => {
                let now = std::time::SystemTime::now();
                let duration = now
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Time went backwards");
                Ok(Rc::new(LoxValue::Number(duration.as_secs_f64())))
            }
            LoxCallable::Function(declaration) => {
                let mut function_env = Environment::new_enclosing(environment);
                let mut arguments = VecDeque::from(arguments);

                for param in declaration.params.iter() {
                    function_env.define(
                        param.lexeme.to_owned(),
                        arguments
                            .pop_front()
                            .expect("Argument list was checked with arity of the function"),
                    );
                }

                declaration
                    .body
                    .evaluate(Rc::new(RefCell::new(function_env)))?;

                Ok(Rc::new(LoxValue::Nil))
            }
        }
    }
}

impl Stringifyable for LoxCallable {
    fn stringify(&self) -> String {
        match self {
            LoxCallable::ClockFunction => "<native fn>".to_string(),
            LoxCallable::Function(declaration) => format!("<fn {}>", declaration.name.lexeme),
        }
    }
}
