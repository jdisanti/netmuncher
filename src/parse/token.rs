//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use std::str::FromStr;
use std::fmt;

use parse::src_unit::Locator;
use error::{self, ErrorKind};

use lalrpop_util::ParseError;

#[derive(Debug, PartialEq, Eq)]
pub enum Tok {
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Equals,
    DotDot,
    Comma,
    Colon,
    Semicolon,
    Num(u32),
    Quote(String),
    Symbol(String),
    KeywordComponent,
    KeywordFootprint,
    KeywordInput,
    KeywordNet,
    KeywordNoConnect,
    KeywordOutput,
    KeywordPassive,
    KeywordPin,
    KeywordPowerIn,
    KeywordPowerOut,
    KeywordPrefix,
    KeywordRequire,
    KeywordTristate,
    KeywordValue,
}

impl fmt::Display for Tok {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Tok::LBrace => write!(f, "{{"),
            Tok::RBrace => write!(f, "}}"),
            Tok::LParen => write!(f, "("),
            Tok::RParen => write!(f, ")"),
            Tok::LBracket => write!(f, "["),
            Tok::RBracket => write!(f, "]"),
            Tok::Equals => write!(f, "="),
            Tok::DotDot => write!(f, ".."),
            Tok::Comma => write!(f, ","),
            Tok::Colon => write!(f, ":"),
            Tok::Semicolon => write!(f, ";"),
            Tok::Num(num) => write!(f, "{}", num),
            Tok::Quote(_) => write!(f, "\""),
            Tok::Symbol(ref sym) => write!(f, "{}", sym),
            Tok::KeywordComponent => write!(f, "component"),
            Tok::KeywordFootprint => write!(f, "footprint"),
            Tok::KeywordInput => write!(f, "input"),
            Tok::KeywordNet => write!(f, "net"),
            Tok::KeywordNoConnect => write!(f, "noconnect"),
            Tok::KeywordOutput => write!(f, "output"),
            Tok::KeywordPassive => write!(f, "passive"),
            Tok::KeywordPin => write!(f, "pin"),
            Tok::KeywordPowerIn => write!(f, "power_in"),
            Tok::KeywordPowerOut => write!(f, "power_out"),
            Tok::KeywordPrefix => write!(f, "prefix"),
            Tok::KeywordRequire => write!(f, "require"),
            Tok::KeywordTristate => write!(f, "tristate"),
            Tok::KeywordValue => write!(f, "value"),
        }
    }
}

