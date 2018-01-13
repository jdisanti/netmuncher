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
use error;
use parse::component::{Component, Instance, PinMap, PinType, Unit};
use parse::source::Sources;

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

struct UnitRef<'a> {
    reference: String,
    unit: &'a Unit,
}

impl<'a> UnitRef<'a> {
    fn new(reference: String, unit: &'a Unit) -> UnitRef<'a> {
        UnitRef {
            reference: reference,
            unit: unit,
        }
    }
}

#[derive(Default)]
struct UnitTracker<'a> {
    available_units: BTreeMap<String, Vec<UnitRef<'a>>>,
}

impl<'a> UnitTracker<'a> {
    fn new() -> UnitTracker<'a> {
        Default::default()
    }

    fn add_component(&mut self, reference: &str, component: &'a Component) {
        assert!(!self.available_units.contains_key(component.name()));
        self.available_units.insert(
            component.name().into(),
            component
                .units
                .iter()
                .map(|c| UnitRef::new(reference.into(), c))
                .collect(),
        );
    }

    fn next_unit<'b>(&'b mut self, component: &'a Component) -> Option<UnitRef<'a>> {
        assert!(component.has_units());
        if !self.available_units.contains_key(component.name()) {
            None
        } else {
            let mut units = self.available_units.remove(component.name()).unwrap();
            let unit = units.remove(0);
            if !units.is_empty() {
                self.available_units.insert(component.name().into(), units);
            }
            Some(unit)
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
    sources: &'input Sources,
    components: &'input BTreeMap<String, Component>,
    global_nets: &'input [String],
    ref_gen: ReferenceGenerator,
    unit_tracker: UnitTracker<'input>,
}

impl<'input> Instantiator<'input> {
    pub fn new(
        circuit: &'input mut Circuit,
        sources: &'input Sources,
        components: &'input BTreeMap<String, Component>,
        global_nets: &'input [String],
    ) -> Instantiator<'input> {
        Instantiator {
            circuit: circuit,
            sources: sources,
            components: components,
            global_nets: global_nets,
            ref_gen: ReferenceGenerator::new(),
            unit_tracker: UnitTracker::new(),
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
            } else if component.has_units() {
                self.instantiate_unit(ctx, component)?;
            } else {
                self.instantiate_concrete(ctx, component)?;
            }
            Ok(())
        } else {
            unreachable!("validation should catch this");
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

        for pin in component.abstract_pins() {
            if self.global_nets.contains(&pin.name) {
                new_net_map.insert(pin.name.clone(), pin.name.clone());
            } else if let Some(mapped_net) = ctx.instance.find_connection(&pin.name) {
                if mapped_net == "noconnect" {
                    new_net_map.insert(pin.name.clone(), "noconnect".into());
                } else if let Some(net_name) = ctx.net_map.get(mapped_net) {
                    new_net_map.insert(pin.name.clone(), net_name.clone());
                } else {
                    unreachable!("validation should catch this");
                }
            } else if pin.typ != PinType::NoConnect {
                unreachable!("validation should catch this");
            }
        }

        let group = GroupBuilder::new(Some(Rc::clone(&ctx.parent_group)), anon_ref);
        for instance in &component.instances {
            let child_ctx = InstantiationContext::new(instance, Rc::clone(&group), &new_net_map);
            self.instantiate_internal(&child_ctx)?;
        }
        GroupBuilder::build(group);
        Ok(())
    }

    fn instantiate_unit(&mut self, ctx: &InstantiationContext, component: &'input Component) -> error::Result<()> {
        if let Some(unit_ref) = self.unit_tracker.next_unit(component) {
            self.instantiate_pins(ctx, &unit_ref.reference, &unit_ref.unit.pins)?;
        } else {
            let reference = self.ref_gen.next(component.prefix());
            ctx.parent_group.borrow_mut().component(reference.clone());

            self.circuit.instances.push(ComponentInstance::new(
                reference.clone(),
                ctx.instance
                    .value()
                    .unwrap_or(component.default_value())
                    .into(),
                component.footprint().into(),
            ));

            self.unit_tracker.add_component(&reference, component);
            let unit_ref = self.unit_tracker.next_unit(component).unwrap();
            self.instantiate_pins(ctx, &reference, &unit_ref.unit.pins)?;
        }
        Ok(())
    }

    fn instantiate_pins(
        &mut self,
        ctx: &InstantiationContext,
        reference: &str,
        pins: &PinMap,
    ) -> error::Result<()> {
        for pin in pins {
            if pin.typ == PinType::NoConnect {
                continue;
            }
            let node = Node::new(reference.into(), pin.num, pin.name.clone(), pin.typ);
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
                        unreachable!("validation should catch this");
                    }
                }
            } else {
                unreachable!("validation should catch this");
            };
        }
        Ok(())
    }

    fn instantiate_concrete(&mut self, ctx: &InstantiationContext, component: &Component) -> error::Result<()> {
        let reference = self.ref_gen.next(component.prefix());
        ctx.parent_group.borrow_mut().component(reference.clone());

        self.circuit.instances.push(ComponentInstance::new(
            reference.clone(),
            ctx.instance
                .value()
                .unwrap_or(component.default_value())
                .into(),
            component.footprint().into(),
        ));

        self.instantiate_pins(ctx, &reference, component.pins())?;
        Ok(())
    }

    fn add_to_net(&mut self, instance: &Instance, net: &str, node: Node) -> error::Result<()> {
        if let Some(net) = self.circuit.find_net_mut(net) {
            erc::check_connection(&self.sources, instance, &node, &net.nodes)?;
            net.nodes.push(node);
        } else {
            unreachable!()
        }
        Ok(())
    }
}
