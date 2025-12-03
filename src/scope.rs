use crate::{
    builtin, parser, tokenizer,
    value::{BuiltIn, Procedure, Value},
};
use std::{cell::RefCell, collections::HashMap, f64::consts::PI, rc::Rc};

#[derive(Debug)]
pub struct Scope {
    frame: Rc<RefCell<HashMap<String, Value>>>,
    outer: Option<Rc<Scope>>,
}

fn add_procedure(name: &str, param_names: Vec<String>, src: &str, scope: &Rc<Scope>) {
    let tokens = tokenizer::tokenize(src).unwrap();
    let body = Rc::new(parser::parse(tokens.into_iter()).unwrap());
    let procedure = Value::Procedure(Procedure::new(param_names, body, scope.clone()));
    Scope::define(scope, name, procedure);
}

impl Scope {
    pub fn global() -> Rc<Self> {
        let mut frame = HashMap::new();
        frame.insert("+".into(), Value::BuiltIn(BuiltIn::new(builtin::op_add)));
        frame.insert("-".into(), Value::BuiltIn(BuiltIn::new(builtin::op_sub)));
        frame.insert("*".into(), Value::BuiltIn(BuiltIn::new(builtin::op_mul)));
        frame.insert("/".into(), Value::BuiltIn(BuiltIn::new(builtin::op_div)));
        frame.insert("and".into(), Value::BuiltIn(BuiltIn::new(builtin::and)));
        frame.insert("or".into(), Value::BuiltIn(BuiltIn::new(builtin::or)));
        frame.insert("not".into(), Value::BuiltIn(BuiltIn::new(builtin::not)));
        frame.insert("mod".into(), Value::BuiltIn(BuiltIn::new(builtin::modulo)));
        frame.insert("=".into(), Value::BuiltIn(BuiltIn::new(builtin::op_eq)));
        frame.insert(
            "<=".into(),
            Value::BuiltIn(BuiltIn::new(|params| {
                builtin::pairwise_compare(&params, |(a, b)| a <= b)
            })),
        );
        frame.insert(
            "<".into(),
            Value::BuiltIn(BuiltIn::new(|params| {
                builtin::pairwise_compare(&params, |(a, b)| a < b)
            })),
        );
        frame.insert(
            ">=".into(),
            Value::BuiltIn(BuiltIn::new(|params| {
                builtin::pairwise_compare(&params, |(a, b)| a >= b)
            })),
        );
        frame.insert(
            ">".into(),
            Value::BuiltIn(BuiltIn::new(|params| {
                builtin::pairwise_compare(&params, |(a, b)| a > b)
            })),
        );
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
        frame.insert("apply".into(), Value::BuiltIn(BuiltIn::new(builtin::apply)));
        frame.insert(
            "read-file".into(),
            Value::BuiltIn(BuiltIn::new(builtin::read_file)),
        );
        frame.insert(
            "split-string".into(),
            Value::BuiltIn(BuiltIn::new(builtin::split_string)),
        );
        frame.insert(
            "split-string-with".into(),
            Value::BuiltIn(BuiltIn::new(builtin::split_string_with)),
        );
        frame.insert(
            "substring".into(),
            Value::BuiltIn(BuiltIn::new(builtin::substring)),
        );
        frame.insert(
            "string->int".into(),
            Value::BuiltIn(BuiltIn::new(builtin::parse_int)),
        );
        frame.insert(
            "->string".into(),
            Value::BuiltIn(BuiltIn::new(builtin::to_string)),
        );
        frame.insert(
            "length".into(),
            Value::BuiltIn(BuiltIn::new(builtin::length)),
        );
        frame.insert("pi".into(), Value::Float(PI));

        let scope = Rc::new(Self {
            frame: Rc::new(RefCell::new(frame)),
            outer: None,
        });

        let src = "
        (begin (define val start) (lambda ()
            (begin
                (define result val)
                (set! val (+ val step))
                result)))";
        add_procedure(
            "make-generator",
            vec!["start".to_string(), "step".to_string()],
            src,
            &scope,
        );

        scope
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
