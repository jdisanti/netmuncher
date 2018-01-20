//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

macro_rules! err {
    ($msg:expr) => {
        return Err(error::ErrorKind::NetmuncherError($msg.into()).into());
    };
    ($msg:expr $(, $prm:expr)*) => {
        return Err(error::ErrorKind::NetmuncherError(format!($msg, $($prm,)*)).into());
    };
}

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
    }

    foreign_links {
        Io(::std::io::Error);
        Fmt(::std::fmt::Error);
        SerdeJson(::serde_json::Error);
    }

    errors {
        NetmuncherError(msg: String) {
            description("netmuncher error")
            display("{}", msg)
        }
    }
}
