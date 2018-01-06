//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use std::collections::BTreeMap;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;

use parse;
use parse::src_tag::SrcTag;
use parse::src_unit::{Locator, SrcUnits};
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

    pub fn compile(file_name: &str) -> error::Result<Circuit> {
        let mut units = SrcUnits::new();
        let unit_id = units.push_unit(file_name.into(), load_file(file_name)?);
        let locator = Locator::new(&units, unit_id);

        let components = parse::parse_components(&locator, units.source(unit_id))?;
        Circuit::from_components(&units, components)
    }

    fn from_components(units: &SrcUnits, input: Vec<Component>) -> error::Result<Circuit> {
        let mut components = BTreeMap::new();
        for component in input {
            if components.contains_key(&component.name) {
                return Err(err!(format!(
                    "component {} is defined more than once",
                    component.name
                )));
            }
            component.validate_parameters(units)?;
            component.validate_pins(units)?;
            components.insert(component.name.clone(), component);
        }

        if let Some(main_component) = components.get("Main") {
            let mut ref_gen = ReferenceGenerator::new();
            let mut circuit = Circuit::new();

            if !main_component.pins.is_empty() {
                return Err(err!(format!(
                    "{}: component Main cannot have pins",
                    units.locate(main_component.tag)
                )));
            }

            let empty_net_map = BTreeMap::new();
            let main_instance = Instance::new(SrcTag::invalid(), "Main".into());
            instantiate(
                units,
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

fn load_file(file_name: &str) -> error::Result<String> {
    let mut file = File::open(file_name)?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;
    Ok(file_contents)
}

fn instantiate(
    units: &SrcUnits,
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
                                "{}: cannot find connection named {} on component {}",
                                units.locate(instance.tag),
                                connection_name,
                                component.name
                            )));
                        }
                    }
                } else {
                    return Err(err!(format!(
                        "{}: no connection stated for pin {} on component {}",
                        units.locate(instance.tag),
                        pin.name,
                        component.name
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
            println!("Instantiating {} with {:#?}", component.name, net_map);
            for pin in &component.pins {
                if let Some(mapped_net) = instance.find_connection(&pin.name) {
                    if let Some(net_name) = net_map.get(mapped_net) {
                        new_net_map.insert(pin.name.clone(), net_name.clone());
                    } else {
                        return Err(err!(format!(
                            "{}: cannot find pin or net named {} in instantiation of component {}",
                            units.locate(instance.tag),
                            mapped_net,
                            component.name
                        )));
                    }
                } else {
                    return Err(err!(format!(
                        "{}: unmapped pin named {} in instantiation of component {}",
                        units.locate(instance.tag),
                        pin.name,
                        component.name
                    )));
                }
            }
            for instance in &component.instances {
                instantiate(units, ref_gen, circuit, &components, instance, &new_net_map)?;
            }
        }
        Ok(())
    } else {
        Err(err!(format!(
            "{}: cannot find component definition for {}",
            units.locate(instance.tag),
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
        for instance in &self.instances {
            writeln!(f, "    (comp (ref {})", instance.reference)?;
            writeln!(f, "      (value {})", instance.value)?;
            writeln!(f, "      (footprint {}))", instance.footprint)?;
        }
        writeln!(f, "  )")?;
        writeln!(f, "  (nets")?;
        for (index, net) in self.nets.iter().enumerate() {
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
        Ok(())
    }
}
