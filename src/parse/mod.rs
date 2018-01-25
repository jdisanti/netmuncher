//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use lalrpop_util::ParseError;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use error;

#[cfg_attr(rustfmt, rustfmt_skip)]
mod grammar;

pub mod ast;
pub mod component;
pub mod source;
pub mod token;
mod validator;

use self::ast::{Ast, Tagged};
use self::component::{Component, Instance, Pin, PinNum};
use self::source::{Locator, Sources};
use self::validator::Validator;

pub struct ParseResult {
    pub sources: Sources,
    pub components: Vec<Component>,
    pub global_nets: Vec<String>,
}

pub fn parse(file_name: &str) -> error::Result<ParseResult> {
    let main_file = Path::new(file_name).file_name().unwrap();
    let main_path = Path::new(file_name).parent().unwrap();

    let mut sources = Sources::new();

    let mut modules_to_require: Vec<PathBuf> = Vec::new();
    let mut modules_required: Vec<PathBuf> = Vec::new();
    modules_to_require.push(module_path(&main_path, &main_file).unwrap());

    let mut global_nets: Vec<String> = Vec::new();
    let mut components: Vec<Component> = Vec::new();
    while let Some(path) = modules_to_require.pop() {
        if !modules_required.contains(&path) {
            modules_required.push(path.clone());
            let source_id = sources.push_source(path.to_str().unwrap().into(), load_file(&path)?);
            let locator = Locator::new(&sources, source_id);
            let parse_result = parse_file(&locator, sources.code(source_id))?;

            let path_parent = path.parent().unwrap();
            for require in parse_result.requires {
                if let Some(module_path) = module_path(&path_parent, &require.module) {
                    modules_to_require.push(module_path);
                } else {
                    err!(
                        "{}: cannot find file named \"{}\"",
                        sources.locate(require.tag),
                        require.module
                    );
                }
            }
            global_nets.extend(parse_result.global_nets.into_iter());
            components.extend(parse_result.components.into_iter());
        }
    }

    Validator::new(&sources, &global_nets, &components).validate()?;

    Ok(ParseResult {
        sources: sources,
        components: components,
        global_nets: global_nets,
    })
}

#[derive(Default)]
pub struct ParseFileResult {
    pub requires: Vec<ast::Require>,
    pub components: Vec<Component>,
    pub global_nets: Vec<String>,
}

impl ParseFileResult {
    pub fn new() -> ParseFileResult {
        Default::default()
    }

    fn consider_tree(&mut self, locator: &Locator, tree: Ast) -> error::Result<()> {
        match tree {
            Ast::Require(require) => {
                self.requires.push(require);
            }
            Ast::Nets(global_nets) => {
                self.global_nets.extend(global_nets.nets.into_iter());
            }
            Ast::ComponentDef(component_def) => {
                let offset = component_def.tag.offset;
                let name = component_def.name.clone();
                self.consider_component(locator, component_def)
                    .map_err(|err| {
                        err.chain_err(|| {
                            error::ErrorKind::NetmuncherError(format!(
                                "{}: error in component {}",
                                locator.locate(offset),
                                name
                            ))
                        })
                    })?;
            }
            _ => unreachable!("grammar should not allow this to be reached"),
        }
        Ok(())
    }

    fn consider_component(
        &mut self,
        locator: &Locator,
        def: ast::ComponentDef,
    ) -> error::Result<()> {
        let mut component = Component::new(def.tag, def.name, def.is_abstract);
        for param in def.params {
            let tag = param.tag();
            self.consider_component_param(&mut component, param)
                .map_err(|err| {
                    error::ErrorKind::NetmuncherError(format!(
                        "{}: {}",
                        locator.locate(tag.offset),
                        err
                    ))
                })?;
        }
        self.components.push(component);
        Ok(())
    }

