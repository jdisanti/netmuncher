//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use std::fmt::Write;

use circuit::{Circuit, ComponentGroup, SerializeCircuit};
use error;

const INDENT_SIZE: usize = 2;

pub struct DotSerializer {
    indent: usize,
}

impl DotSerializer {
    pub fn new() -> DotSerializer {
        DotSerializer {
            indent: INDENT_SIZE,
        }
    }

    fn component(
        &mut self,
        f: &mut Write,
        circuit: &Circuit,
        reference: &str,
    ) -> error::Result<()> {
        let instance = circuit
            .instances
            .iter()
            .find(|&instance| instance.reference == reference)
            .unwrap();

        let mut pins = Vec::new();
        for net in &circuit.nets {
            for node in &net.nodes {
                if node.reference == reference {
                    pins.push(&node.pin_name);
                }
            }
        }

        let pins: Vec<String> = pins.into_iter().map(|p| format!("<{0}>{0}", p)).collect();
        if pins.len() > 1 {
            let pivot = pins.len() / 2;
            let left = &pins[0..pivot];
            let right = &pins[pivot..pins.len()];
            let left_pin_str: String = left.join("|");
            let right_pin_str: String = right.join("|");
            writeln!(
                f,
                "{0:1$}{2}[label=\"{{ {{{4}}}|{2}\\n{3}|{{{5}}} }}\"];",
                "", self.indent, reference, instance.value, left_pin_str, right_pin_str
            )?;
        } else {
            let pin_str: String = pins.join("|");
            writeln!(
                f,
                "{0:1$}{2}[label=\"{{ {2}\\n{3}|{{{4}}} }}\"];",
                "", self.indent, reference, instance.value, pin_str
            )?;
        }

        Ok(())
    }

    fn group(
        &mut self,
        f: &mut Write,
        circuit: &Circuit,
        group: &ComponentGroup,
    ) -> error::Result<()> {
        writeln!(
            f,
            "{0:1$}subgraph \"cluster_{2}\" {{",
            "", self.indent, group.name
        )?;
        self.indent += INDENT_SIZE;

        writeln!(f, "{0:1$}label = \"{2}\";", "", self.indent, group.name)?;
        writeln!(f, "{0:1$}style = \"dashed\";", "", self.indent)?;

        for sub_group in &group.sub_groups {
            self.group(f, circuit, sub_group)?;
        }

        for component in &group.components {
            self.component(f, circuit, component)?;
        }

        self.indent -= INDENT_SIZE;
        writeln!(f, "{0:1$}}}", "", self.indent)?;
        Ok(())
    }
}

impl SerializeCircuit for DotSerializer {
    fn serialize(mut self, circuit: &Circuit) -> error::Result<Vec<u8>> {
        let mut f = String::new();
        writeln!(f, "digraph G {{")?;
        writeln!(f, "{0:1$}graph[rankdir=LR];", "", INDENT_SIZE)?;
        writeln!(f, "{0:1$}node[shape=record];", "", INDENT_SIZE)?;

        let main_group = &circuit.root_group.sub_groups[0];
        for sub_group in &main_group.sub_groups {
            self.group(&mut f, circuit, sub_group)?;
        }
        for component in &main_group.components {
            self.component(&mut f, circuit, component)?;
        }
        for net in &circuit.nets {
            let node_list: Vec<String> = net.nodes
                .iter()
                .map(|n| format!("{}:{}", n.reference, n.pin_name))
                .collect();
            let node_str: String = node_list.join(" -> ");
            writeln!(
                f,
                "{0:1$}{2} [arrowhead=\"none\",label=\"{3}\"];",
                "", INDENT_SIZE, node_str, net.name
            )?;
        }

        writeln!(f, "}}")?;
        Ok(f.into_bytes())
    }
}
