//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use std::collections::HashSet;
use std::fmt::Write;

use circuit::{Circuit, ComponentGroup, SerializeCircuit};
use error;

struct Groups {
    groups: Vec<Group>,
}

impl Groups {
    fn new() -> Groups {
        Groups { groups: Vec::new() }
    }

    fn generate_groups(circuit: &Circuit) -> Groups {
        let mut groups = Groups::new();
        Groups::generate_groups_internal(&mut groups, &circuit.root_group.sub_groups, "");
        groups
    }

    fn generate_groups_internal(
        groups: &mut Groups,
        component_groups: &[ComponentGroup],
        path_prefix: &str,
    ) {
        for component_group in component_groups {
            let path = format!("{}/{}", path_prefix, component_group.name);
            if !component_group.sub_groups.is_empty() {
                Groups::generate_groups_internal(groups, &component_group.sub_groups, &path);
            }
            groups.groups.push(Group::new(
                path,
                component_group.components.iter().cloned().collect(),
            ));
        }
    }

    fn find_by_ref(&self, reference: &String) -> &Group {
        for group in &self.groups {
            if group.references.contains(reference) {
                return group;
            }
        }
        unreachable!()
    }
}

struct Group {
    path: String,
    references: HashSet<String>,
}

impl Group {
    fn new(path: String, references: HashSet<String>) -> Group {
        Group {
            path: path,
            references: references,
        }
    }
}

pub struct KicadNetListSerializer {}

impl KicadNetListSerializer {
    pub fn new() -> KicadNetListSerializer {
        KicadNetListSerializer {}
    }
}

impl SerializeCircuit for KicadNetListSerializer {
    fn serialize(self, circuit: &Circuit) -> error::Result<Vec<u8>> {
        let groups = Groups::generate_groups(circuit);

        let mut f = String::new();
        writeln!(f, "(export (version D)")?;
        writeln!(f, "  (design")?;
        writeln!(f, "    (source \"netmuncher_generated\")")?;
        writeln!(f, "    (tool \"netmuncher (0.1)\")")?;

        for (i, group) in groups.groups.iter().enumerate() {
            writeln!(
                f,
                "    (sheet (number {}) (name {}/) (tstamps {}/))",
                i + 1,
                group.path,
                group.path,
            )?;
        }

        writeln!(f, "  )")?;
        writeln!(f, "  (components")?;
        for instance in &circuit.instances {
            let group = groups.find_by_ref(&instance.reference);
            writeln!(f, "    (comp (ref {})", instance.reference)?;
            writeln!(f, "      (value {})", instance.value)?;
            writeln!(f, "      (footprint {})", instance.footprint)?;
            writeln!(
                f,
                "      (sheetpath (names {}/) (tstamps {}/))",
                group.path, group.path,
            )?;
            writeln!(f, "      (tstamp {})", instance.reference)?;
            writeln!(f, "    )")?;
        }
        writeln!(f, "  )")?;
        writeln!(f, "  (nets")?;
        for (index, net) in circuit.nets.iter().enumerate() {
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
        Ok(f.into_bytes())
    }
}
