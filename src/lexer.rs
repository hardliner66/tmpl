use logos::Logos;
use thiserror::Error;

#[allow(clippy::enum_variant_names)]
#[derive(Default, Debug, Clone, PartialEq, Error)]
pub enum LexingError {
    #[error("Invalid integer: {0}")]
    InvalidInteger(#[from] std::num::ParseIntError),
    #[error("Invalid float: {0}")]
    InvalidFloat(#[from] std::num::ParseFloatError),
    #[error("Invalid lexeme")]
    #[default]
    InvalidLexeme,
}

#[derive(Debug, Logos, PartialEq)]
#[logos(error = LexingError)]
pub enum Token {
    #[regex(r"([ \t]|\r\n|\n)+")]
    Ws,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[regex(r"[-+*/=>\\_.:,;<>!$%&?@]+", |lex| lex.slice().to_owned())]
    Symbol(String),
    #[regex(r"[a-zA-Z_][a-zA-Z_0-9]+", |lex| lex.slice().to_owned())]
    Ident(String),
    #[regex(r"[0-9]+\.[0-9]*", |lex| lex.slice().parse())]
    Float(f64),
    #[regex(r"[0-9]+", |lex| lex.slice().parse())]
    Integer(i64),
}
