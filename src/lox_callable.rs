use std::{cell::RefCell, collections::VecDeque, fmt::Debug, rc::Rc};

use crate::{
    environment::Environment,
    interpreter::{Evaluatable, LoxValue, RuntimeError, RuntimeEvent, Stringifyable},
    lox_class::LoxClass,
    lox_instance::LoxInstance,
    stmt::Stmt,
    token::Token,
    token_type::TokenType,
};

#[derive(Debug)]
pub struct FunctionStmt {
    pub name: Rc<Token>,
    pub params: Rc<Vec<Token>>,
    pub body: Rc<RefCell<Vec<Stmt>>>,
}

#[derive(Debug)]
pub enum LoxCallable {
    ClockFunction,
    Function {
        declaration: Rc<FunctionStmt>,
        closure: Rc<RefCell<Environment>>,
        is_initializer: bool,
    },
    Class {
        class: Rc<LoxClass>,
    },
}

impl LoxCallable {
    pub fn new_function(
        declaration: Rc<FunctionStmt>,
        closure: Rc<RefCell<Environment>>,
        is_initializer: bool,
    ) -> LoxCallable {
        LoxCallable::Function {
            declaration,
            closure,
            is_initializer,
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            LoxCallable::ClockFunction => 0,
            LoxCallable::Function {
                declaration,
                closure: _,
                is_initializer: _,
            } => declaration.params.len(),
            LoxCallable::Class { class } => class.arity(),
        }
    }

    pub fn call(
        &self,
        mut arguments: VecDeque<Rc<LoxValue>>,
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
                is_initializer,
            } => {
                let mut function_env = Environment::new_enclosing(closure.clone());

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
                    .borrow()
                    .evaluate(Rc::new(RefCell::new(function_env)));

                match result {
                    Ok(_) => match is_initializer {
                        // init() methods should always return this
                        true => closure.borrow().get_at(
                            Some(0),
                            &Token::new(TokenType::This, "this".to_string(), call_token.line),
                        ),
                        false => Ok(Rc::new(LoxValue::Nil)),
                    },
                    Err(err) => match err {
                        RuntimeEvent::Return(value) => {
                            match is_initializer {
                                // Handle case where have an early return in an initializer function
                                true => closure.borrow().get_at(
                                    Some(0),
                                    &Token::new(
                                        TokenType::This,
                                        "this".to_string(),
                                        call_token.line,
                                    ),
                                ),
                                false => Ok(value),
                            }
                        }
                        other => Err(other),
                    },
                }
            }
            LoxCallable::Class { class } => {
                let instance = LoxInstance::new(class.clone());
                let instance = Rc::new(RefCell::new(instance));

                if let Some(initializer) = class.find_method("init") {
                    // First bind the init method to the instance (so it has access to `this`)
                    match initializer.bind(instance.clone()).as_ref() {
                        LoxValue::Callable(callable) => {
                            // And then invoke it
                            callable.call(arguments, call_token)?;
                        }
                        _ => unreachable!("Bind always returns a callable"),
                    };
                }

                Ok(Rc::new(LoxValue::Instance(instance)))
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
                is_initializer: _,
            } => format!("<fn {}>", declaration.name.lexeme),
            LoxCallable::Class { class } => class.name.to_string(),
        }
    }
}
