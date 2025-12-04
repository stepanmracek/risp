use crate::{
    eval::{RuntimeError, evaluate},
    parser::Expr,
    scope::Scope,
    tokenizer::Token,
    value::{Procedure, Value},
};
use std::rc::Rc;

pub fn func_call(func: &Value, params: Vec<Value>) -> Result<Value, RuntimeError> {
    match func {
        Value::BuiltIn(func) => func.call(params),
        Value::Procedure(proc) => proc.call(params),
        _ => Err(RuntimeError::NotProcedure),
    }
}

pub fn begin(exprs: &[Rc<Expr>], scope: &Rc<Scope>) -> Result<Value, RuntimeError> {
    let last = exprs
        .iter()
        .map(|expr| evaluate(expr, scope))
        .try_fold(None, |_, result| {
            let value = result?;
            Ok(Some(value))
        })?;

    if let Some(last) = last {
        Ok(last)
    } else {
        Err(RuntimeError::IllFormedSpecialForm)
    }
}

pub fn if_statement(exprs: &[Rc<Expr>], scope: &Rc<Scope>) -> Result<Value, RuntimeError> {
    if exprs.len() != 3 {
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

pub fn lambda(exprs: &[Rc<Expr>], scope: &Rc<Scope>) -> Result<Value, RuntimeError> {
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

pub enum DefineBehavior {
    DefineNew,
    SetValueOfExisting,
}

pub fn define_variable(
    exprs: &[Rc<Expr>],
    scope: &Rc<Scope>,
    behavior: DefineBehavior,
) -> Result<Value, RuntimeError> {
    let symbol = &exprs[0];

    if let Expr::Token(Token::Symbol(symbol)) = symbol.as_ref() {
        let rhs_expr = &exprs[1];
        let rhs_val = evaluate(rhs_expr, scope)?;

        match behavior {
            DefineBehavior::SetValueOfExisting => Scope::set(scope, symbol, rhs_val)
                .map(|_| Value::Nil)
                .map_err(|_| RuntimeError::UnboundVariable(symbol.to_string())),
            DefineBehavior::DefineNew => {
                Scope::define(scope, symbol, rhs_val);
                Ok(Value::Nil)
            }
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

pub fn define(exprs: &[Rc<Expr>], scope: &Rc<Scope>) -> Result<Value, RuntimeError> {
    if exprs.len() != 2 {
        return Err(RuntimeError::IllFormedSpecialForm);
    }

    let head = exprs[0].as_ref();
    match head {
        Expr::Token(_) => define_variable(exprs, scope, DefineBehavior::DefineNew),
        Expr::List(_) => define_procedure(exprs, scope),
    }
}

fn expr2params(expr: &[Rc<Expr>], scope: &Rc<Scope>) -> Result<Vec<Value>, RuntimeError> {
    expr.iter().map(|param| evaluate(param, scope)).collect()
}

pub fn invoke_named_function(
    exprs: &[Rc<Expr>],
    scope: &Rc<Scope>,
    symbol: &str,
) -> Result<Value, RuntimeError> {
    let params = expr2params(exprs, scope)?;
    let func = Scope::get(scope, symbol)
        .ok_or_else(|| RuntimeError::UnboundVariable(symbol.to_string()))?;
    func_call(&func, params)
}

pub fn invoke_lambda(
    body: &Rc<Expr>,
    params: &[Rc<Expr>],
    scope: &Rc<Scope>,
) -> Result<Value, RuntimeError> {
    let func = evaluate(body, scope)?;
    let params = expr2params(params, scope)?;
    func_call(&func, params)
}

fn parse_binding(init_expr: &Rc<Expr>) -> Result<[Rc<Expr>; 3], RuntimeError> {
    let init_expr = match init_expr.as_ref() {
        Expr::List(init_expr) if init_expr.len() == 3 => init_expr,
        _ => return Err(RuntimeError::IllFormedSpecialForm),
    };

    Ok([
        init_expr[0].clone(),
        init_expr[1].clone(),
        init_expr[2].clone(),
    ])
}

pub fn do_loop(exprs: &[Rc<Expr>], scope: &Rc<Scope>) -> Result<Value, RuntimeError> {
    // (do ((symbol init value) ...) (test_cond expr1 expr2 ...) expr1 expr2 ...)
    if exprs.len() < 2 {
        return Err(RuntimeError::IllFormedSpecialForm);
    }

    // Vec<(symbol init value)>
    let init: Vec<_> = match exprs[0].as_ref() {
        Expr::List(list) => list,
        _ => {
            return Err(RuntimeError::IllFormedSpecialForm);
        }
    }
    .iter()
    .map(parse_binding)
    .collect::<Result<_, _>>()?;

    let test = match exprs[1].as_ref() {
        Expr::List(list) if list.len() >= 1 => list,
        _ => {
            return Err(RuntimeError::IllFormedSpecialForm);
        }
    };

    let scope = Scope::nest(scope);
    let body = &exprs[2..];

    for [symbol, init, _] in init.iter() {
        define_variable(
            &[symbol.clone(), init.clone()],
            &scope,
            DefineBehavior::DefineNew,
        )?;
    }

    loop {
        let terminate = evaluate(&test[0], &scope)?.truthy();
        if terminate {
            let ans = if test.len() > 1 {
                begin(&test[1..], &scope)?
            } else {
                Value::Nil
            };
            return Ok(ans);
        }

        if !body.is_empty() {
            begin(body, &scope)?;
        }

        for [symbol, _, value] in init.iter() {
            define_variable(
                &[symbol.clone(), value.clone()],
                &scope,
                DefineBehavior::SetValueOfExisting,
            )?;
        }
    }
}
