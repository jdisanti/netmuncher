//
// Copyright 2017 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use std::collections::BTreeMap;
use std::fmt;

use parse::component::Component;
use error;

macro_rules! err {
    ($msg:expr) => {
        {
            let e: error::Error = error::ErrorKind::CircuitError($msg.into()).into();
            e
        }
    }
}

#[derive(Debug)]
pub struct Circuit {
    components: BTreeMap<String, Component>,
}

impl Circuit {
    pub fn from_components(input: Vec<Component>) -> error::Result<Circuit> {
        let mut components = BTreeMap::new();
        for component in input {
            if components.contains_key(&component.name) {
                return Err(err!(format!(
                    "component {} is defined more than once",
                    component.name
                )));
            }
            component.validate_parameters()?;
            component.validate_pins()?;
            components.insert(component.name.clone(), component);
        }

        // TODO: validate Main component exists
        // TODO: validate pin mappings
        // TODO: validate all pins assigned or no connected in Main
        // TODO: ERC
        // TODO: generate nets

        Ok(Circuit {
            components: components,
        })
    }
}

impl fmt::Display for Circuit {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        writeln!(f, "(export (version D)")?;
        writeln!(f, "  (design")?;
        writeln!(f, "    (source \"netmuncher_generated\")")?;
        writeln!(f, "    (tool \"netmuncher (0.1)\"))")?;
        writeln!(f, "  (components")?;
        writeln!(f, "  )")?;
        // Looks like we don't need libparts
        /*
        writeln!(f, "  (libparts")?;
        for component in self.components.values() {
            writeln!(f, "    (libpart (lib netmuncher_generated) (part {})", component.name)?;
            writeln!(f, "      (fields")?;
            writeln!(f, "        (field (name Reference) {})", component.reference_prefix)?;
            writeln!(f, "        (field (name Value) {}))", component.name)?;
            writeln!(f, "      (pins")?;
            for pin in component.pins() {
                writeln!(f, "        (pin (num {}) (name {}) (type {}))", pin.num, pin.name, match pin.wire_class {
                    WireClass::Input => "input",
                    WireClass::Output => "output",
                    WireClass::Passive => "passive",
                    WireClass::Power => "power_in",
                    WireClass::Tristate => "3state",
                    WireClass::NoConnect => "NotConnected",
                })?;
            }
            writeln!(f, "      ))")?;
        }
        writeln!(f, "  )")?;*/
        writeln!(f, "  (nets ))")?;
        Ok(())
    }
}
