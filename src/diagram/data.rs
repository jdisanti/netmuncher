//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use serde_json;

use diagram::compile::DiagramCompiler;
use error;
use parse;

#[derive(Serialize)]
pub struct Diagram {
    pub global_nets: Vec<String>,
    pub main: Node,
}

impl Diagram {
    pub fn compile(file_name: &str) -> error::Result<Diagram> {
        let result = parse::parse(file_name)?;
        DiagramCompiler::new(result).compile()
    }

    pub fn to_json_bytes(self) -> error::Result<Vec<u8>> {
        let result = serde_json::to_vec_pretty(&self)?;
        Ok(result)
    }
}

#[derive(Default, Serialize)]
pub struct Node {
    pub name: String,
    pub value: Option<String>,

    pub input_pins: Vec<String>,
    pub output_pins: Vec<String>,
    pub other_pins: Vec<String>,

    pub child_nodes: Vec<Node>,
    pub connections: Vec<Connection>,
}

impl Node {
    pub fn new(name: String) -> Node {
        Node {
            name: name,
            ..Default::default()
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "typ")]
pub enum Point {
    Global { net: String },
    Node { node: String, pin: String },
}

#[derive(Serialize)]
pub struct Connection {
    pub name: String,
    pub from: Point,
    pub to: Point,
}
