//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use serde_json;

use circuit::{Circuit, SerializeCircuit};
use error;

pub struct JsonSerializer {}

impl JsonSerializer {
    pub fn new() -> JsonSerializer {
        JsonSerializer {}
    }
}

impl SerializeCircuit for JsonSerializer {
    fn serialize(self, circuit: &Circuit) -> error::Result<Vec<u8>> {
        let result = serde_json::to_vec_pretty(circuit)?;
        Ok(result)
    }
}
