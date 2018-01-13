//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use lalrpop_util::ParseError;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use error;

#[cfg_attr(rustfmt, rustfmt_skip)]
mod grammar;

pub mod component;
pub mod token;
pub mod source;
mod validate;

use parse::component::Component;
use parse::source::{Locator, Sources};

pub struct ParseResult {
    pub sources: Sources,
    pub components: Vec<Component>,
    pub global_nets: Vec<String>,
}

pub fn parse(file_name: &str) -> error::Result<ParseResult> {
    let main_file = Path::new(file_name).file_name().unwrap();
    let main_path = Path::new(file_name).parent().unwrap();

    let mut sources = Sources::new();

    let mut modules_to_require: Vec<PathBuf> = Vec::new();
    let mut modules_required: Vec<PathBuf> = Vec::new();
    modules_to_require.push(module_path(&main_path, &main_file).unwrap());

    let mut global_nets: Vec<String> = Vec::new();
    let mut components: Vec<Component> = Vec::new();
    while let Some(path) = modules_to_require.pop() {
        if !modules_required.contains(&path) {
            modules_required.push(path.clone());
            let source_id = sources.push_source(path.to_str().unwrap().into(), load_file(path)?);
            let locator = Locator::new(&sources, source_id);
            let parse_result = parse_file(&locator, sources.code(source_id))?;
            modules_to_require.extend(
                parse_result
                    .requires
                    .into_iter()
                    .filter_map(|r| module_path(&main_path, &r)),
            );
            global_nets.extend(parse_result.global_nets.into_iter());
            components.extend(parse_result.components.into_iter());
        }
    }

    validate::Validator::new(&sources, &global_nets, &components).validate()?;

    Ok(ParseResult {
        sources: sources,
        components: components,
        global_nets: global_nets,
    })
}

#[derive(Default)]
pub struct ParseFileResult {
    pub requires: Vec<String>,
    pub components: Vec<Component>,
    pub global_nets: Vec<String>,
}

impl ParseFileResult {
    pub fn new() -> ParseFileResult {
        Default::default()
    }
}

fn parse_file(locator: &Locator, source: &str) -> error::Result<ParseFileResult> {
    let tokens = token::tokenize(locator, source)?;
    grammar::parse_Source(&locator, tokens.into_iter()).map_err(|e| match e {
        ParseError::InvalidToken { location } => {
            error::ErrorKind::ParseError(format!("{}: invalid token", locator.locate(location)))
                .into()
        }
        ParseError::UnrecognizedToken { token, expected } => match token {
            Some((location, token, _)) => error::ErrorKind::ParseError(format!(
                "{}: unexpected token \"{}\". Expected one of: {}",
                locator.locate(location),
                token,
                expected.join(", ")
            )).into(),
            None => error::ErrorKind::ParseError(
                format!("{}: unexpected end of file", locator.name()).into(),
            ).into(),
        },
        ParseError::ExtraToken { token } => error::ErrorKind::ParseError(format!(
            "{}: extra token {}",
            locator.locate(token.0),
            token.1
        )).into(),
        ParseError::User { error } => error,
    })
}

fn module_path<P: AsRef<Path>>(main_path: &Path, module_name: P) -> Option<PathBuf> {
    let path = main_path.join(module_name);
    if path.is_file() {
        Some(path)
    } else {
        None
    }
}

fn load_file<P: AsRef<Path>>(file_name: P) -> error::Result<String> {
    let mut file = File::open(file_name.as_ref())?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;
    Ok(file_contents)
}
