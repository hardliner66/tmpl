use std::{collections::HashMap, fmt::Display, num::ParseIntError};

use regex::Regex;
use serde::{Deserialize, Serialize};
use stringlit::s;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, DefinitionParseError>;

#[derive(Error, Debug)]
pub enum DefinitionParseError {
    #[error("Unknown Error")]
    Unknown,
    #[error("Missing main rule")]
    MissingMainRule,
    #[error("Invalid regex: {0}")]
    InvalidRegex(#[from] regex::Error),
    #[error("Invalid integer: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("Invalid float: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("Invalid repeat mode: {0}")]
    InvalidRepeatMode(String),
    #[error("Invalid char: {0}")]
    InvalidChar(char),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InternalPatternKind {
    Ident,
    Int,
    Float,
    String,
    Bool,
    Regex(#[serde(with = "serde_regex")] Regex),
    Keyword(String),
    Custom(String),
    Symbol(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InternalPattern {
    Named {
        name: Option<String>,
        kind: InternalPatternKind,
    },
    Raw {
        value: String,
    },
    Exact {
        pattern: Vec<TokenPattern>,
    },
}

pub fn ident(name: Option<String>) -> InternalPattern {
    InternalPattern::Named {
        name,
        kind: InternalPatternKind::Ident,
    }
}

pub fn int(name: Option<String>) -> InternalPattern {
    InternalPattern::Named {
        name,
        kind: InternalPatternKind::Int,
    }
}

pub fn float(name: Option<String>) -> InternalPattern {
    InternalPattern::Named {
        name,
        kind: InternalPatternKind::Float,
    }
}

pub fn string(name: Option<String>) -> InternalPattern {
    InternalPattern::Named {
        name,
        kind: InternalPatternKind::String,
    }
}

pub fn bool(name: Option<String>) -> InternalPattern {
    InternalPattern::Named {
        name,
        kind: InternalPatternKind::Bool,
    }
}

pub fn regex(name: Option<String>, value: &str) -> Result<InternalPattern> {
    Ok(InternalPattern::Named {
        name,
        kind: InternalPatternKind::Regex(regex::Regex::new(value)?),
    })
}

pub fn raw(value: &str) -> InternalPattern {
    InternalPattern::Raw {
        value: value.to_string(),
    }
}

pub fn keyword(name: Option<String>, value: &str) -> InternalPattern {
    InternalPattern::Named {
        name,
        kind: InternalPatternKind::Keyword(value.to_string()),
    }
}

pub fn symbol(name: Option<String>, value: &str) -> InternalPattern {
    InternalPattern::Named {
        name,
        kind: InternalPatternKind::Symbol(value.to_string()),
    }
}

pub fn custom(name: Option<String>, value: &str) -> InternalPattern {
    InternalPattern::Named {
        name,
        kind: InternalPatternKind::Custom(value.to_string()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RepeatMode {
    ZeroOrMore,
    OneOrMore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPattern {
    pub pattern: InternalPattern,
    pub is_optional: bool,
    pub repeat_mode: Option<RepeatMode>,
    pub separator: Option<String>,
}

pub fn optional(pattern: InternalPattern) -> Result<TokenPattern> {
    Ok(TokenPattern {
        pattern,
        is_optional: true,
        repeat_mode: None,
        separator: None,
    })
}

pub fn repeated(pattern: InternalPattern, repeat_mode: RepeatMode) -> Result<TokenPattern> {
    Ok(TokenPattern {
        pattern,
        is_optional: false,
        repeat_mode: Some(repeat_mode),
        separator: None,
    })
}

pub fn separated(
    pattern: InternalPattern,
    repeat_mode: RepeatMode,
    separator: String,
) -> Result<TokenPattern> {
    Ok(TokenPattern {
        pattern,
        is_optional: false,
        repeat_mode: Some(repeat_mode),
        separator: Some(separator),
    })
}

pub fn rw(pattern: InternalPattern) -> Result<TokenPattern> {
    Ok(TokenPattern {
        pattern,
        is_optional: false,
        repeat_mode: None,
        separator: None,
    })
}

pub fn alternative(left: Vec<TokenPattern>, right: Pattern) -> Result<Pattern> {
    Ok(Pattern::Alternative {
        left: left.into_iter().collect(),
        right: Box::new(right),
    })
}

pub fn token(left: Vec<TokenPattern>) -> Result<Pattern> {
    Ok(Pattern::Token(left))
}

impl std::iter::FromIterator<TokenPattern> for Vec<Pattern> {
    fn from_iter<I: IntoIterator<Item = TokenPattern>>(iter: I) -> Self {
        iter.into_iter().collect()
    }
}

pub fn unpack<T>(value: Vec<Result<T>>) -> Result<Vec<T>> {
    value.into_iter().collect()
}

pub fn with_repeat_mode(pat: InternalPattern, re: Option<String>) -> Result<TokenPattern> {
    match re {
        Some(s) => {
            let chars = s.chars().collect::<Vec<_>>();
            match &chars[..] {
                ['*', '*', rest @ ..] => {
                    separated(pat, RepeatMode::ZeroOrMore, rest.iter().collect())
                }
                ['+', '+', rest @ ..] => {
                    separated(pat, RepeatMode::OneOrMore, rest.iter().collect())
                }
                ['?'] => optional(pat),
                ['*'] => repeated(pat, RepeatMode::ZeroOrMore),
                ['+'] => repeated(pat, RepeatMode::OneOrMore),
                _ => Err(DefinitionParseError::InvalidRepeatMode(s.to_string()))?,
            }
        }
        None => rw(pat),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pattern {
    Alternative {
        left: Vec<TokenPattern>,
        right: Box<Pattern>,
    },
    Token(Vec<TokenPattern>),
}

impl From<TokenPattern> for Pattern {
    fn from(pattern: TokenPattern) -> Self {
        Pattern::Token(vec![pattern])
    }
}

impl From<Vec<TokenPattern>> for Pattern {
    fn from(pattern: Vec<TokenPattern>) -> Self {
        Pattern::Token(pattern)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Char(char),
    String(String),
    Int(String),
    Float(String),
    Bool(bool),
    List(Vec<Value>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Define {
    pub name: String,
    pub value: Value,
}

pub enum RuleOrDefine {
    Rule { name: String, pattern: Vec<Pattern> },
    Define(Define),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserDefinition {
    pub entry: Vec<Pattern>,
    pub rules: HashMap<String, Vec<Pattern>>,
    pub defines: Vec<Define>,
}

impl Display for ParserDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for def in &self.defines {
            writeln!(f, "{}", def)?;
        }
        writeln!(f)?;
        write!(f, "Main:")?;
        let formatted: Vec<_> = self.entry.iter().map(|p| format!("{p}")).collect();
        writeln!(f, "{}", formatted.join(""))?;
        Ok(())
    }
}

impl Display for Define {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "define {}: {}", self.name, self.value)
    }
}

impl Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pattern::Alternative { left, right } => {
                write!(
                    f,
                    "{}\n| {right}",
                    left.iter()
                        .map(|p| format!("{p}"))
                        .collect::<Vec<_>>()
                        .join("")
                )
            }
            Pattern::Token(token_patterns) => token_patterns
                .iter()
                .map(|p| format!("{p}"))
                .collect::<Vec<_>>()
                .join("")
                .fmt(f),
        }
    }
}

impl Display for InternalPatternKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Display for TokenPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rep = match (&self.repeat_mode, &self.separator) {
            (Some(RepeatMode::ZeroOrMore), None) => s!("*"),
            (Some(RepeatMode::OneOrMore), None) => s!("+"),
            (Some(RepeatMode::ZeroOrMore), Some(sep)) => format!("** \"{sep}\""),
            (Some(RepeatMode::OneOrMore), Some(sep)) => format!("++ \"{sep}\""),
            _ => s!(""),
        };
        match &self.pattern {
            InternalPattern::Named {
                name: Some(name),
                kind,
            } => write!(f, "<{}:{}>", name, kind),
            InternalPattern::Named { name: None, kind } => write!(f, "<{}>", kind),
            InternalPattern::Raw { value } => value.fmt(f),
            InternalPattern::Exact { pattern } => pattern
                .iter()
                .map(|p| format!("{p}"))
                .collect::<Vec<_>>()
                .join("")
                .fmt(f),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Char(c) => write!(f, "'{}'", c),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Bool(b) => write!(f, "{}", b),
            Value::List(list) => {
                write!(f, "[")?;
                for (i, v) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    v.fmt(f)?;
                }
                write!(f, "]")
            }
        }
    }
}
