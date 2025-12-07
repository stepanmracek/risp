use crate::{
    builtin,
    eval::RuntimeError,
    parser, tokenizer,
    value::{BuiltIn, Procedure, Value},
};
use std::{cell::RefCell, collections::HashMap, f64::consts::PI, rc::Rc};

#[derive(Debug)]
pub struct Scope {
    frame: Rc<RefCell<HashMap<String, Value>>>,
    outer: Option<Rc<Scope>>,
}

fn add_procedure(name: &str, param_names: Vec<String>, src: &str, scope: &Rc<Scope>) {
    // This is internal method only for adding built-in procedures that expects
    // correct syntax. Therefore we can unwrap results of both tokenization and parsing.
    let tokens = tokenizer::tokenize(src).unwrap();
    let body = Rc::new(parser::parse(tokens.into_iter()).unwrap());
    let procedure = Value::Procedure(Procedure::new(param_names, vec![body], scope.clone()));
    Scope::define(scope, name, procedure);
}

fn add_built_in(
    frame: &mut HashMap<String, Value>,
    symbol: &str,
    func: fn(Vec<Value>) -> Result<Value, RuntimeError>,
) {
    frame.insert(symbol.to_string(), Value::BuiltIn(BuiltIn::new(func)));
}

impl Scope {
    pub fn global() -> Rc<Self> {
        let mut frame = HashMap::new();
        add_built_in(&mut frame, "+", builtin::op_add);
        add_built_in(&mut frame, "-", builtin::op_sub);
        add_built_in(&mut frame, "*", builtin::op_mul);
        add_built_in(&mut frame, "/", builtin::op_div);
        add_built_in(&mut frame, "and", builtin::and);
        add_built_in(&mut frame, "or", builtin::or);
        add_built_in(&mut frame, "not", builtin::not);
        add_built_in(&mut frame, "mod", builtin::modulo);
        add_built_in(&mut frame, "=", builtin::op_eq);

        add_built_in(&mut frame, "<=", |params| {
            builtin::pairwise_compare(&params, |(a, b)| a <= b)
        });
        add_built_in(&mut frame, "<", |params| {
            builtin::pairwise_compare(&params, |(a, b)| a < b)
        });
        add_built_in(&mut frame, ">=", |params| {
            builtin::pairwise_compare(&params, |(a, b)| a >= b)
        });
        add_built_in(&mut frame, ">", |params| {
            builtin::pairwise_compare(&params, |(a, b)| a > b)
        });

        add_built_in(&mut frame, "list", builtin::list);
        add_built_in(&mut frame, "iota", builtin::iota);
        add_built_in(&mut frame, "zip", builtin::zip);
        add_built_in(&mut frame, "append", builtin::append);
        add_built_in(&mut frame, "string-concatenate", builtin::string_concat);
        add_built_in(&mut frame, "display", builtin::display);
        add_built_in(&mut frame, "map", builtin::map);
        add_built_in(&mut frame, "apply", builtin::apply);
        add_built_in(&mut frame, "read-file", builtin::read_file);
        add_built_in(&mut frame, "split-string", builtin::split_string);
        add_built_in(&mut frame, "split-string-with", builtin::split_string_with);
        add_built_in(&mut frame, "substring", builtin::substring);
        add_built_in(&mut frame, "string-ref", builtin::string_ref);
        add_built_in(&mut frame, "string->int", builtin::parse_int);
        add_built_in(&mut frame, "->string", builtin::to_string);
        add_built_in(&mut frame, "length", builtin::length);
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
