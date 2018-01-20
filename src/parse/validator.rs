//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use std::collections::{BTreeMap, HashSet};

use error;
use parse::component::{Component, Instance, Pin, PinType};
use parse::source::Sources;

pub struct Validator<'input> {
    sources: &'input Sources,
    global_nets: &'input Vec<String>,
    components: &'input Vec<Component>,
    global_net_pins: BTreeMap<&'input String, Vec<(&'input Instance, &'input Pin)>>,
    local_net_pins: BTreeMap<&'input String, Vec<(&'input Instance, &'input Pin)>>,
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
            global_net_pins: BTreeMap::new(),
            local_net_pins: BTreeMap::new(),
        }
    }

    pub fn validate(mut self) -> error::Result<()> {
        self.validate_global_nets()?;
        self.validate_components()?;
        Ok(())
    }

    fn validate_global_nets(&mut self) -> error::Result<()> {
        let unique_nets: HashSet<&String> = self.global_nets.iter().collect();
        if unique_nets.len() != self.global_nets.len() {
            err!("detected duplicate global nets");
        }
        Ok(())
    }

    fn validate_components(&mut self) -> error::Result<()> {
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
        self.validate_nets(&self.global_net_pins)?;
        self.validate_main(main_component)?;
        Ok(())
    }

    fn validate_main(&mut self, main_component: Option<&Component>) -> error::Result<()> {
        if let Some(main) = main_component {
            if !main.is_abstract() {
                err!(
                    "{}: component Main must be abstract",
                    self.sources.locate(main.tag)
                );
            }
            if !main.abstract_pins().is_empty() {
                err!(
                    "{}: component Main cannot have pins",
                    self.sources.locate(main.tag)
                );
            }
        } else {
            err!("missing component Main");
        }
        Ok(())
    }

    fn validate_component(&mut self, component: &'input Component) -> error::Result<()> {
        if component.is_abstract() {
            let mut net_pins: BTreeMap<
                &'input String,
                Vec<(&'input Instance, &'input Pin)>,
            > = BTreeMap::new();
            for instance in &component.instances {
                self.validate_instance(component, instance)?;
                for (net_name, pins) in &self.local_net_pins {
                    if !net_pins.contains_key(net_name) {
                        net_pins.insert(net_name, Vec::new());
                    }
                    net_pins.get_mut(net_name).unwrap().extend(pins);
                }
                self.local_net_pins.clear();
            }
            self.validate_nets(&net_pins)?;
        }
        Ok(())
    }

    fn validate_nets(
        &self,
        net_pins: &BTreeMap<&'input String, Vec<(&Instance, &Pin)>>,
    ) -> error::Result<()> {
        for (net_name, pins) in net_pins {
            for &(first_instance, first_pin) in pins {
                for &(second_instance, second_pin) in pins {
                    if first_instance.tag == second_instance.tag && first_pin == second_pin {
                        continue;
                    }
                    self.electronic_rules_check(
                        net_name,
                        first_instance,
                        first_pin,
                        second_instance,
                        second_pin,
                    )?;
                }
            }
        }
        Ok(())
    }

    fn add_global_net_pin(
        &mut self,
        net: &'input String,
        instance: &'input Instance,
        pin: &'input Pin,
    ) {
        if !self.global_net_pins.contains_key(net) {
            self.global_net_pins.insert(net, Vec::new());
        }
        self.global_net_pins
            .get_mut(net)
            .unwrap()
            .push((instance, pin));
    }

    fn add_local_net_pin(
        &mut self,
        net: &'input String,
        instance: &'input Instance,
        pin: &'input Pin,
    ) {
        if !self.local_net_pins.contains_key(net) {
            self.local_net_pins.insert(net, Vec::new());
        }
        self.local_net_pins
            .get_mut(net)
            .unwrap()
            .push((instance, pin));
    }

    fn validate_instance(
        &mut self,
        parent_component: &'input Component,
        instance: &'input Instance,
    ) -> error::Result<()> {
        if let Some(component) = self.find_component(&instance.name) {
            let unit = component.first_unit();
            for pin in &unit.pins {
                if self.global_nets.contains(&pin.name) {
                    if !component.is_abstract() {
                        self.add_global_net_pin(&pin.name, instance, pin);
                    }
                    continue;
                }
                if let Some(mapping) = instance.find_connection(&pin.name) {
                    if pin.typ == PinType::NoConnect && mapping != "noconnect" {
                        err!(
                            "{}: cannot connect noconnect pin named {} in instantiation of \
                             component {}",
                            self.sources.locate(instance.tag),
                            pin.name,
                            component.name()
                        );
                    }
                    if self.global_nets.contains(mapping) {
                        if !component.is_abstract() {
                            self.add_global_net_pin(mapping, instance, pin);
                        }
                        continue;
                    }
                    if mapping != "noconnect" {
                        if let Some(connected_pin) =
                            parent_component.abstract_pins().find_by_name(mapping)
                        {
                            self.parameter_rules_check(instance, connected_pin, pin)?;
                        } else if parent_component.nets.exists(mapping) {
                            if !component.is_abstract() {
                                self.add_local_net_pin(mapping, instance, pin);
                            }
                        } else {
                            err!(
                                "{}: cannot find pin or net named {} in instantiation of \
                                 component {}",
                                self.sources.locate(instance.tag),
                                mapping,
                                component.name()
                            );
                        }
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

    fn parameter_rules_check(
        &self,
        instance: &Instance,
        instance_pin: &Pin,
        other_pin: &Pin,
    ) -> error::Result<()> {
        match check_parameter_connection(instance_pin.typ, other_pin.typ) {
            ERCResult::Valid => Ok(()),
            r @ ERCResult::Warning | r @ ERCResult::Error => {
                let error = error::ErrorKind::NetmuncherError(format!(
                    "{}: in instantiation of {}, pin {} ({:?}) mapped to {} ({:?})",
                    self.sources.locate(instance.tag),
                    instance.name,
                    instance_pin.name,
                    instance_pin.typ,
                    other_pin.name,
                    other_pin.typ
                ));
                if r == ERCResult::Warning {
                    println!("WARN: {}", error);
                    Ok(())
                } else {
                    Err(error.into())
                }
            }
        }
    }

    fn electronic_rules_check(
        &self,
        net: &str,
        first_instance: &Instance,
        first_pin: &Pin,
        second_instance: &Instance,
        second_pin: &Pin,
    ) -> error::Result<()> {
        match check_electric_connection(first_pin.typ, second_pin.typ) {
            ERCResult::Valid => Ok(()),
            r @ ERCResult::Warning | r @ ERCResult::Error => {
                let error = error::ErrorKind::NetmuncherError(format!(
                    "{}: in instantiation of {}, pin {} ({:?}) is connected by net {} to pin {} \
                     ({:?}) of instantiation {} at {}",
                    self.sources.locate(first_instance.tag),
                    first_instance.name,
                    first_pin.name,
                    first_pin.typ,
                    net,
                    second_pin.name,
                    second_pin.typ,
                    second_instance.name,
                    self.sources.locate(second_instance.tag),
                ));
                if r == ERCResult::Warning {
                    println!("WARN: {}", error);
                    Ok(())
                } else {
                    Err(error.into())
                }
            }
        }
    }

    fn find_component(&self, name: &str) -> Option<&'input Component> {
        self.components.iter().find(|c| c.name() == name)
    }
}

#[derive(Debug, Eq, PartialEq)]
enum ERCResult {
    Valid,
    Warning,
    Error,
}

fn check_parameter_connection(parent_pin: PinType, child_pin: PinType) -> ERCResult {
    use self::ERCResult::*;
    use parse::component::PinType::*;

    match parent_pin {
        Input => match child_pin {
            Input | Passive | Bidirectional => Valid,
            Tristate | PowerIn => Warning,
            PowerOut | Output | NoConnect => Error,
        },
        Output => match child_pin {
            Input | Output | Passive | PowerOut | Bidirectional => Valid,
            Tristate | PowerIn => Warning,
            NoConnect => Error,
        },
        Passive => match child_pin {
            Input | Output | Passive | PowerIn | PowerOut | Tristate | Bidirectional => Valid,
            NoConnect => Error,
        },
        PowerIn => match child_pin {
            Input | Passive | PowerIn | Bidirectional => Valid,
            Tristate | PowerOut | Output | NoConnect => Error,
        },
        PowerOut => match child_pin {
            PowerIn | Input | Passive | PowerOut | Output => Valid,
            Bidirectional => Warning,
            Tristate | NoConnect => Error,
        },
        Tristate => match child_pin {
            Tristate | Input | Passive | Bidirectional => Valid,
            PowerIn | Output => Warning,
            PowerOut | NoConnect => Error,
        },
        Bidirectional => match child_pin {
            Bidirectional | Input | Output | Passive | PowerIn | Tristate => Valid,
            PowerOut => Warning,
            NoConnect => Error,
        },
        NoConnect => Error,
    }
}

fn check_electric_connection(first: PinType, second: PinType) -> ERCResult {
    use self::ERCResult::*;
    use parse::component::PinType::*;

    match first {
        Input => match second {
            Input | Output | Passive | PowerIn | PowerOut | Tristate | Bidirectional => Valid,
            NoConnect => Error,
        },
        Output => match second {
            Input | Passive | PowerIn | Bidirectional => Valid,
            Tristate => Warning,
            NoConnect | PowerOut | Output => Error,
        },
        Passive => match second {
            Input | Output | Passive | PowerIn | PowerOut | Tristate | Bidirectional => Valid,
            NoConnect => Error,
        },
        PowerIn => match second {
            Input | Output | Passive | PowerIn | PowerOut | Bidirectional => Valid,
            Tristate => Warning,
            NoConnect => Error,
        },
        PowerOut => match second {
            Input | Passive | PowerIn => Valid,
            Bidirectional | Tristate => Warning,
            NoConnect | Output | PowerOut => Error,
        },
        Tristate => match second {
            Input | Tristate | Passive | Bidirectional => Valid,
            Output | PowerIn | PowerOut => Warning,
            NoConnect => Error,
        },
        Bidirectional => match second {
            Bidirectional | Input | Output | Passive | PowerIn | Tristate => Valid,
            PowerOut => Warning,
            NoConnect => Error,
        },
        NoConnect => Error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commutative() {
        use parse::component::PinType::*;

        let all = vec![
            Input, Output, Passive, PowerIn, PowerOut, Tristate, NoConnect
        ];

        for first in &all {
            for second in &all {
                println!("{:?} vs {:?}", first, second);
                assert_eq!(
                    check_electric_connection(*first, *second),
                    check_electric_connection(*second, *first)
                );
            }
        }
    }
}
