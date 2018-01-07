//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use std::fmt::Write;

use circuit::{Circuit, SerializeCircuit};
use error;

pub struct KicadNetListSerializer {}

impl KicadNetListSerializer {
    pub fn new() -> KicadNetListSerializer {
        KicadNetListSerializer {}
    }
}

impl SerializeCircuit for KicadNetListSerializer {
    fn serialize(self, circuit: &Circuit) -> error::Result<Vec<u8>> {
        let mut f = String::new();
        writeln!(f, "(export (version D)")?;
        writeln!(f, "  (design")?;
        writeln!(f, "    (source \"netmuncher_generated\")")?;
        writeln!(f, "    (tool \"netmuncher (0.1)\"))")?;
        writeln!(f, "  (components")?;
        for instance in &circuit.instances {
            writeln!(f, "    (comp (ref {})", instance.reference)?;
            writeln!(f, "      (value {})", instance.value)?;
            writeln!(f, "      (footprint {}))", instance.footprint)?;
        }
        writeln!(f, "  )")?;
        writeln!(f, "  (nets")?;
        for (index, net) in circuit.nets.iter().enumerate() {
            writeln!(f, "    (net (code {}) (name \"{}\")", index, net.name)?;
            for node in &net.nodes {
                writeln!(
                    f,
                    "      (node (ref {}) (pin {}))",
                    node.reference, node.pin
                )?;
            }
            writeln!(f, "    )")?;
        }
        writeln!(f, "  ))")?;
        Ok(f.into_bytes())
    }
}
