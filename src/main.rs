mod builtin;
mod eval;
mod parser;
mod scope;
mod tokenizer;
mod value;
use std::{fmt::Debug, rc::Rc};

#[derive(Debug)]
enum Error {
    Lexing(Vec<(tokenizer::LexingError, logos::Span)>),
    Parsing(parser::ParsingError),
    Runtime(eval::RuntimeError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Lexing(les) => std::fmt::Display::fmt(&les[0].0, f), // TODO: print ALL lexing errors
            Error::Parsing(e) => std::fmt::Display::fmt(&e, f),
            Error::Runtime(e) => std::fmt::Display::fmt(&e, f),
        }
    }
}

fn eval(src: &str, scope: &Rc<scope::Scope>) -> Result<value::Value, Error> {
    let tokens = tokenizer::tokenize(src).map_err(Error::Lexing)?;
    let expr = parser::parse(tokens.into_iter()).map_err(Error::Parsing)?;
    eval::evaluate(&Rc::new(expr), scope).map_err(Error::Runtime)
}

fn interactive_shell(scope: &Rc<scope::Scope>) {
    let mut rl = rustyline::DefaultEditor::new().unwrap();
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line).unwrap();
                let ans = eval(&line, scope);
                match ans {
                    Err(e) => println!("ERROR: {}", e),
                    Ok(value::Value::Nil) => {}
                    Ok(v) => println!("{}", v),
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
}

