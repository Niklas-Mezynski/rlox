use std::{fmt::Debug, rc::Rc};

use crate::{
    interpreter::{LoxValue, RuntimeError, Stringifyable},
    token::Token,
};

#[derive(PartialEq, Debug)]
pub enum LoxCallable {
    ClockFunction,
}

impl LoxCallable {
    fn arity(&self) -> usize {
        match self {
            LoxCallable::ClockFunction => 0,
        }
    }

    pub fn call(
        &self,
        arguments: Vec<Rc<LoxValue>>,
        call_token: &Token,
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
        }
    }
}

impl Stringifyable for LoxCallable {
    fn stringify(&self) -> String {
        match self {
            LoxCallable::ClockFunction => "<native fn>".to_string(),
        }
    }
}
