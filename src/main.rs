#![allow(dead_code, unused_imports)]

use std::path::PathBuf;

use clap::Parser;
use logos::Logos;
use serde::Serialize;

#[derive(Parser)]
struct Opts {
    grammar: PathBuf,
    src: Option<PathBuf>,
}

fn print<T: Serialize>(t: &T) {
    println!("{}", serde_yaml::to_string(t).unwrap());
}

fn main() -> anyhow::Result<()> {
    let Opts { grammar, src: _ } = Opts::parse();
    let parsed = tmpl::definition::parse(std::fs::read_to_string(grammar)?.as_str())??;
    print(&parsed);
    // let src = std::fs::read_to_string(src)?;
    // let lexer = tmpl::lexer::Token::lexer(&src);
    // let parser = tmpl::custom::Parser::new(parsed, lexer.collect::<Result<Vec<_>, _>>()?);
    // let ast = parser.parse()?;
    // println!("{}", rsn::to_string_pretty(&ast)?);
    Ok(())
}
