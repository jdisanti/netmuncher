//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use std::collections::BTreeMap;

pub struct ReferenceGenerator {
    separator: String,
    counts: BTreeMap<String, usize>,
}

impl ReferenceGenerator {
    pub fn new<S: Into<String>>(separator: S) -> ReferenceGenerator {
        ReferenceGenerator {
            separator: separator.into(),
            counts: BTreeMap::new(),
        }
    }

    pub fn next(&mut self, prefix: &str) -> String {
        if !self.counts.contains_key(prefix) {
            self.counts.insert(String::from(prefix), 0);
        }
        if let Some(value) = self.counts.get_mut(prefix) {
            *value += 1;
            let reference = format!("{}{}{}", prefix, self.separator, value);
            reference
        } else {
            unreachable!()
        }
    }
}
