use crate::custom::Ast;
use crate::definition::*;

use std::cell::RefCell;
use std::num::ParseIntError;
use std::rc::Rc;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, ParseError>;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unknown Error")]
    Unknown,
}

pub struct Parser {
    definition: crate::definition::ParserDefinition,
    index: Rc<RefCell<usize>>,
    lexer: Vec<crate::lexer::Token>,
}

impl Parser {
    pub fn new(
        definition: crate::definition::ParserDefinition,
        lexer: Vec<crate::lexer::Token>,
    ) -> Self {
        Self {
            definition,
            lexer,
            index: Rc::new(RefCell::new(0)),
        }
    }

    fn parse_pattern(&self, pattern: &[TokenPattern]) -> Result<Ast> {
        Ok(Ast {})
    }

    fn parse_alternative(&self, left: &[TokenPattern], right: &Pattern) -> Result<Ast> {
        let current_pos = *self.index.borrow();
        if let Ok(ast) = self.parse_pattern(left) {
            return Ok(ast);
        }
        *self.index.borrow_mut() = current_pos;
        match right {
            Pattern::Token(t) => {}
            Pattern::Alternative { left, right } => {
                let ast = self.parse_alternative(left, right)?;
            }
        }
        Ok(Ast {})
    }

    fn parse_rule(&self, rule_name: &str) -> Result<Ast> {
        let pattern = self.definition.rules.get(rule_name).unwrap();
        for p in pattern {
            match p {
                Pattern::Token(t) => {}
                Pattern::Alternative { left, right } => {
                    let ast = self.parse_alternative(left, right)?;
                }
            }
        }

        Ok(Ast {})
    }

    pub fn parse(&self) -> Result<Ast> {
        Ok(Ast {})
    }
}
