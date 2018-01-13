//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use std::collections::HashSet;

use error;
use parse::source::Sources;
use parse::component::{Component, Instance, PinType};

macro_rules! err {
    ($msg:expr) => {
        return Err(error::ErrorKind::ValidationError($msg.into()).into());
    };
    ($msg:expr $(, $prm:expr)*) => {
        return Err(error::ErrorKind::ValidationError(format!($msg, $($prm,)*)).into());
    };
}

pub struct Validator<'input> {
    sources: &'input Sources,
    global_nets: &'input Vec<String>,
    components: &'input Vec<Component>,
}

impl<'input> Validator<'input> {
    pub fn new<'a: 'input>(
        sources: &'a Sources,
        global_nets: &'a Vec<String>,
        components: &'a Vec<Component>,
    ) -> Validator<'input> {
        Validator {
            sources: sources,
            global_nets: global_nets,
            components: components,
        }
    }

    pub fn validate(self) -> error::Result<()> {
        self.validate_global_nets()?;
        self.validate_components()?;
        Ok(())
    }

    fn validate_global_nets(&self) -> error::Result<()> {
        let unique_nets: HashSet<&String> = self.global_nets.iter().collect();
        if unique_nets.len() != self.global_nets.len() {
            err!("detected duplicate global nets");
        }
        Ok(())
    }

    fn validate_components(&self) -> error::Result<()> {
        let mut components: HashSet<String> = HashSet::new();
        let mut main_component = None;
        for component in self.components {
            if components.contains(component.name()) {
                err!("component {} is defined more than once", component.name());
            }
            if component.name() == "Main" {
                main_component = Some(component);
            }
            components.insert(component.name().into());

            component.validate_parameters(self.sources)?;
            component.validate_units(self.sources)?;
            self.validate_component(component)?;
        }
        self.validate_main(main_component)?;
        Ok(())
    }

    fn validate_main(&self, main_component: Option<&Component>) -> error::Result<()> {
        if let Some(main) = main_component {
            if !main.is_abstract() {
                err!("{}: component Main must be abstract", self.sources.locate(main.tag));
            }
            if !main.abstract_pins().is_empty() {
                err!("{}: component Main cannot have pins", self.sources.locate(main.tag));
            }
        } else {
            err!("missing component Main");
        }
        Ok(())
    }

    fn validate_component(&self, component: &Component) -> error::Result<()> {
        if component.is_abstract() {
            for instance in &component.instances {
                self.validate_instance(component, instance)?;
            }
        }
        Ok(())
    }

    fn validate_instance(&self, parent_component: &Component, instance: &Instance) -> error::Result<()> {
        if let Some(component) = self.find_component(&instance.name) {
            let unit = component.first_unit();
            for pin in &unit.pins {
                println!("self.global_nets: {:?}, pin: {}", self.global_nets, pin.name);
                if self.global_nets.contains(&pin.name) {
                    continue;
                }
                if let Some(mapping) = instance.find_connection(&pin.name) {
                    if pin.typ == PinType::NoConnect && mapping != "noconnect" {
                        err!(
                            "{}: cannot connect noconnect pin named {} in instantiation of component {}",
                            self.sources.locate(instance.tag),
                            pin.name,
                            component.name()
                        );
                    }
                    if self.global_nets.contains(mapping) {
                        continue;
                    }
                    if mapping != "noconnect" && !parent_component.nets.exists(mapping) &&
                            parent_component.abstract_pins().find_by_name(mapping).is_none() {
                        err!(
                            "{}: cannot find pin or net named {} in instantiation of component {}",
                            self.sources.locate(instance.tag),
                            mapping,
                            component.name()
                        );
                    }
                } else if pin.typ != PinType::NoConnect {
                    err!(
                        "{}: no connection stated for pin {} on component {}",
                        self.sources.locate(instance.tag),
                        pin.name,
                        component.name()
                    );
                }
            }
        } else {
            err!(
                "{}: cannot find component definition for {}",
                self.sources.locate(instance.tag),
                instance.name
            );
        }
        Ok(())
    }

    fn find_component(&self, name: &str) -> Option<&Component> {
        self.components.iter().find(|c| c.name() == name)
    }
}