fn main() {
    let file_path = std::env::args().nth(1);

    let global_scope = scope::Scope::global();
    match file_path {
        Some(file_path) => {
            let src = std::fs::read_to_string(file_path).unwrap();
            match eval(&src, &global_scope) {
                Ok(_) => {}
                Err(e) => eprintln!("{e}"),
            }
        }
        None => interactive_shell(&global_scope),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn run(src: &str) -> Result<(value::Value, Rc<scope::Scope>), Error> {
        let global_scope = crate::scope::Scope::global();
        let ans = eval(src, &global_scope)?;
        Ok((ans, global_scope))
    }

    #[test]
    fn built_in_func_call() {
        let src = "(+ 10 10)";
        let ans = run(src);
        assert!(matches!(ans, Ok((value::Value::Int(20), _))));
    }

    #[test]
    fn pointer_to_build_in_func() {
        let src = "(define foo +)";
        let ans = run(src);
        assert!(matches!(ans, Ok((value::Value::Nil, _))));
        let global_scope = ans.expect("Ok value expected").1;
        let foo = scope::Scope::get(&global_scope, "foo").expect("foo variable expected");
        assert!(matches!(foo, value::Value::BuiltIn(_)));
    }

    #[test]
    fn call_anonymous_func() {
        let src = "((lambda (a b) (+ a b)) 40 2)";
        let ans = run(src);
        assert!(matches!(ans, Ok((value::Value::Int(42), _))));
    }

    #[test]
    fn begin() {
        let src = "(begin (define r 10.0) (* pi (* r r)))";
        let ans = run(src);
        assert!(matches!(ans, Ok((value::Value::Float(_), _))));
    }

    #[test]
    fn if_then_else() {
        let src = "(if (= 0 (+ 1 1)) 42.0 (list 1 2 3))";
        let ans = run(src);
        assert!(matches!(ans, Ok((value::Value::List(_), _))));
    }

    #[test]
    fn factorial() {
        let src = "
            (begin
              (define fact (lambda (n) (if (<= n 1) 1 (* n (fact (- n 1))))))
              (fact 5))";
        let ans = run(src);
        assert!(matches!(ans, Ok((value::Value::Int(120), _))));
    }

    #[test]
    fn define_function_shortcut() {
        let src = "
            (begin
              (define (f x y) (+ y x))
              (f 4 2))";
        let ans = run(src);
        assert!(matches!(ans, Ok((value::Value::Int(6), _))));
    }

    #[test]
    fn set_outer_scope() {
        let src = "
            (begin
              (define x 1)
              (define (f y) (set! x y))
              (f 2)
              x)";
        let ans = run(src).expect("Run expected to success");
        assert!(
            matches!(ans.0, value::Value::Int(2)),
            "Value::Int(2) expected, got {:?}",
            ans.0
        );
    }

    #[test]
    fn zeronary_ops() {
        let examples = [("(+)", 0), ("(*)", 1)];

        for (src, expected) in examples {
            let ans = run(src).unwrap().0;
            match ans {
                value::Value::Int(ans) => assert_eq!(ans, expected),
                _ => panic!(),
            }
        }
    }

    #[test]
    fn floating_point_arithmetic() {
        let examples = [
            ("(+ 42.0)", 42.0),
            ("(- 42.0)", -42.0),
            ("(* 42.0)", 42.0),
            ("(/ 42.0)", 1.0 / 42.0),
            ("(+ 40.0 2.0)", 42.0),
            ("(- 44.0 2.0)", 42.0),
            ("(* 21.0 2.0)", 42.0),
            ("(/ 84.0 2.0)", 42.0),
            ("(+ 40.0 1.0 1.0)", 42.0),
            ("(- 44.0 1.0 1.0)", 42.0),
            ("(* 10.5 2.0 2.0)", 42.0),
            ("(/ 84.0 4.0 0.5)", 42.0),
        ];

        for (src, expected) in examples {
            let ans = run(src).unwrap().0;
            match ans {
                value::Value::Float(ans) => assert_eq!(ans, expected, "{}", src),
                _ => panic!(),
            }
        }
    }

    #[test]
    fn fixed_point_arithmetic() {
        let examples = [
            ("(+ 42)", 42),
            ("(- 42)", -42),
            ("(* 42)", 42),
            ("(/ 42)", 1 / 42),
            ("(+ 40 2)", 42),
            ("(- 44 2)", 42),
            ("(* 21 2)", 42),
            ("(/ 84 2)", 42),
            ("(+ 40 1 1)", 42),
            ("(- 44 1 1)", 42),
            ("(* 3 2 7)", 42),
            ("(/ 252 2 3)", 42),
        ];

        for (src, expected) in examples {
            let ans = run(src).unwrap().0;
            match ans {
                value::Value::Int(ans) => assert_eq!(ans, expected, "{}", src),
                _ => panic!(),
            }
        }
    }

    #[test]
    fn zero_division_error() {
        let examples = [
            "(/ 0.0)",
            "(/ 0)",
            "(/ 42.0 0.0)",
            "(/ 42 0)",
            "(/ 42.0 2.0 0.0)",
            "(/ 42 2 0)",
            "(/ 42.0 2.0 0.0 2.0)",
            "(/ 42 2 0 2)",
        ];

        for src in examples {
            let ans = run(src);
            assert!(matches!(
                ans,
                Err(Error::Runtime(eval::RuntimeError::DivideByZero))
            ))
        }
    }

    #[test]
    fn modulo() {
        let examples = [
            ("(mod 95 100)", 95),
            ("(mod 195 100)", 95),
            ("(mod -5 100)", 95),
        ];

        for (src, expected) in examples {
            let ans = run(src).unwrap().0;
            match ans {
                value::Value::Int(ans) => assert_eq!(ans, expected, "{}", src),
                _ => panic!(),
            }
        }
    }

    #[test]
    fn string_operations() {
        let src = "
            (parse-int
                (substring
                    (apply string-concatenate
                        (split-string
                            (read-file \"fixtures/hello\")))
                5 -1)
            )";
        let ans = run(src);
        match ans {
            Ok((value::Value::Int(ans), _)) => assert_eq!(ans, 123),
            _ => panic!(),
        }
    }

    fn values_to_ints(params: Vec<value::Value>) -> Vec<i64> {
        params
            .iter()
            .map(|param| match param {
                value::Value::Int(i) => *i,
                _ => panic!(),
            })
            .collect()
    }

    #[test]
    fn map() {
        let examples = [
            ("(map + (list 1 2 3))", vec![1, 2, 3]),
            ("(map + (list 1 2 3) (list 4 5 6))", vec![5, 7, 9]),
            ("(map + (list 1 2 3) (list 2 0) (list 4 5 6))", vec![7, 7]),
        ];

        for (src, expected) in examples {
            let ans = run(src).unwrap().0;
            match ans {
                value::Value::List(ans) => {
                    let ans = values_to_ints(ans);
                    assert_eq!(ans, expected, "{}", src)
                }
                _ => panic!(),
            }
        }
    }

    #[test]
    fn apply() {
        let ans = run("(apply + (list 1 2 3))");
        assert!(matches!(ans, Ok((value::Value::Int(6), _))));
    }

    #[test]
    fn generator() {
        let ans = run("(begin
            (define next (make-generator 10 -3))
            (list (next) (next) (next))
        )")
        .unwrap()
        .0;
        match ans {
            value::Value::List(values) => {
                let ints = values_to_ints(values);
                assert_eq!(ints, vec![10, 7, 4])
            }
            _ => panic!(),
        }
    }
}
