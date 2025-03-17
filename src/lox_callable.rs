use std::{fmt::Debug, rc::Rc};

use crate::interpreter::{LoxValue, RuntimeError};

#[derive(Clone, PartialEq, Debug)]
pub enum LoxCallable {
    Function(Box<dyn LoxCallableTrait>),
}

pub trait LoxCallableTrait: Clone + PartialEq + Debug {
    fn call(&self, arguments: Vec<Rc<LoxValue>>) -> Result<Rc<LoxValue>, RuntimeError>;

    fn arity(&self) -> usize;
}

pub struct ClockFunction;

impl Debug for ClockFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn \"clock\">")
    }
}

impl LoxCallableTrait for ClockFunction {
    fn call(&self, arguments: Vec<Rc<LoxValue>>) -> Result<Rc<LoxValue>, RuntimeError> {
        todo!()
    }

    fn arity(&self) -> usize {
        todo!()
    }
}
