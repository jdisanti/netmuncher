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

use parse::component::{Component, Instance, PinNum, PinType};
use error;

macro_rules! err {
    ($msg:expr) => {
        {
            let e: error::Error = error::ErrorKind::CircuitError($msg.into()).into();
            e
        }
    }
}

struct ReferenceGenerator {
    counts: BTreeMap<String, usize>,
}

impl ReferenceGenerator {
    fn new() -> ReferenceGenerator {
        ReferenceGenerator {
            counts: BTreeMap::new(),
        }
    }

    fn next(&mut self, prefix: &str) -> String {
        if !self.counts.contains_key(prefix) {
            self.counts.insert(String::from(prefix), 0);
        }
        if let Some(value) = self.counts.get_mut(prefix) {
            *value += 1;
            let reference = format!("{}{}", prefix, value);
            reference
        } else {
            unreachable!()
        }
    }
}

#[derive(Debug)]
pub struct ComponentInstance {
    reference: String,
    value: String,
    footprint: String,
}

impl ComponentInstance {
    pub fn new(reference: String, value: String, footprint: String) -> ComponentInstance {
        ComponentInstance {
            reference: reference,
            value: value,
            footprint: footprint,
        }
    }
}

#[derive(Debug)]
pub struct Node {
    reference: String,
    pin: PinNum,
}

impl Node {
    pub fn new(reference: String, pin: PinNum) -> Node {
        Node {
            reference: reference,
            pin: pin,
        }
    }
}

#[derive(Debug)]
pub struct Net {
    name: String,
    nodes: Vec<Node>,
}

impl Net {
    pub fn new(name: String) -> Net {
        Net {
            name: name,
            nodes: Vec::new(),
        }
    }
}

#[derive(Default, Debug)]
pub struct Circuit {
    instances: Vec<ComponentInstance>,
    nets: Vec<Net>,
}

impl Circuit {
    pub fn new() -> Circuit {
        Default::default()
    }

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

        if let Some(main_component) = components.get("Main") {
            let mut ref_gen = ReferenceGenerator::new();
            let mut circuit = Circuit::new();

            if !main_component.pins.is_empty() {
                return Err(err!("component Main cannot have pins"));
            }

            let empty_net_map = BTreeMap::new();
            let main_instance = Instance::new("Main".into());
            instantiate(
                &mut ref_gen,
                &mut circuit,
                &components,
                &main_instance,
                &empty_net_map,
            )?;

            let net_names: Vec<String> = circuit.nets.iter().map(|n| n.name.clone()).collect();
            for net_name in &net_names {
                if let Some(dot_index) = net_name.find('.') {
                    let simplified_name = &net_name[0..dot_index];
                    if !net_names
                        .iter()
                        .find(|n: &&String| *n == simplified_name)
                        .is_some()
                    {
                        circuit.find_net_mut(&net_name).unwrap().name = simplified_name.into();
                    }
                }
            }

            // TODO: ERC

            Ok(circuit)
        } else {
            Err(err!("missing component Main"))
        }
    }

    fn find_net_mut(&mut self, name: &str) -> Option<&mut Net> {
        self.nets.iter_mut().find(|n: &&mut Net| n.name == name)
    }
}

fn instantiate(
    ref_gen: &mut ReferenceGenerator,
    circuit: &mut Circuit,
    components: &BTreeMap<String, Component>,
    instance: &Instance,
    net_map: &BTreeMap<String, String>,
) -> error::Result<()> {
    if let Some(component) = components.get(&instance.name) {
        if let Some(ref prefix) = component.prefix {
            let reference = ref_gen.next(prefix);
            circuit.instances.push(ComponentInstance::new(
                reference.clone(),
                component.name.clone(),
                component.footprint.as_ref().unwrap().clone(),
            ));
            for pin in &component.pins {
                if pin.typ == PinType::NoConnect {
                    continue;
                }
                if let Some(&(_, ref connection_name)) = instance
                    .connections
                    .iter()
                    .find(|&&(ref pin_name, _)| **pin_name == pin.name)
                {
                    if connection_name != "noconnect" {
                        if let Some(net_name) = net_map.get(connection_name) {
                            if let Some(net) = circuit.find_net_mut(net_name) {
                                net.nodes.push(Node::new(reference.clone(), pin.num));
                            } else {
                                unreachable!()
                            }
                        } else {
                            return Err(err!(format!(
                                "cannot find connection named {} on component {}",
                                connection_name, component.name
                            )));
                        }
                    }
                } else {
                    return Err(err!(format!(
                        "no connection stated for pin {} on component {}",
                        pin.name, component.name
                    )));
                }
            }
        } else {
            let mut new_net_map = BTreeMap::new();
            let anon_ref = ref_gen.next(&component.name);
            for net in &component.nets {
                let net_name = format!("{}.{}", net, anon_ref);
                new_net_map.insert(net.clone(), net_name.clone());
                circuit.nets.push(Net::new(net_name));
            }
            for pin in &component.pins {
                if let Some(net_name) = net_map.get(&pin.name) {
                    new_net_map.insert(pin.name.clone(), net_name.clone());
                } else {
                    return Err(err!(format!(
                        "unmapped pin named {} in instantiation of component {}",
                        pin.name, component.name
                    )));
                }
            }
            for instance in &component.instances {
                instantiate(ref_gen, circuit, &components, instance, &new_net_map)?;
            }
        }
        Ok(())
    } else {
        Err(err!(format!(
            "cannot find component definition for {}",
            instance.name
        )))
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
