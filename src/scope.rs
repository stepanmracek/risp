use crate::{
    builtin,
    value::{BuiltIn, Value},
};
use std::{cell::RefCell, collections::HashMap, f64::consts::PI, rc::Rc};

#[derive(Debug)]
pub struct Scope {
    frame: Rc<RefCell<HashMap<String, Value>>>,
    outer: Option<Rc<Scope>>,
}

impl Scope {
    pub fn global() -> Rc<Self> {
        let mut frame = HashMap::new();
        frame.insert("+".into(), Value::BuiltIn(BuiltIn::new(builtin::op_add)));
        frame.insert("-".into(), Value::BuiltIn(BuiltIn::new(builtin::op_sub)));
        frame.insert("*".into(), Value::BuiltIn(BuiltIn::new(builtin::op_mul)));
        frame.insert("/".into(), Value::BuiltIn(BuiltIn::new(builtin::op_div)));
        frame.insert("=".into(), Value::BuiltIn(BuiltIn::new(builtin::op_eq)));
        frame.insert("<=".into(), Value::BuiltIn(BuiltIn::new(builtin::op_leq)));
        frame.insert("list".into(), Value::BuiltIn(BuiltIn::new(builtin::list)));
        frame.insert(
            "string-concatenate".into(),
            Value::BuiltIn(BuiltIn::new(builtin::string_concat)),
        );
        frame.insert(
            "display".into(),
            Value::BuiltIn(BuiltIn::new(builtin::display)),
        );
        frame.insert("map".into(), Value::BuiltIn(BuiltIn::new(builtin::map)));
        frame.insert("pi".into(), Value::Float(PI));

        Rc::new(Self {
            frame: Rc::new(RefCell::new(frame)),
            outer: None,
        })
    }

    pub fn nest(parent: &Rc<Self>) -> Rc<Self> {
        Rc::new(Self {
            frame: Rc::new(RefCell::new(HashMap::new())),
            outer: Some(parent.clone()),
        })
    }

    pub fn define(scope: &Rc<Self>, name: &str, value: Value) {
        scope.frame.borrow_mut().insert(name.to_string(), value);
    }

    pub fn get(scope: &Rc<Self>, name: &str) -> Option<Value> {
        if let Some(val) = scope.frame.borrow().get(name) {
            return Some(val.clone());
        }
        if let Some(parent) = &scope.outer {
            return Self::get(parent, name);
        }
        None
    }

    pub fn set(scope: &Rc<Self>, name: &str, value: Value) -> Result<(), ()> {
        if scope.frame.borrow().contains_key(name) {
            scope.frame.borrow_mut().insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &scope.outer {
            Self::set(parent, name, value)
        } else {
            Err(())
        }
    }

    pub fn variables(&self) -> Vec<String> {
        self.frame.borrow().keys().cloned().collect()
    }
}