    fn consider_component_param(
        &mut self,
        component: &mut Component,
        param: Ast,
    ) -> error::Result<()> {
        match param {
            Ast::AbstractPins(abstract_pins) => {
                if !component.is_abstract() {
                    err!("concrete components must state pin numbers for pins");
                }
                for pin in abstract_pins {
                    let num = (component.abstract_pins().len() + 1) as u32;
                    component.add_pin(Pin::new(pin.name, pin.typ, PinNum(num)))?;
                }
            }
            Ast::ConcretePins(concrete_pins) => {
                if component.is_abstract() {
                    err!("abstract components shouldn't state pin numbers for pins");
                }
                for pin in concrete_pins {
                    component.add_pin(Pin::new(pin.name, pin.typ, pin.num))?;
                }
            }
            Ast::Connect(connect) => {
                if !component.is_abstract() {
                    err!("concrete components cannot have internal connects");
                }
                if connect.left.len() != connect.right.len() {
                    err!("must connect the same number of pins/nets on the left and right");
                }
                let zipped = connect.left.into_iter().zip(connect.right.into_iter());
                component.connects.extend(zipped);
            }
            Ast::Footprint(footprint) => {
                if component.is_abstract() {
                    err!("abstract components shouldn't have footprints");
                }
                component.set_footprint(footprint.footprint)?;
            }
            Ast::InstanceDef(instance_def) => {
                self.consider_instance(component, instance_def)?;
            }
            Ast::Nets(nets) => {
                if !component.is_abstract() {
                    err!("concrete components shouldn't have nets");
                }
                for net in nets.nets {
                    component.nets.add_net(net)?;
                }
            }
            Ast::Prefix(prefix) => {
                if component.is_abstract() {
                    err!("abstract components shouldn't have prefixes");
                }
                component.set_prefix(prefix.prefix)?;
            }
            Ast::Value(value) => {
                component.set_default_value(value.value);
            }
            Ast::Unit(unit) => {
                component.add_unit_pins(unit.pins)?;
            }
            _ => unreachable!("grammar should not allow this to be reached"),
        }
        Ok(())
    }

    fn consider_instance(
        &mut self,
        component: &mut Component,
        def: ast::InstanceDef,
    ) -> error::Result<()> {
        if !component.is_abstract() {
            err!("concrete components cannot have instances");
        }
        let mut instance = Instance::new(def.tag, def.name);
        for param in def.parameters {
            match param {
                Ast::Value(value) => {
                    if instance.value.is_some() {
                        err!("multiple values specified for instance");
                    }
                    instance.value = Some(value.value);
                }
                Ast::ConnectionMap(conn_map) => {
                    instance
                        .connections
                        .extend(conn_map.connections.into_iter());
                }
                _ => unreachable!("grammar should not allow this to be reached"),
            }
        }
        component.instances.push(instance);
        Ok(())
    }
}

fn parse_file(locator: &Locator, source: &str) -> error::Result<ParseFileResult> {
    let tokens = token::tokenize(locator, source)?;
    let trees = grammar::parse_Source(&locator, tokens.into_iter()).map_err(|e| match e {
        ParseError::InvalidToken { location } => error::ErrorKind::NetmuncherError(format!(
            "{}: invalid token",
            locator.locate(location)
        )).into(),
        ParseError::UnrecognizedToken { token, expected } => match token {
            Some((location, token, _)) => error::ErrorKind::NetmuncherError(format!(
                "{}: unexpected token \"{}\". Expected one of: {}",
                locator.locate(location),
                token,
                expected.join(", ")
            )).into(),
            None => error::ErrorKind::NetmuncherError(
                format!("{}: unexpected end of file", locator.name()).into(),
            ).into(),
        },
        ParseError::ExtraToken { token } => error::ErrorKind::NetmuncherError(format!(
            "{}: extra token {}",
            locator.locate(token.0),
            token.1
        )).into(),
        ParseError::User { error } => error,
    })?;

    let mut result = ParseFileResult::new();
    for tree in trees {
        result.consider_tree(locator, tree)?;
    }
    Ok(result)
}

fn module_path<P: AsRef<Path>>(main_path: &Path, module_name: P) -> Option<PathBuf> {
    let path = main_path.join(module_name);
    if path.is_file() {
        Some(path)
    } else {
        None
    }
}

fn load_file<P: AsRef<Path>>(file_name: P) -> error::Result<String> {
    let mut file = File::open(file_name.as_ref())?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;
    Ok(file_contents)
}
