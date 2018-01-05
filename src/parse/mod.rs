//
// Copyright 2017 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use error;

#[cfg_attr(rustfmt, rustfmt_skip)]
mod grammar;

pub mod component;
pub mod token;

use parse::component::Component;

pub fn parse_components(source: &str) -> error::Result<Vec<Component>> {
    let tokens = token::tokenize(&source)?;
    grammar::parse_Components(tokens.into_iter()).map_err(|e| error::ErrorKind::ParseError(format!("{:?}", e)).into())
}
