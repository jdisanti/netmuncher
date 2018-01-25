//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use std::collections::BTreeMap;

mod instantiator;
mod serialize_dot;
mod serialize_kicad;

pub use circuit::serialize_dot::DotSerializer;
pub use circuit::serialize_kicad::KicadNetListSerializer;

use circuit::instantiator::Instantiator;
use error;
use parse;
use parse::component::{Component, Instance, PinNum, PinType};
use parse::source::Sources;

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
    pub reference: String,
    pub pin: PinNum,
    pub pin_name: String,
    pub pin_type: PinType,
}

impl Node {
    pub fn new(reference: String, pin: PinNum, pin_name: String, pin_type: PinType) -> Node {
        Node {
            reference: reference,
            pin: pin,
            pin_name: pin_name,
            pin_type: pin_type,
        }
    }
}

#[derive(Debug)]
pub struct Net {
    pub name: String,
    pub nodes: Vec<Node>,
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
pub struct ComponentGroup {
    pub name: String,
    pub components: Vec<String>,
    pub sub_groups: Vec<ComponentGroup>,
}

impl ComponentGroup {
    pub fn new(name: String) -> ComponentGroup {
        ComponentGroup {
            name: name,
            ..Default::default()
        }
    }
}

#[derive(Default, Debug)]
pub struct Circuit {
    pub instances: Vec<ComponentInstance>,
    pub nets: Vec<Net>,
    pub root_group: ComponentGroup,
}

impl Circuit {
    pub fn new() -> Circuit {
        Default::default()
    }

    pub fn compile(file_name: &str) -> error::Result<Circuit> {
        let result = parse::parse(file_name)?;
        Circuit::from_components(&result.sources, &result.global_nets, result.components)
    }

    fn from_components(
        sources: &Sources,
        global_nets: &Vec<String>,
        input: Vec<Component>,
    ) -> error::Result<Circuit> {
        let components: BTreeMap<String, Component> = input
            .into_iter()
            .map(|c| (String::from(c.name()), c))
            .collect();

        let main_component = components.get("Main").unwrap();
        let mut circuit = Circuit::new();

        let main_instance = Instance::new(main_component.tag, "Main".into());
        Instantiator::new(&mut circuit, &components, global_nets).instantiate(&main_instance)?;

        if circuit.instances.is_empty() {
            err!(
                "{}: empty circuit: no concrete components",
                sources.locate(main_instance.tag)
            );
        }

        for net in &circuit.nets {
            if net.nodes.len() <= 1 {
                err!(
                    "net named {} needs to have more than one connection",
                    net.name
                );
            }
        }

        let mut net_names: Vec<String> = circuit.nets.iter().map(|n| n.name.clone()).collect();
        let len = net_names.len();
        for i in 0..len {
            let replacement = {
                let net_name = &net_names[i];
                if let Some(dot_index) = net_name.find('.') {
                    let simplified_name = &net_name[0..dot_index];
                    if !net_names
                        .iter()
                        .find(|n: &&String| *n == simplified_name)
                        .is_some()
                    {
                        circuit.find_net_mut(&net_name).unwrap().name = simplified_name.into();
                        Some(String::from(simplified_name))
                    } else {
                        None
                    }
                } else {
                    None
                }
            };
            if let Some(replacement) = replacement {
                net_names[i] = replacement;
            }
        }

        Ok(circuit)
    }

    pub fn find_net_mut(&mut self, name: &str) -> Option<&mut Net> {
        self.nets.iter_mut().find(|n: &&mut Net| n.name == name)
    }
}

pub trait SerializeCircuit {
    fn serialize(self, circuit: &Circuit) -> error::Result<Vec<u8>>;
}
