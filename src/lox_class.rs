use std::{collections::HashMap, rc::Rc};

use crate::{interpreter::LoxValue, lox_callable::LoxCallable};

#[derive(Debug)]
pub struct LoxClass {
    pub name: String,
    superclass: Option<Rc<LoxValue>>,
    methods: HashMap<String, Rc<LoxValue>>,
}

impl LoxClass {
    pub fn new(
        name: String,
        superclass: Option<Rc<LoxValue>>,
        methods: HashMap<String, Rc<LoxValue>>,
    ) -> LoxClass {
        LoxClass {
            name,
            superclass,
            methods,
        }
    }

    pub fn find_method(&self, name: &str) -> Option<Rc<LoxValue>> {
        match self.methods.get(name).map(Rc::clone) {
            None => {
                if let Some(superclass) = &self.superclass {
                    return match superclass.as_ref() {
                        LoxValue::Callable(LoxCallable::Class { class }) => class.find_method(name),
                        _ => panic!("Superclass must be a LoxClass"),
                    };
                }
                None
            }
            some => some,
        }
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
