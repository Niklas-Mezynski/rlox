use std::{collections::HashMap, rc::Rc};

use crate::interpreter::LoxValue;

#[derive(Debug)]
pub struct LoxClass {
    pub name: String,
    methods: HashMap<String, Rc<LoxValue>>,
}

impl LoxClass {
    pub fn new(name: String, methods: HashMap<String, Rc<LoxValue>>) -> LoxClass {
        LoxClass { name, methods }
    }

    pub fn find_method(&self, name: &str) -> Option<Rc<LoxValue>> {
        self.methods.get(name).map(Rc::clone)
    }

    pub fn arity(&self) -> usize {
        match self.find_method("init") {
            Some(initializer) => match initializer.as_ref() {
                LoxValue::Callable(callable) => callable.arity(),
                _ => unreachable!("All class methods must be functions"),
            },
            None => 0,
        }
    }
}