pub fn tokenize(locator: &Locator, s: &str) -> error::Result<Vec<(usize, Tok, usize)>> {
    let mut tokens = vec![];
    let mut chars = s.char_indices();
    let mut lookahead = chars.next();

    while let Some((start, c)) = lookahead {
        if !c.is_whitespace() {
            match c {
                '{' => tokens.push((start, Tok::LBrace, start + 1)),
                '}' => tokens.push((start, Tok::RBrace, start + 1)),
                '(' => tokens.push((start, Tok::LParen, start + 1)),
                ')' => tokens.push((start, Tok::RParen, start + 1)),
                '[' => tokens.push((start, Tok::LBracket, start + 1)),
                ']' => tokens.push((start, Tok::RBracket, start + 1)),
                '=' => tokens.push((start, Tok::Equals, start + 1)),
                ',' => tokens.push((start, Tok::Comma, start + 1)),
                ':' => tokens.push((start, Tok::Colon, start + 1)),
                ';' => tokens.push((start, Tok::Semicolon, start + 1)),
                '.' => {
                    if let Some((_, c)) = chars.next() {
                        if c == '.' {
                            tokens.push((start, Tok::DotDot, start + 2));
                        } else {
                            bail!(ErrorKind::TokenizationError(format!(
                                "{}: unexpected character: {}",
                                locator.locate(start),
                                c
                            )));
                        }
                    } else {
                        bail!(ErrorKind::TokenizationError(format!(
                            "{}: unexpected dot",
                            locator.locate(start)
                        )));
                    }
                }
                '"' => {
                    let (quoted, _) = take_while(None, &mut chars, |c| c != '"');
                    let len = quoted.len();
                    tokens.push((start, Tok::Quote(quoted), start + len + 1));
                }
                '/' => {
                    if let Some((_, c)) = chars.next() {
                        if c == '/' {
                            drop(take_while(None, &mut chars, |c| c != '\n'));
                        } else {
                            unimplemented!("error condition for invalid comment")
                        }
                    } else {
                        unimplemented!("error condition for invalid comment")
                    }
                }
                _ if c.is_digit(10) => {
                    let (numstr, next) = take_while(Some(c), &mut chars, |c| c.is_digit(10));
                    lookahead = next;
                    tokens.push((
                        start,
                        Tok::Num(u32::from_str(&numstr).unwrap()),
                        start + numstr.len(),
                    ));
                    continue;
                }
                _ if c.is_ascii_alphabetic() => {
                    let (symbol, next) = take_while(Some(c), &mut chars, |c| {
                        c.is_ascii_alphanumeric() || c == '_'
                    });
                    let symbol_len = symbol.len();
                    lookahead = next;

                    match &symbol as &str {
                        "component" => tokens.push((start, Tok::KeywordComponent, start + 9)),
                        "footprint" => tokens.push((start, Tok::KeywordFootprint, start + 9)),
                        "input" => tokens.push((start, Tok::KeywordInput, start + 5)),
                        "net" => tokens.push((start, Tok::KeywordNet, start + 3)),
                        "noconnect" => tokens.push((start, Tok::KeywordNoConnect, start + 9)),
                        "output" => tokens.push((start, Tok::KeywordOutput, start + 6)),
                        "passive" => tokens.push((start, Tok::KeywordPassive, start + 7)),
                        "pin" => tokens.push((start, Tok::KeywordPin, start + 3)),
                        "power_in" => tokens.push((start, Tok::KeywordPowerIn, start + 8)),
                        "power_out" => tokens.push((start, Tok::KeywordPowerOut, start + 9)),
                        "prefix" => tokens.push((start, Tok::KeywordPrefix, start + 6)),
                        "require" => tokens.push((start, Tok::KeywordRequire, start + 7)),
                        "tristate" => tokens.push((start, Tok::KeywordTristate, start + 8)),
                        "value" => tokens.push((start, Tok::KeywordValue, start + 8)),
                        _ => tokens.push((start, Tok::Symbol(symbol), start + symbol_len)),
                    }
                    continue;
                }
                _ => {
                    bail!(ErrorKind::TokenizationError(format!(
                        "{}: unexpected character: {}",
                        locator.locate(start),
                        c
                    )));
                }
            }
        }

        lookahead = chars.next();
    }
    Ok(tokens)
}

fn take_while<C, F>(c0: Option<char>, chars: &mut C, f: F) -> (String, Option<(usize, char)>)
where
    C: Iterator<Item = (usize, char)>,
    F: Fn(char) -> bool,
{
    let mut buf = String::new();

    if c0.is_some() {
        buf.push(c0.unwrap());
    }

    while let Some((i, c)) = chars.next() {
        if !f(c) {
            return (buf, Some((i, c)));
        }

        buf.push(c);
    }

    return (buf, None);
}

pub fn validate_symbol(
    locator: &Locator,
    offset: usize,
    val: String,
) -> Result<String, ParseError<usize, Tok, error::Error>> {
    let valid_char = |c: char| c.is_ascii_alphanumeric() || c == '_';

    for c in val.chars() {
        if !valid_char(c) {
            return Err(ParseError::User {
                error: ErrorKind::ParseError(format!(
                    "{}: invalid character '{}' in symbol. \
                     Symbols must be alphanumeric with underscores.",
                    locator.locate(offset),
                    c
                )).into(),
            });
        }
    }
    Ok(val)
}
