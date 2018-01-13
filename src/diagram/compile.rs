//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use std::collections::BTreeMap;

use diagram::data::*;
use error;
use parse::ParseResult;
use parse::component::{Component, Instance, PinType};
use ref_gen::ReferenceGenerator;

pub struct DiagramCompiler {
    input: Input,
    output: Output,
}

impl DiagramCompiler {
    pub fn new(parse_result: ParseResult) -> DiagramCompiler {
        DiagramCompiler {
            input: Input {
                components: parse_result.components,
                global_nets: parse_result.global_nets,
            },
            output: Output {
                ref_gen: ReferenceGenerator::new("_"),
            },
        }
    }

    pub fn compile(mut self) -> error::Result<Diagram> {
        let main = {
            let main_component = self.input.find_component("Main").unwrap();
            let main_instance = Instance::new(main_component.tag, "Main".into());
            self.output
                .instantiate(&self.input, &main_component, &main_instance)
        };

        Ok(Diagram {
            global_nets: self.input.global_nets,
            main: main,
        })
    }
}

struct Output {
    ref_gen: ReferenceGenerator,
}

impl Output {
    fn instantiate(&mut self, input: &Input, component: &Component, instance: &Instance) -> Node {
        let instance_name = self.ref_gen.next(&instance.name);

        let mut node = Node::new(instance_name);
        node.value = instance.value.clone();

        for pin in &component.units[0].pins {
            use parse::component::PinType::*;
            let pin_name = pin.name.clone();
            match pin.typ {
                Input | PowerIn => node.input_pins.push(pin_name),
                Output | PowerOut => node.output_pins.push(pin_name),
                _ => node.other_pins.push(pin_name),
            }
        }

        let mut net_pins: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
        for instance in &component.instances {
            let child_component = input.find_component(&instance.name).unwrap();
            let child_node = self.instantiate(input, &child_component, &instance);
            for pin in &child_component.units[0].pins {
                if input.global_nets.contains(&pin.name) {
                    node.connections.push(Connection {
                        name: pin.name.clone(),
                        from: Point::Global {
                            net: pin.name.clone(),
                        },
                        to: Point::Node {
                            node: child_node.name.clone(),
                            pin: pin.name.clone(),
                        },
                    });
                } else if let Some(&(_, ref connection)) = instance
                    .connections
                    .iter()
                    .find(|&&(ref p, _)| *p == pin.name)
                {
                    if input.global_nets.contains(connection) {
                        node.connections.push(Connection {
                            name: pin.name.clone(),
                            from: Point::Global {
                                net: pin.name.clone(),
                            },
                            to: Point::Node {
                                node: child_node.name.clone(),
                                pin: pin.name.clone(),
                            },
                        });
                    } else if connection != "noconnect" {
                        if component.nets.exists(connection) {
                            if !net_pins.contains_key(connection) {
                                net_pins.insert(connection.clone(), Vec::new());
                            }
                            net_pins
                                .get_mut(connection)
                                .unwrap()
                                .push((child_node.name.clone(), pin.name.clone()));
                        } else {
                            node.connections.push(Connection {
                                name: connection.clone(),
                                from: Point::Node {
                                    node: node.name.clone(),
                                    pin: connection.clone(),
                                },
                                to: Point::Node {
                                    node: child_node.name.clone(),
                                    pin: pin.name.clone(),
                                },
                            });
                        }
                    }
                } else if pin.typ != PinType::NoConnect {
                    unreachable!("validation should catch this");
                }
            }

            node.child_nodes.push(child_node);
        }

        for (net_name, pins) in net_pins {
            for i in 0..(pins.len() - 1) {
                let (left, right) = (&pins[i], &pins[i + 1]);
                node.connections.push(Connection {
                    name: net_name.clone(),
                    from: Point::Node {
                        node: left.0.clone(),
                        pin: left.1.clone(),
                    },
                    to: Point::Node {
                        node: right.0.clone(),
                        pin: right.1.clone(),
                    },
                });
            }
        }

        node
    }
}

struct Input {
    components: Vec<Component>,
    global_nets: Vec<String>,
}

impl Input {
    fn find_component(&self, name: &str) -> Option<&Component> {
        self.components.iter().find(|c| c.name() == name)
    }
}
