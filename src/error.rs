//
// Copyright 2017 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
    }

    foreign_links {
    }

    errors {
        CircuitError(msg: String) {
            description("circuit error")
            display("{}", msg)
        }
        ComponentError(msg: String) {
            description("component error")
            display("{}", msg)
        }
        ParseError(msg: String) {
            description("parse error")
            display("{}", msg)
        }
    }
}
