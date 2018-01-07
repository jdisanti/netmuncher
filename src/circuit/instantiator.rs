//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use std::collections::BTreeMap;

use circuit::{Circuit, ComponentInstance, Net, Node};
use circuit::erc;
use error::{self, ErrorKind};
use parse::component::{Component, Instance, PinType};
use parse::src_unit::SrcUnits;

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

pub struct Instantiator<'input> {
    circuit: &'input mut Circuit,
    units: &'input SrcUnits,
    components: &'input BTreeMap<String, Component>,
    global_nets: &'input [String],
    ref_gen: ReferenceGenerator,
}

impl<'input> Instantiator<'input> {
    pub fn new(
        circuit: &'input mut Circuit,
        units: &'input SrcUnits,
        components: &'input BTreeMap<String, Component>,
        global_nets: &'input [String],
    ) -> Instantiator<'input> {
        Instantiator {
            circuit: circuit,
            units: units,
            components: components,
            global_nets: global_nets,
            ref_gen: ReferenceGenerator::new(),
        }
    }

    pub fn instantiate(mut self, instance: &Instance) -> error::Result<()> {
        for global_net in self.global_nets {
            self.circuit.nets.push(Net::new(global_net.clone()));
        }

        self.instantiate_internal(instance, &BTreeMap::new())
    }

    fn instantiate_internal(&mut self, instance: &Instance, net_map: &BTreeMap<String, String>) -> error::Result<()> {
        if let Some(component) = self.components.get(&instance.name) {
            // Concrete components should always have a prefix and footprint
            if let Some(ref prefix) = component.prefix {
                self.instantiate_concrete(instance, net_map, component, prefix)?;
            } else {
                self.instantiate_abstract(instance, net_map, component)?;
            }
            Ok(())
        } else {
            bail!(ErrorKind::InstantiationError(format!(
                "{}: cannot find component definition for {}",
                self.units.locate(instance.tag),
                instance.name
            )))
        }
    }

    fn instantiate_abstract(
        &mut self,
        instance: &Instance,
        net_map: &BTreeMap<String, String>,
        component: &Component,
    ) -> error::Result<()> {
        let mut new_net_map = BTreeMap::new();
        let anon_ref = self.ref_gen.next(&component.name);
        for net in &component.nets {
            let net_name = format!("{}.{}", net, anon_ref);
            new_net_map.insert(net.clone(), net_name.clone());
            self.circuit.nets.push(Net::new(net_name));
        }
        for pin in &component.pins {
            if self.global_nets.contains(&pin.name) {
                new_net_map.insert(pin.name.clone(), pin.name.clone());
            } else if let Some(mapped_net) = instance.find_connection(&pin.name) {
                if mapped_net == "noconnect" {
                    new_net_map.insert(pin.name.clone(), "noconnect".into());
                } else if let Some(net_name) = net_map.get(mapped_net) {
                    new_net_map.insert(pin.name.clone(), net_name.clone());
                } else {
                    bail!(ErrorKind::InstantiationError(format!(
                        "{}: cannot find pin or net named {} in instantiation of component {}",
                        self.units.locate(instance.tag),
                        mapped_net,
                        component.name
                    )));
                }
            } else if pin.typ != PinType::NoConnect {
                bail!(ErrorKind::InstantiationError(format!(
                    "{}: unmapped pin named {} in instantiation of component {}",
                    self.units.locate(instance.tag),
                    pin.name,
                    component.name
                )));
            }
        }
        for instance in &component.instances {
            self.instantiate_internal(instance, &new_net_map)?;
        }
        Ok(())
    }

    fn instantiate_concrete(
        &mut self,
        instance: &Instance,
        net_map: &BTreeMap<String, String>,
        component: &Component,
        prefix: &str,
    ) -> error::Result<()> {
        let reference = self.ref_gen.next(prefix);
        self.circuit.instances.push(ComponentInstance::new(
            reference.clone(),
            instance
                .value
                .as_ref()
                .unwrap_or_else(|| &component.name)
                .clone(),
            component.footprint.as_ref().unwrap().clone(),
        ));
        for pin in &component.pins {
            if pin.typ == PinType::NoConnect {
                continue;
            }
            let node = Node::new(reference.clone(), pin.num, pin.name.clone(), pin.typ);
            if self.global_nets.contains(&pin.name) {
                self.add_to_net(instance, &pin.name, node)?;
            } else if let Some(&(_, ref connection_name)) = instance
                .connections
                .iter()
                .find(|&&(ref pin_name, _)| **pin_name == pin.name)
            {
                if connection_name != "noconnect" {
                    if self.global_nets.contains(connection_name) {
                        self.add_to_net(instance, connection_name, node)?;
                    } else if let Some(net_name) = net_map.get(connection_name) {
                        if net_name == "noconnect" {
                            // no connection; no op
                        } else {
                            self.add_to_net(instance, net_name, node)?;
                        }
                    } else {
                        bail!(ErrorKind::InstantiationError(format!(
                            "{}: cannot find connection named {} on component {}",
                            self.units.locate(instance.tag),
                            connection_name,
                            component.name
                        )));
                    }
                }
            } else {
                bail!(ErrorKind::InstantiationError(format!(
                    "{}: no connection stated for pin {} on component {}",
                    self.units.locate(instance.tag),
                    pin.name,
                    component.name
                )));
            };
        }
        Ok(())
    }

    fn add_to_net(&mut self, instance: &Instance, net: &str, node: Node) -> error::Result<()> {
        if let Some(net) = self.circuit.find_net_mut(net) {
            erc::check_connection(&self.units, instance, &node, &net.nodes)?;
            net.nodes.push(node);
        } else {
            unreachable!()
        }
        Ok(())
    }
}
