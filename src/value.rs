use crate::{
    eval::{RuntimeError, evaluate},
    parser::Expr,
    scope::Scope,
};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Procedure {
    param_names: Vec<String>,
    body: Rc<Expr>,
    scope: Rc<Scope>,
}

impl Procedure {
    pub fn new(param_names: Vec<String>, body: Rc<Expr>, scope: Rc<Scope>) -> Self {
        Self {
            param_names,
            body,
            scope,
        }
    }

    pub fn call(&self, params: Vec<Value>) -> Result<Value, RuntimeError> {
        if self.param_names.len() != params.len() {
            return Err(RuntimeError::WrongNumberOfAgumentsPassed);
        }

        let scope = Scope::nest(&self.scope);
        self.param_names
            .iter()
            .zip(params)
            .for_each(|(name, param)| {
                Scope::define(&scope, name, param);
            });

        evaluate(&self.body, &scope)
    }
}

#[derive(Debug, Clone)]
pub struct BuiltIn {
    func: fn(Vec<Value>) -> Result<Value, RuntimeError>,
}

impl BuiltIn {
    pub fn new(func: fn(Vec<Value>) -> Result<Value, RuntimeError>) -> Self {
        Self { func }
    }

    pub fn call(&self, params: Vec<Value>) -> Result<Value, RuntimeError> {
        (self.func)(params)
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(Rc<String>),
    List(Vec<Value>),
    BuiltIn(BuiltIn),
    Procedure(Procedure),
    Nil,
}

impl Value {
    pub fn truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(b) => *b,
            _ => true,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(v) => std::fmt::Display::fmt(v, f),
            Value::Int(v) => std::fmt::Display::fmt(v, f),
            Value::Float(v) => std::fmt::Display::fmt(v, f),
            Value::String(v) => write!(f, "\"{}\"", v.as_ref()),
            Value::List(l) => {
                write!(f, "(")?;
                for (i, v) in l.iter().enumerate() {
                    write!(f, "{v}")?;
                    if i != l.len() - 1 {
                        write!(f, " ")?;
                    }
                }
                write!(f, ")")
            }
            Value::BuiltIn(_) => write!(f, "<built-in function>"),
            Value::Procedure(p) => write!(f, "<procedure with {} parameters>", p.param_names.len()),
            Value::Nil => write!(f, "#nil"),
        }
    }
}
