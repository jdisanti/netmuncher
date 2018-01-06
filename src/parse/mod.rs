//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use lalrpop_util::ParseError;

use error;

#[cfg_attr(rustfmt, rustfmt_skip)]
mod grammar;

pub mod component;
pub mod token;
pub mod src_tag;
pub mod src_unit;

use parse::component::Component;
use parse::src_unit::Locator;

pub fn parse_components(locator: &Locator, source: &str) -> error::Result<Vec<Component>> {
    let tokens = token::tokenize(locator, source)?;
    grammar::parse_Components(&locator, tokens.into_iter()).map_err(|e| match e {
        ParseError::InvalidToken { location } => {
            error::ErrorKind::ParseError(format!("{}: invalid token", locator.locate(location))).into()
        }
        ParseError::UnrecognizedToken { token, expected } => match token {
            Some((location, token, _)) => error::ErrorKind::ParseError(format!(
                "{}: unexpected token \"{}\". Expected one of: {}",
                locator.locate(location),
                token,
                expected.join(",")
            )).into(),
            None => error::ErrorKind::ParseError(format!(
                "{}: unexpected end of file",
                locator.name()
            ).into()).into(),
        },
        ParseError::ExtraToken { token } => error::ErrorKind::ParseError(format!(
            "{}: extra token {}",
            locator.locate(token.0),
            token.1
        )).into(),
        ParseError::User { error } => error,
    })
}
