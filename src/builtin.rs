use std::rc::Rc;

use crate::{eval::RuntimeError, value::Value};
use itertools::Itertools;

fn values_to_ints(params: &[Value]) -> Result<Vec<i64>, RuntimeError> {
    params
        .iter()
        .map(|param| match param {
            Value::Int(number) => Ok(*number),
            _ => Err(RuntimeError::NumberExpected(param.clone())),
        })
        .collect()
}

fn values_to_floats(params: &[Value]) -> Result<Vec<f64>, RuntimeError> {
    params
        .iter()
        .map(|param| match param {
            Value::Float(number) => Ok(*number),
            _ => Err(RuntimeError::NumberExpected(param.clone())),
        })
        .collect()
}

fn values_to_strings(params: &[Value]) -> Result<Vec<Rc<String>>, RuntimeError> {
    params
        .iter()
        .map(|param| match param {
            Value::String(s) => Ok(s.clone()),
            _ => Err(RuntimeError::StringExpected(param.clone())),
        })
        .collect()
}

fn values_to_bool(params: &[Value]) -> Result<Vec<bool>, RuntimeError> {
    params
        .iter()
        .map(|param| match param {
            Value::Bool(b) => Ok(*b),
            _ => Err(RuntimeError::BooleanExpected(param.clone())),
        })
        .collect()
}

pub fn op_add(params: Vec<Value>) -> Result<Value, RuntimeError> {
    match params.first() {
        None => Ok(Value::Int(0)),
        Some(Value::Int(_)) => Ok(Value::Int(values_to_ints(&params)?.into_iter().sum())),
        Some(Value::Float(_)) => Ok(Value::Float(values_to_floats(&params)?.into_iter().sum())),
        Some(other) => Err(RuntimeError::NumberExpected(other.clone())),
    }
}

pub fn op_sub(params: Vec<Value>) -> Result<Value, RuntimeError> {
    match params.first() {
        None => Err(RuntimeError::WrongNumberOfAgumentsPassed),
        Some(Value::Int(_)) => {
            let ops = values_to_ints(&params)?;
            if ops.len() == 1 {
                Ok(Value::Int(-ops[0]))
            } else {
                let ans = ops.into_iter().reduce(|acc, op| acc - op);
                Ok(Value::Int(ans.unwrap_or(0)))
            }
        }
        Some(Value::Float(_)) => {
            let ops = values_to_floats(&params)?;
            if ops.len() == 1 {
                Ok(Value::Float(-ops[0]))
            } else {
                let ans = ops.into_iter().reduce(|acc, op| acc - op);
                Ok(Value::Float(ans.unwrap_or(0.0)))
            }
        }
        Some(other) => Err(RuntimeError::NumberExpected(other.clone())),
    }
}

pub fn op_mul(params: Vec<Value>) -> Result<Value, RuntimeError> {
    match params.first() {
        None => Ok(Value::Int(1)),
        Some(Value::Int(_)) => Ok(Value::Int(values_to_ints(&params)?.into_iter().product())),
        Some(Value::Float(_)) => Ok(Value::Float(
            values_to_floats(&params)?.into_iter().product(),
        )),
        Some(other) => Err(RuntimeError::NumberExpected(other.clone())),
    }
}

pub fn op_div(params: Vec<Value>) -> Result<Value, RuntimeError> {
    match params.first() {
        None => Err(RuntimeError::WrongNumberOfAgumentsPassed),
        Some(Value::Int(_)) => {
            let ops = values_to_ints(&params)?;
            if ops.len() == 1 {
                let ans = 1i64.checked_div(ops[0]).ok_or(RuntimeError::DivideByZero);
                Ok(Value::Int(ans?))
            } else {
                let first = ops[0];
                let ans = ops
                    .into_iter()
                    .skip(1)
                    .try_fold(first, |acc, op| acc.checked_div(op))
                    .ok_or(RuntimeError::DivideByZero)?;
                Ok(Value::Int(ans))
            }
        }
        Some(Value::Float(_)) => {
            let ops = values_to_floats(&params)?;
            let first = ops[0];
            if ops.len() == 1 {
                if ops[0] == 0.0 {
                    Err(RuntimeError::DivideByZero)
                } else {
                    Ok(Value::Float(1.0 / ops[0]))
                }
            } else {
                let ans = ops
                    .into_iter()
                    .skip(1)
                    .try_fold(
                        first,
                        |acc, op| if op != 0.0 { Some(acc / op) } else { None },
                    )
                    .ok_or(RuntimeError::DivideByZero)?;
                Ok(Value::Float(ans))
            }
        }
        Some(other) => Err(RuntimeError::NumberExpected(other.clone())),
    }
}

pub fn modulo(params: Vec<Value>) -> Result<Value, RuntimeError> {
    let [a, b] = values_to_ints(&params)?
        .try_into()
        .map_err(|_| RuntimeError::WrongNumberOfAgumentsPassed)?;
    if b == 0 {
        Err(RuntimeError::DivideByZero)
    } else {
        Ok(Value::Int((a + b) % b))
    }
}

pub fn op_eq(params: Vec<Value>) -> Result<Value, RuntimeError> {
    if params.is_empty() {
        Ok(Value::Bool(true))
    } else if let Ok(ops) = values_to_ints(&params) {
        let first = ops[0];
        Ok(Value::Bool(ops.into_iter().all(|v| v == first)))
    } else if let Ok(ops) = values_to_strings(&params) {
        let first = &ops[0];
        Ok(Value::Bool(ops.iter().all(|v| v == first)))
    } else {
        Err(RuntimeError::NumberExpected(params[0].clone()))
    }
}

