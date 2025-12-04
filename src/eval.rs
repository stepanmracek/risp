use crate::{parser::Expr, scope::Scope, special_forms::*, tokenizer::Token, value::Value};
use std::{fmt::Display, rc::Rc};

#[derive(Debug)]
pub enum RuntimeError {
    NotProcedure,
    UnboundVariable(String),
    IllFormedExpression,
    IllFormedSpecialForm,
    ParameterMustBeIdentifier,
    OperatorIsNotProcedure,
    NumberExpected(Value),
    StringExpected(Value),
    BooleanExpected(Value),
    WrongNumberOfAgumentsPassed,
    IdentifierExpected,
    DivideByZero,
    ListExpected(Value),
    IO,
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub fn evaluate(expr: &Rc<Expr>, scope: &Rc<Scope>) -> Result<Value, RuntimeError> {
    let expr = expr.as_ref();
    match expr {
        Expr::Token(token) => match token {
            Token::Symbol(symbol) => Scope::get(scope, symbol)
                .ok_or_else(|| RuntimeError::UnboundVariable(symbol.clone())),
            Token::Int(i) => Ok(Value::Int(*i)),
            Token::Float(f) => Ok(Value::Float(*f)),
            Token::StringLiteral(s) => Ok(Value::String(s.clone())),
            Token::Bool(b) => Ok(Value::Bool(*b)),
            _ => todo!(),
        },
        Expr::List(list) => match list.first() {
            None => Err(RuntimeError::IllFormedExpression),
            Some(head) => {
                let tail = &list[1..];
                match head.as_ref() {
                    Expr::List(_) => invoke_lambda(head, tail, scope),
                    Expr::Token(head_token) => match head_token {
                        // list of commands - evaluate all and return last one
                        Token::Begin => begin(tail, scope),
                        // if (cond) (if_true_expr) (else_expr)
                        Token::If => if_statement(tail, scope),
                        // create custom procedure
                        Token::Lambda => lambda(tail, scope),
                        // set value of variable
                        Token::Set => {
                            define_variable(tail, scope, DefineBehavior::SetValueOfExisting)
                        }
                        // create new variable
                        Token::Define => define(tail, scope),
                        // invoke procedure or built-in function
                        Token::Symbol(symbol) => invoke_named_function(tail, scope, symbol),
                        // loop
                        Token::Do => do_loop(tail, scope),
                        Token::Int(_)
                        | Token::Float(_)
                        | Token::StringLiteral(_)
                        | Token::Bool(_) => Err(RuntimeError::OperatorIsNotProcedure),
                        // Following case should not happen because brackets are converted to nested lists
                        // and whitespace and comments are ignored in tokenizer
                        Token::LBracket | Token::RBracket | Token::WhiteSpace | Token::Comment => {
                            panic!()
                        }
                    },
                }
            }
        },
    }
}
