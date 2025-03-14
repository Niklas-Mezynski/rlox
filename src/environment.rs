use std::collections::HashMap;

use crate::{
    interpreter::{LoxValue, RuntimeError},
    token::Token,
};

pub struct Environment {
    pub values: HashMap<String, LoxValue>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: LoxValue) {
        self.values.insert(name, value);
    }

    pub fn get(&mut self, name: &Token) -> Result<&mut LoxValue, RuntimeError> {
        if self.values.contains_key(&name.lexeme) {
            return Ok(self
                .values
                .get_mut(&name.lexeme)
                .expect("Value must be present, key was checked"));
        }

        Err(RuntimeError {
            token: name.clone(),
            message: format!("Undefined variable '{}'.", name.lexeme),
        })
    }
}
