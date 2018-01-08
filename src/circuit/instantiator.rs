//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use circuit::{Circuit, ComponentGroup, ComponentInstance, Net, Node};
use circuit::erc;
use error::{self, ErrorKind};
use parse::component::{Component, Instance, Pin, PinType};
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

type GroupBuilderPtr = Rc<RefCell<GroupBuilder>>;

// This is hideous, but I couldn't find a way to make it compile with references and lifetimes
struct GroupBuilder {
    parent: Option<GroupBuilderPtr>,
    name: String,
    components: Vec<String>,
    children: Vec<ComponentGroup>,
}

impl GroupBuilder {
    fn new(parent: Option<GroupBuilderPtr>, name: String) -> GroupBuilderPtr {
        Rc::new(RefCell::new(GroupBuilder {
            parent: parent,
            name: name,
            components: Vec::new(),
            children: Vec::new(),
        }))
    }

    fn component(&mut self, reference: String) {
        self.components.push(reference);
    }

    fn build(group: GroupBuilderPtr) -> Option<ComponentGroup> {
        let this = Rc::try_unwrap(group).ok().unwrap().into_inner();
        let group = ComponentGroup {
            name: this.name,
            components: this.components,
            sub_groups: this.children,
        };
        if let Some(parent) = this.parent {
            parent.borrow_mut().children.push(group);
            None
        } else {
            Some(group)
        }
    }
}

struct InstantiationContext<'a> {
    instance: &'a Instance,
    parent_group: GroupBuilderPtr,
    net_map: &'a BTreeMap<String, String>,
}

impl<'a> InstantiationContext<'a> {
    fn new(
        instance: &'a Instance,
        parent_group: GroupBuilderPtr,
        net_map: &'a BTreeMap<String, String>,
    ) -> InstantiationContext<'a> {
        InstantiationContext {
            instance: instance,
            parent_group: parent_group,
            net_map: net_map,
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

        let empty_net_map = BTreeMap::new();
        let root_group = GroupBuilder::new(None, "root".into());
        let ctx = InstantiationContext::new(instance, root_group, &empty_net_map);
        self.instantiate_internal(&ctx)?;
        self.circuit.root_group = GroupBuilder::build(ctx.parent_group).unwrap();
        Ok(())
    }

    fn instantiate_internal(&mut self, ctx: &InstantiationContext) -> error::Result<()> {
        if let Some(component) = self.components.get(&ctx.instance.name) {
            if component.is_abstract() {
                self.instantiate_abstract(ctx, component)?;
            } else {
                self.instantiate_concrete(ctx, component)?;
            }
            Ok(())
        } else {
            bail!(self.error_component_def_not_found(ctx));
        }
    }

    fn instantiate_abstract(&mut self, ctx: &InstantiationContext, component: &Component) -> error::Result<()> {
        let mut new_net_map = BTreeMap::new();
        let anon_ref = self.ref_gen.next(component.name());
        for net in &component.nets {
            let net_name = format!("{}.{}", net, anon_ref);
            new_net_map.insert(net.clone(), net_name.clone());
            self.circuit.nets.push(Net::new(net_name));
        }

        for pin in &component.pins {
            if self.global_nets.contains(&pin.name) {
                new_net_map.insert(pin.name.clone(), pin.name.clone());
            } else if let Some(mapped_net) = ctx.instance.find_connection(&pin.name) {
                if mapped_net == "noconnect" {
                    new_net_map.insert(pin.name.clone(), "noconnect".into());
                } else if let Some(net_name) = ctx.net_map.get(mapped_net) {
                    new_net_map.insert(pin.name.clone(), net_name.clone());
                } else {
                    bail!(self.error_missing_pin_in_instantiation(ctx, mapped_net, component));
                }
            } else if pin.typ != PinType::NoConnect {
                bail!(self.error_unmapped_pin_in_instantiation(ctx, pin, component));
            }
        }

        let group = GroupBuilder::new(Some(Rc::clone(&ctx.parent_group)), component.name().into());
        for instance in &component.instances {
            let child_ctx = InstantiationContext::new(instance, Rc::clone(&group), &new_net_map);
            self.instantiate_internal(&child_ctx)?;
        }
        GroupBuilder::build(group);
        Ok(())
    }

    fn instantiate_concrete(&mut self, ctx: &InstantiationContext, component: &Component) -> error::Result<()> {
        let reference = self.ref_gen.next(component.prefix());
        ctx.parent_group.borrow_mut().component(reference.clone());

        self.circuit.instances.push(ComponentInstance::new(
            reference.clone(),
            ctx.instance.value().into(),
            component.footprint().into(),
        ));

        for pin in &component.pins {
            if pin.typ == PinType::NoConnect {
                continue;
            }
            let node = Node::new(reference.clone(), pin.num, pin.name.clone(), pin.typ);
            if self.global_nets.contains(&pin.name) {
                self.add_to_net(ctx.instance, &pin.name, node)?;
            } else if let Some(connection_name) = ctx.instance.find_connection(&pin.name) {
                if connection_name != "noconnect" {
                    if self.global_nets.contains(connection_name) {
                        self.add_to_net(ctx.instance, connection_name, node)?;
                    } else if let Some(net_name) = ctx.net_map.get(connection_name) {
                        if net_name == "noconnect" {
                            // no connection; no op
                        } else {
                            self.add_to_net(ctx.instance, net_name, node)?;
                        }
                    } else {
                        bail!(self.error_no_connection_on_component(ctx, connection_name, component));
                    }
                }
            } else {
                bail!(self.error_no_connection_stated(ctx, pin, component));
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

    fn error_component_def_not_found(&self, ctx: &InstantiationContext) -> error::Error {
        ErrorKind::InstantiationError(format!(
            "{}: cannot find component definition for {}",
            self.units.locate(ctx.instance.tag),
            ctx.instance.name
        )).into()
    }

    fn error_unmapped_pin_in_instantiation(
        &self,
        ctx: &InstantiationContext,
        pin: &Pin,
        component: &Component,
    ) -> error::Error {
        ErrorKind::InstantiationError(format!(
            "{}: unmapped pin named {} in instantiation of component {}",
            self.units.locate(ctx.instance.tag),
            pin.name,
            component.name()
        )).into()
    }

    fn error_missing_pin_in_instantiation(
        &self,
        ctx: &InstantiationContext,
        net: &str,
        component: &Component,
    ) -> error::Error {
        ErrorKind::InstantiationError(format!(
            "{}: cannot find pin or net named {} in instantiation of component {}",
            self.units.locate(ctx.instance.tag),
            net,
            component.name()
        )).into()
    }

    fn error_no_connection_on_component(
        &self,
        ctx: &InstantiationContext,
        connection_name: &str,
        component: &Component,
    ) -> error::Error {
        ErrorKind::InstantiationError(format!(
            "{}: cannot find connection named {} on component {}",
            self.units.locate(ctx.instance.tag),
            connection_name,
            component.name()
        )).into()
    }

    fn error_no_connection_stated(&self, ctx: &InstantiationContext, pin: &Pin, component: &Component) -> error::Error {
        ErrorKind::InstantiationError(format!(
            "{}: no connection stated for pin {} on component {}",
            self.units.locate(ctx.instance.tag),
            pin.name,
            component.name()
        )).into()
    }
}
