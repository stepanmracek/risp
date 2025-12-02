use crate::{
    parser::Expr,
    scope::Scope,
    tokenizer::Token,
    value::{Procedure, Value},
};
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

fn expr2params(expr: &[Rc<Expr>], scope: &Rc<Scope>) -> Result<Vec<Value>, RuntimeError> {
    expr.iter().map(|param| evaluate(param, scope)).collect()
}

pub fn func_call(func: &Value, params: Vec<Value>) -> Result<Value, RuntimeError> {
    match func {
        Value::BuiltIn(func) => func.call(params),
        Value::Procedure(proc) => proc.call(params),
        _ => Err(RuntimeError::NotProcedure),
    }
}

fn begin(exprs: &[Rc<Expr>], scope: &Rc<Scope>) -> Result<Value, RuntimeError> {
    #[allow(clippy::double_ended_iterator_last)]
    let last = exprs.iter().map(|arg| evaluate(arg, scope)).last();
    if let Some(last) = last {
        last
    } else {
        Err(RuntimeError::IllFormedSpecialForm)
    }
}

fn if_statement(exprs: &[Rc<Expr>], scope: &Rc<Scope>) -> Result<Value, RuntimeError> {
    if exprs.len() != 3 {
        // TODO: Scheme also supports just `if (cond) (if_true_expr)` variant
        return Err(RuntimeError::IllFormedSpecialForm);
    }
    let cond = evaluate(&exprs[0], scope)?.truthy();
    if cond {
        evaluate(&exprs[1], scope)
    } else {
        evaluate(&exprs[2], scope)
    }
}

fn parser_param_names(params: &[Rc<Expr>]) -> Result<Vec<String>, RuntimeError> {
    params
        .iter()
        .map(|param| match param.as_ref() {
            Expr::Token(Token::Symbol(param_name)) => Ok(param_name.clone()),
            _ => Err(RuntimeError::ParameterMustBeIdentifier),
        })
        .collect()
}

fn lambda(exprs: &[Rc<Expr>], scope: &Rc<Scope>) -> Result<Value, RuntimeError> {
    if exprs.len() != 2 {
        // TODO: this is wrong, if there are multiple expressions after the parameter list,
        // they should be all executed and last value should be returned (like begin keyword)
        return Err(RuntimeError::IllFormedSpecialForm);
    }
    let params = &exprs[0].as_ref();
    let body = &exprs[1];
    if let Expr::List(params) = params {
        let param_names = parser_param_names(params)?;
        Ok(Value::Procedure(Procedure::new(
            param_names,
            body.clone(),
            scope.clone(),
        )))
    } else {
        // TODO: this is wrong, lambda also accepts single symbol as a parameter when it is the only one
        Err(RuntimeError::IllFormedSpecialForm)
    }
}

fn define_variable(
    exprs: &[Rc<Expr>],
    scope: &Rc<Scope>,
    set: bool,
) -> Result<Value, RuntimeError> {
    let symbol = &exprs[0];

    if let Expr::Token(Token::Symbol(symbol)) = symbol.as_ref() {
        let rhs_expr = &exprs[1];
        let rhs_val = evaluate(rhs_expr, scope)?;

        if set {
            Scope::set(scope, symbol, rhs_val)
                .map(|_| Value::Nil)
                .map_err(|_| RuntimeError::UnboundVariable(symbol.to_string()))
        } else {
            Scope::define(scope, symbol, rhs_val);
            Ok(Value::Nil)
        }
    } else {
        Err(RuntimeError::IdentifierExpected)
    }
}

fn define_procedure(exprs: &[Rc<Expr>], scope: &Rc<Scope>) -> Result<Value, RuntimeError> {
    let symbol_and_params = &exprs[0].as_ref();

    if let Expr::List(symbol_and_params) = symbol_and_params {
        if symbol_and_params.is_empty() {
            return Err(RuntimeError::IdentifierExpected);
        }
        let symbol_and_params = parser_param_names(symbol_and_params)?;
        let symbol = symbol_and_params[0].clone();
        let param_names = symbol_and_params.into_iter().skip(1).collect();
        let body = &exprs[1];

        let procedure = Value::Procedure(Procedure::new(param_names, body.clone(), scope.clone()));
        Scope::define(scope, &symbol, procedure);
        Ok(Value::Nil)
    } else {
        Err(RuntimeError::IllFormedSpecialForm)
    }
}

fn define(exprs: &[Rc<Expr>], scope: &Rc<Scope>) -> Result<Value, RuntimeError> {
    if exprs.len() != 2 {
        return Err(RuntimeError::IllFormedSpecialForm);
    }

    let head = exprs[0].as_ref();
    match head {
        Expr::Token(_) => define_variable(exprs, scope, false),
        Expr::List(_) => define_procedure(exprs, scope),
    }
}

fn invoke_named_function(
    exprs: &[Rc<Expr>],
    scope: &Rc<Scope>,
    symbol: &str,
) -> Result<Value, RuntimeError> {
    let params = expr2params(exprs, scope)?;
    let func = Scope::get(scope, symbol)
        .ok_or_else(|| RuntimeError::UnboundVariable(symbol.to_string()))?;
    func_call(&func, params)
}

fn invoke_lambda(
    body: &Rc<Expr>,
    params: &[Rc<Expr>],
    scope: &Rc<Scope>,
) -> Result<Value, RuntimeError> {
    let func = evaluate(body, scope)?;
    let params = expr2params(params, scope)?;
    func_call(&func, params)
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
                        Token::Set => define_variable(tail, scope, true),
                        // create new variable
                        Token::Define => define(tail, scope),
                        // invoke procedure or built-in function
                        Token::Symbol(symbol) => invoke_named_function(tail, scope, symbol),
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
