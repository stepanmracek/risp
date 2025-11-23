use logos::{Logos, Span};
use std::{convert::Infallible, fmt::Display, rc::Rc, str::FromStr};

#[derive(Debug, PartialEq, Clone, Default)]
pub enum LexingError {
    NumberParseError,
    InvalidEscape(char),
    UnexpectedEof,
    #[default]
    Other,
}

impl From<std::num::ParseIntError> for LexingError {
    fn from(_: std::num::ParseIntError) -> Self {
        LexingError::NumberParseError
    }
}

impl From<std::num::ParseFloatError> for LexingError {
    fn from(_: std::num::ParseFloatError) -> Self {
        LexingError::NumberParseError
    }
}

impl From<Infallible> for LexingError {
    fn from(_: Infallible) -> Self {
        LexingError::Other
    }
}

impl Display for LexingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexingError::NumberParseError => write!(f, "Invalid number format"),
            LexingError::InvalidEscape(c) => {
                write!(f, "Invalid escape character {}", c)
            }
            LexingError::UnexpectedEof => write!(f, "Unexpected EOF"),
            LexingError::Other => write!(f, "Unspecified lexing error"),
        }
    }
}

fn parse_string(lex: &mut logos::Lexer<Token>) -> Result<Rc<String>, LexingError> {
    let slice = lex.slice(); // includes quotes
    let inner = &slice[1..slice.len() - 1]; // remove outer quotes

    // Unescape using Rust's own unescape rules (simple version):
    let mut result = String::new();
    let mut chars = inner.chars();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some(ch) => return Err(LexingError::InvalidEscape(ch)),
                None => return Err(LexingError::UnexpectedEof),
            }
        } else {
            result.push(c);
        }
    }

    Ok(Rc::new(result))
}

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(error = LexingError)]
pub enum Token {
    #[token("(")]
    LBracket,
    #[token(")")]
    RBracket,
    #[token("begin")]
    Begin,
    #[token("define")]
    Define,
    #[token("set!")]
    Set,
    #[token("lambda")]
    Lambda,
    #[token("if")]
    If,
    #[regex(r#""([^"\\]|\\.)*""#, parse_string)]
    StringLiteral(Rc<String>),
    #[regex(r"[\*\+\-/=<>a-zA-Z]+", |lex| String::from_str(lex.slice()))]
    Symbol(String),
    #[regex("-?[0-9]+", |lex| lex.slice().parse())]
    Int(i64),
    #[regex("-?([0-9]+[.]([0-9]*)?|[.][0-9]+)", |lex| lex.slice().parse())]
    Float(f64),
    #[regex(r"[ \t\n\f]+", logos::skip)]
    WhiteSpace,
}

type Tokens = Vec<(Token, Span)>;
type LexingErrors = Vec<(LexingError, Span)>;

pub fn tokenize(src: &str) -> Result<Tokens, LexingErrors> {
    let lexer = Token::lexer(src);

    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    for (token, span) in lexer.spanned() {
        match token {
            Ok(token) => tokens.push((token, span)),
            Err(error) => errors.push((error, span)),
        }
    }

    if errors.is_empty() {
        Ok(tokens)
    } else {
        Err(errors)
    }
}
