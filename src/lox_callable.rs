use std::{cell::RefCell, collections::VecDeque, fmt::Debug, rc::Rc};

use crate::{
    environment::Environment,
    interpreter::{Evaluatable, LoxValue, RuntimeError, RuntimeEvent, Stringifyable},
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
    Function {
        declaration: FunctionStmt,
        closure: Rc<RefCell<Environment>>,
    },
}

impl LoxCallable {
    pub fn new_function(
        declaration: FunctionStmt,
        closure: Rc<RefCell<Environment>>,
    ) -> LoxCallable {
        LoxCallable::Function {
            declaration,
            closure,
        }
    }

    fn arity(&self) -> usize {
        match self {
            LoxCallable::ClockFunction => 0,
            LoxCallable::Function {
                declaration,
                closure: _,
            } => declaration.params.len(),
        }
    }

    pub fn call(
        &self,
        arguments: Vec<Rc<LoxValue>>,
        call_token: &Token,
    ) -> Result<Rc<LoxValue>, RuntimeEvent> {
        if self.arity() != arguments.len() {
            return Err(RuntimeEvent::Error(RuntimeError::new(
                call_token.to_owned(),
                format!(
                    "Expected {} arguments but got {}.",
                    self.arity(),
                    arguments.len()
                ),
            )));
        }

        match self {
            LoxCallable::ClockFunction => {
                let now = std::time::SystemTime::now();
                let duration = now
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Time went backwards");
                Ok(Rc::new(LoxValue::Number(duration.as_secs_f64())))
            }
            LoxCallable::Function {
                declaration,
                closure,
            } => {
                let mut function_env = Environment::new_enclosing(closure.clone());
                let mut arguments = VecDeque::from(arguments);

                for param in declaration.params.iter() {
                    function_env.define(
                        param.lexeme.to_owned(),
                        arguments
                            .pop_front()
                            .expect("Argument list was checked with arity of the function"),
                    );
                }

                let result = declaration
                    .body
                    .evaluate(Rc::new(RefCell::new(function_env)));

                match result {
                    Ok(_) => Ok(Rc::new(LoxValue::Nil)),
                    Err(err) => match err {
                        RuntimeEvent::Return(value) => Ok(value),
                        other => Err(other),
                    },
                }
            }
        }
    }
}

impl Stringifyable for LoxCallable {
    fn stringify(&self) -> String {
        match self {
            LoxCallable::ClockFunction => "<native fn>".to_string(),
            LoxCallable::Function {
                declaration,
                closure: _,
            } => format!("<fn {}>", declaration.name.lexeme),
        }
    }
}
