use crate::tokenizer::Token;
use logos::Span;
use std::{fmt::Display, rc::Rc};

#[derive(Debug)]
pub enum Expr {
    Token(Token),
    List(Vec<Rc<Expr>>),
}

#[derive(Debug)]
pub enum ParsingError {
    ExpectedToken,
    UnexpectedRightBracket(Span),
}

impl Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingError::ExpectedToken => write!(f, "Expected token"),
            ParsingError::UnexpectedRightBracket(s) => {
                write!(f, "Unexpeted right bracket at {}", s.start)
            }
        }
    }
}

fn parse_recursive<I>(head: (Token, Span), tail: &mut I) -> Result<Expr, ParsingError>
where
    I: Iterator<Item = (Token, Span)>,
{
    match head {
        (Token::LBracket, _) => {
            let mut list = vec![];
            loop {
                let (head, span) = tail.next().ok_or(ParsingError::ExpectedToken)?;
                if head == Token::RBracket {
                    break;
                } else {
                    let expr = parse_recursive((head, span), tail)?;
                    list.push(Rc::new(expr));
                }
            }
            Ok(Expr::List(list))
        }
        (Token::RBracket, span) => Err(ParsingError::UnexpectedRightBracket(span)),
        (token, _) => Ok(Expr::Token(token)),
    }
}

pub fn parse<I>(mut tokens: I) -> Result<Expr, ParsingError>
where
    I: Iterator<Item = (Token, Span)>,
{
    let head = tokens.next().ok_or(ParsingError::ExpectedToken)?;
    parse_recursive(head, &mut tokens)
}