pub fn op_leq(params: Vec<Value>) -> Result<Value, RuntimeError> {
    let ops = values_to_ints(&params)?;

    if ops.len() <= 1 {
        // empty list or just one op -> true
        Ok(Value::Bool(true))
    } else {
        let ans = ops.into_iter().tuple_windows().all(|(a, b)| a <= b);
        Ok(Value::Bool(ans))
    }
}

pub fn list(params: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::List(params))
}

pub fn string_concat(params: Vec<Value>) -> Result<Value, RuntimeError> {
    if params.is_empty() {
        Err(RuntimeError::WrongNumberOfAgumentsPassed)
    } else {
        let args = values_to_strings(&params)?;
        let mut ans = String::new();
        for s in args.iter() {
            ans.push_str(s);
        }
        Ok(Value::String(Rc::new(ans)))
    }
}

pub fn display(params: Vec<Value>) -> Result<Value, RuntimeError> {
    if params.len() != 1 {
        Err(RuntimeError::WrongNumberOfAgumentsPassed)
    } else {
        println!("{}", params[0]);
        Ok(Value::Nil)
    }
}

fn zip_vecs<T: Clone>(v: &[Vec<T>]) -> Vec<Vec<T>> {
    let min_len = v.iter().map(|x| x.len()).min().unwrap_or(0);
    (0..min_len)
        .map(|i| v.iter().map(|row| row[i].clone()).collect())
        .collect()
}

pub fn map(params: Vec<Value>) -> Result<Value, RuntimeError> {
    if params.len() < 2 {
        return Err(RuntimeError::WrongNumberOfAgumentsPassed);
    }

    let mut params = params.into_iter();
    let func = params.next().unwrap();
    let lists = params
        .map(|p| match p {
            Value::List(l) => Ok(l),
            v => Err(RuntimeError::ListExpected(v)),
        })
        .collect::<Result<Vec<_>, RuntimeError>>()?;

    let ans = zip_vecs(&lists)
        .into_iter()
        .map(|zipped_params| crate::eval::func_call(&func, zipped_params))
        .collect::<Result<Vec<_>, RuntimeError>>();

    Ok(Value::List(ans?))
}

pub fn apply(params: Vec<Value>) -> Result<Value, RuntimeError> {
    let [func, params] = params
        .try_into()
        .map_err(|_| RuntimeError::WrongNumberOfAgumentsPassed)?;
    match params {
        Value::List(params) => crate::eval::func_call(&func, params),
        v => Err(RuntimeError::ListExpected(v.clone())),
    }
}

pub fn read_file(params: Vec<Value>) -> Result<Value, RuntimeError> {
    if params.len() != 1 {
        return Err(RuntimeError::WrongNumberOfAgumentsPassed);
    }
    let file_name = &values_to_strings(&params)?[0];
    let content = std::fs::read_to_string(file_name.as_ref()).map_err(|_| RuntimeError::IO)?;
    Ok(Value::String(Rc::new(content)))
}

pub fn split_string(params: Vec<Value>) -> Result<Value, RuntimeError> {
    if params.len() != 1 {
        return Err(RuntimeError::WrongNumberOfAgumentsPassed);
    }
    let string = &values_to_strings(&params)?[0];

    let strings: Vec<_> = string
        .split_ascii_whitespace()
        .map(|s| Value::String(Rc::new(s.to_string())))
        .collect();
    Ok(Value::List(strings))
}

pub fn substring(params: Vec<Value>) -> Result<Value, RuntimeError> {
    let string = values_to_strings(&params[..1])?
        .first()
        .ok_or(RuntimeError::WrongNumberOfAgumentsPassed)?
        .clone();
    let [start, end] = values_to_ints(&params[1..])?
        .try_into()
        .map_err(|_| RuntimeError::WrongNumberOfAgumentsPassed)?;

    let start = if start < 0 {
        string.len() - start.unsigned_abs() as usize + 1
    } else {
        start as usize
    };

    let end = if end < 0 {
        string.len() - end.unsigned_abs() as usize + 1
    } else {
        end as usize
    };

    let substring = &string[start..end];
    Ok(Value::String(Rc::new(substring.to_string())))
}

pub fn parse_int(params: Vec<Value>) -> Result<Value, RuntimeError> {
    let [string] = values_to_strings(&params)?
        .try_into()
        .map_err(|_| RuntimeError::WrongNumberOfAgumentsPassed)?;

    let int = string.parse::<i64>().ok().unwrap_or_default();
    Ok(Value::Int(int))
}

pub fn and(params: Vec<Value>) -> Result<Value, RuntimeError> {
    let params = values_to_bool(&params)?;
    Ok(Value::Bool(params.into_iter().all(|v| v)))
}

pub fn or(params: Vec<Value>) -> Result<Value, RuntimeError> {
    let params = values_to_bool(&params)?;
    Ok(Value::Bool(params.into_iter().any(|v| v)))
}

pub fn not(params: Vec<Value>) -> Result<Value, RuntimeError> {
    let [param] = values_to_bool(&params)?
        .try_into()
        .map_err(|_| RuntimeError::WrongNumberOfAgumentsPassed)?;
    Ok(Value::Bool(!param))
}
