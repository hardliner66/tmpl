use std::collections::HashMap;

use crate::definition::ast::*;

peg::parser! {
    grammar parser() for str {
        rule traced<T>(e: rule<T>) -> T =
            &(input:$([_]*) {
                #[cfg(feature = "trace")]
                println!("[PEG_INPUT_START]\n{}\n[PEG_TRACE_START]", input);
            })
            e:e()? {?
                #[cfg(feature = "trace")]
                println!("[PEG_TRACE_STOP]");
                e.ok_or("")
            }

        pub rule main() -> Result<ParserDefinition>
            = traced(<top()>)

        rule top() -> Result<ParserDefinition>
            = _ other:rule_or_define()* _ {
                let mut rules = HashMap::new();
                let mut defines = Vec::new();
                for rod in unpack(other)? {
                    match rod {
                        RuleOrDefine::Rule{name, pattern} => _ = rules.insert(name, pattern),
                        RuleOrDefine::Define(d) => defines.push(d)
                    }
                }
                match rules.remove("Main") {
                    Some(entry) =>
                        Ok(ParserDefinition {
                            entry,
                            rules,
                            defines,
                        }),
                    None => Err(DefinitionParseError::MissingMainRule),
                }
            }
            / expected!("Main Rule")

        rule rule_or_define() -> Result<RuleOrDefine>
            = d:define() { Ok(RuleOrDefine::Define(d?)) }
            / r:r#rule() { let (name, pattern) = r?; Ok(RuleOrDefine::Rule{name, pattern}) }
            / expected!("Rule or Define")

        rule define() -> Result<Define>
            = _ "define" _ r:ident() _ ":" _ rs:value() _ ";" _ {
                Ok(Define { name: r, value: rs? })
            }
            / expected!("Define")

        rule value() -> Result<Value>
            = _ "[" _ v:value() ** "," _ "]" _ {
                Ok(Value::List(unpack(v)?))
            }
            / _ "'" v:$([^'\'' | '\\'] / "\\\\" / "\\'") "'" _ {
                let chars = v.chars().collect::<Vec<_>>();
                if chars.len() == 1 {
                    Ok(Value::Char(v.chars().next().unwrap()))
                } else if chars[0] == '\\' {
                    match chars[1] {
                        '\\' => Ok(Value::Char('\\')),
                        '\'' => Ok(Value::Char('\'')),
                        _ => Err(DefinitionParseError::InvalidChar(chars[1])),
                    }
                } else {
                    Err(DefinitionParseError::InvalidChar(chars[0]))
                }
            }
            / _ "\"" v:$([^'"' | '\\'] / "\\\\" / "\\\"") "\"" _ {
                let str = v.chars().collect::<String>();
                Ok(Value::String(str.replace("\\\"", "\"").replace("\\\\", "\\")))
            }
            / _ v:float() _ {
                Ok(Value::Float(v?))
            }
            / _ v:int() _ {
                Ok(Value::Int(v?))
            }
            / _ v:bool() _ {
                Ok(Value::Bool(v))
            }
            / expected!("value")

        rule r#rule() -> Result<(String, Vec<Pattern>)>
            = _ r:ident() _ ":" _ rs:pattern()+ _ "~~~" _ {
                Ok((r, unpack(rs)?))
            }
            / expected!("Rule")

        rule pattern() -> Result<Pattern>
            = _ left:token()+ _ "|" _ right:pattern() {
                alternative(unpack(left)?, right?)
            }
            / _ left:token()+ {
                token(unpack(left)?)
            }

        rule pat<'a, T>(p: rule<T>) -> Option<String>
            = _ "<" _ r:ident() _ ":" _ p() _ ">" { Some(r) }
            / _ "<" _ p() _ ">" { None }

        rule repeat() -> String
            = "?" { "?".to_string() }
            / "*" { "*".to_string() }
            / "+" { "+".to_string() }
            / __ "**" __ sym:string() { format!("**{sym}") }
            / __ "++" __ sym:string() { format!("++{sym}") }

        rule string() -> String
            = "\"" s:$(([^'\\' | '"'] / "\\\\" / "\\\"")+) "\"" { s.to_string() }
            / expected!("string")

        rule token() -> Result<TokenPattern>
            = r:pat(<"ident">) re:repeat()? { with_repeat_mode(ident(r), re) }
            / r:pat(<"int">) re:repeat()? { with_repeat_mode(int(r), re) }
            / r:pat(<"float">) re:repeat()? { with_repeat_mode(float(r), re) }
            / r:pat(<"string">) re:repeat()? { with_repeat_mode(string(r), re) }
            / r:pat(<"bool">) re:repeat()? { with_repeat_mode(bool(r), re) }
            / _ "<" _ r:(r:ident() _ ":" _ { r })? "sym[" v:symbol() "]" _ ">" re:repeat()? { with_repeat_mode(symbol(r, &v), re) }
            / _ "<" _ r:(r:ident() _ ":" _ { r })? "string" _ ">" re:repeat()? { with_repeat_mode(string(r), re) }
            / _ "<" _ r:(r:ident() _ ":" _ { r })? "bool" _ ">" re:repeat()? { with_repeat_mode(bool(r), re) }
            / _ "<" _ r:(r:ident() _ ":" _ { r })? "s/" v:regex() "/" _ ">" re:repeat()? { with_repeat_mode(regex(r, &v)?, re) }
            / _ "<" _ r:(r:ident() _ ":" _ { r })? "kw[" _ v:ident()  _ "]" _ ">" re:repeat()? { with_repeat_mode(keyword(r, &v), re) }
            / _ "<" _ r:(r:ident() _ ":" _ { r })? v:ident() _ ">" re:repeat()? { with_repeat_mode(custom(r, &v), re) }
            / _ r:ident() re:repeat()? { with_repeat_mode(raw(&r), re) }
            / _ r:$(([^'\n' | ' ' | '\t' | '~' | '0' ..= '9' | 'a' ..= 'z' | 'A' ..= 'Z'] / "\\~~~")) { rw(symbol(None, r)) }
            / expected!("pattern")

        rule regex() -> String
            = s:$([^'/'] / "\\/")+ {
                s.join("")
            }
            / expected!("regex")

        rule ident() -> String
            = s:$(['A'..='Z' | 'a'..='z' | '_']['A'..='Z' | 'a'..='z' | '_' | '0'..='9']*) { s.to_string() }
            / expected!("identifier")

        rule float() -> Result<String>
            = s:$(['0'..='9']+ "." ['0'..='9']*) {
                Ok(s.to_string())
            }
            / expected!("float")

        rule int() -> Result<String>
            = s:$(['0'..='9']+) { Ok(s.to_string()) }
            / expected!("int")

        rule bool() -> bool
            = "true" { true }
            / "false" { false }
            / expected!("bool")

        rule symbol() -> String
            = s:$(['-' | '+' | '*' | '/' | '=' | '>' | '\\' | '_' | '.' | ':' | ',' |
            ';' | '<' | '>' | '!' | '$' | '%' | '&' | '?' | '@']+) { s.to_string() }
            / expected!("symbol")

        rule _() = quiet!{[' ' | '\n' | '\t' | '\r']*}
        rule __() = quiet!{[' ' | '\t']*}
    }
}

pub fn parse(
    src: &str,
) -> std::result::Result<Result<ParserDefinition>, peg::error::ParseError<peg::str::LineCol>> {
    parser::main(src)
}
