//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

mod erc;
mod instantiator;
mod serialize_kicad;
mod serialize_dot;

pub use circuit::serialize_dot::DotSerializer;
pub use circuit::serialize_kicad::KicadNetListSerializer;

use circuit::instantiator::Instantiator;
use error::{self, ErrorKind};
use parse;
use parse::component::{Component, Instance, PinNum, PinType};
use parse::src_unit::{Locator, SrcUnits};

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
        let main_file = Path::new(file_name).file_name().unwrap();
        let main_path = Path::new(file_name).parent().unwrap();

        let mut units = SrcUnits::new();

        let mut modules_to_require: Vec<PathBuf> = Vec::new();
        let mut modules_required: Vec<PathBuf> = Vec::new();
        modules_to_require.push(module_path(&main_path, &main_file).unwrap());

        let mut global_nets: Vec<String> = Vec::new();
        let mut components: Vec<Component> = Vec::new();
        while let Some(path) = modules_to_require.pop() {
            if !modules_required.contains(&path) {
                modules_required.push(path.clone());
                let unit_id = units.push_unit(path.to_str().unwrap().into(), load_file(path)?);
                let locator = Locator::new(&units, unit_id);
                let parse_result = parse::parse_components(&locator, units.source(unit_id))?;
                modules_to_require.extend(
                    parse_result
                        .requires
                        .into_iter()
                        .filter_map(|r| module_path(&main_path, &r)),
                );
                global_nets.extend(parse_result.global_nets.into_iter());
                components.extend(parse_result.components.into_iter());
            }
        }

        let original_net_size = global_nets.len();
        global_nets.sort();
        global_nets.dedup();
        if original_net_size != global_nets.len() {
            bail!(ErrorKind::CircuitError(
                "detected duplicate global nets".into()
            ));
        }

        Circuit::from_components(&units, &global_nets, components)
    }

    fn from_components(units: &SrcUnits, global_nets: &Vec<String>, input: Vec<Component>) -> error::Result<Circuit> {
        let mut components = BTreeMap::new();
        for component in input {
            if components.contains_key(component.name()) {
                bail!(ErrorKind::CircuitError(format!(
                    "component {} is defined more than once",
                    component.name()
                )));
            }
            component.validate_parameters(units)?;
            component.validate_pins(units)?;
            components.insert(component.name().into(), component);
        }

        if let Some(main_component) = components.get("Main") {
            let mut circuit = Circuit::new();

            if !main_component.pins.is_empty() {
                bail!(ErrorKind::CircuitError(format!(
                    "{}: component Main cannot have pins",
                    units.locate(main_component.tag)
                )));
            }

            let main_instance = Instance::new(main_component.tag, "Main".into());
            Instantiator::new(&mut circuit, units, &components, global_nets).instantiate(&main_instance)?;

            if circuit.instances.is_empty() {
                bail!(ErrorKind::CircuitError(format!(
                    "{}: empty circuit: no concrete components",
                    units.locate(main_instance.tag),
                )));
            }

            for net in &circuit.nets {
                if net.nodes.len() <= 1 {
                    bail!(ErrorKind::CircuitError(format!(
                        "net named {} needs to have more than one connection",
                        net.name,
                    )));
                }
            }

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

            Ok(circuit)
        } else {
            bail!(ErrorKind::CircuitError("missing component Main".into()));
        }
    }

    pub fn find_net_mut(&mut self, name: &str) -> Option<&mut Net> {
        self.nets.iter_mut().find(|n: &&mut Net| n.name == name)
    }
}

fn load_file<P: AsRef<Path>>(file_name: P) -> error::Result<String> {
    let mut file = File::open(file_name.as_ref())?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;
    Ok(file_contents)
}

fn module_path<P: AsRef<Path>>(main_path: &Path, module_name: P) -> Option<PathBuf> {
    let path = main_path.join(module_name);
    if path.is_file() {
        Some(path)
    } else {
        None
    }
}

pub trait SerializeCircuit {
    fn serialize(self, circuit: &Circuit) -> error::Result<Vec<u8>>;
}
