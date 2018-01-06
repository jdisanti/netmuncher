//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use std::fmt;

use parse::src_tag::SrcTag;
use parse::src_unit::SrcUnits;
use error;

macro_rules! err {
    ($msg:expr) => {
        return Err(error::ErrorKind::ComponentError($msg.into()).into());
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PinType {
    Input,
    Output,
    Passive,
    PowerIn,
    PowerOut,
    Tristate,
    NoConnect,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PinNum(pub u32);

impl fmt::Display for PinNum {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct Pin {
    pub name: String,
    pub typ: PinType,
    pub num: PinNum,
}

#[derive(Debug)]
pub struct Instance {
    pub tag: SrcTag,
    pub name: String,
    pub connections: Vec<(String, String)>,
}

impl Instance {
    pub fn new(tag: SrcTag, name: String) -> Instance {
        Instance {
            tag: tag,
            name: name,
            connections: Vec::new(),
        }
    }

    pub fn find_connection(&self, pin_name: &str) -> Option<&String> {
        self.connections
            .iter()
            .find(|&&(ref name, _)| *name == pin_name)
            .map(|tup| &tup.1)
    }
}

#[derive(Default, Debug)]
pub struct PinMap {
    pins: Vec<Pin>,
}

impl PinMap {
    pub fn add_pin(&mut self, pin: Pin) -> error::Result<()> {
        if pin.num == PinNum(0) {
            err!("pin numbers must start at 1");
        }
        if self.find_by_name(&pin.name).is_some() {
            err!(format!("duplicate pin named {}", pin.name));
        }
        if let Some(other) = self.find_by_num(pin.num) {
            err!(format!(
                "pin number {} assigned to multiple names: {}, {}",
                pin.num, pin.name, other.name
            ));
        }
        self.pins.push(pin);
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.pins.is_empty()
    }

    pub fn len(&self) -> usize {
        self.pins.len()
    }

    pub fn find_by_num(&self, num: PinNum) -> Option<&Pin> {
        self.pins.iter().find(|p: &&Pin| p.num == num)
    }

    pub fn find_by_name(&self, name: &str) -> Option<&Pin> {
        self.pins.iter().find(|p: &&Pin| p.name == name)
    }
}

impl<'a> IntoIterator for &'a PinMap {
    type Item = &'a Pin;
    type IntoIter = ::std::slice::Iter<'a, Pin>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.pins).into_iter()
    }
}

#[derive(Default, Debug)]
pub struct NetList {
    nets: Vec<String>,
}

impl NetList {
    pub fn add_net(&mut self, net: String) -> error::Result<()> {
        if self.exists(&net) {
            err!(format!("duplicate net named {}", net))
        } else {
            self.nets.push(net);
            Ok(())
        }
    }

    pub fn extend<I>(&mut self, iterator: I) -> error::Result<()>
    where
        I: Iterator<Item = String>,
    {
        for net in iterator {
            self.add_net(net)?;
        }
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.nets.is_empty()
    }

    pub fn len(&self) -> usize {
        self.nets.len()
    }

    pub fn exists(&self, net: &str) -> bool {
        self.nets.iter().find(|n: &&String| *n == net).is_some()
    }

    pub fn iter<'a>(&'a self) -> ::std::slice::Iter<'a, String> {
        self.into_iter()
    }
}

impl<'a> IntoIterator for &'a NetList {
    type Item = &'a String;
    type IntoIter = ::std::slice::Iter<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.nets).into_iter()
    }
}

#[derive(Debug)]
pub struct Component {
    pub tag: SrcTag,
    pub name: String,
    pub footprint: Option<String>,
    pub prefix: Option<String>,
    pub pins: PinMap,
    pub nets: NetList,
    pub instances: Vec<Instance>,
}

impl Component {
    pub fn new(tag: SrcTag, name: String) -> Component {
        Component {
            tag: tag,
            name: name,
            footprint: None,
            prefix: None,
            pins: Default::default(),
            nets: Default::default(),
            instances: Vec::new(),
        }
    }

    pub fn validate_parameters(&self, units: &SrcUnits) -> error::Result<()> {
        if self.footprint.is_some() || self.prefix.is_some() {
            if !self.footprint.is_some() || self.footprint.as_ref().unwrap().is_empty() {
                err!(format!(
                    "{}: component {} must specify a footprint",
                    units.locate(self.tag),
                    self.name
                ));
            }
            if !self.prefix.is_some() || self.prefix.as_ref().unwrap().is_empty() {
                err!(format!(
                    "{}: component {} must specify a reference prefix",
                    units.locate(self.tag),
                    self.name
                ));
            }
            if !self.instances.is_empty() {
                err!(format!(
                    "{}: component {} cannot have instances if it has a footprint and prefix",
                    units.locate(self.tag),
                    self.name
                ));
            }
        }
        Ok(())
    }

    pub fn validate_pins(&self, units: &SrcUnits) -> error::Result<()> {
        if self.pins.is_empty() {
            return Ok(());
        }

        for i in 0..self.pins.len() {
            let pin_num = PinNum((i + 1) as u32);
            if !self.pins.find_by_num(pin_num).is_some() {
                err!(format!(
                    "{}: component {} is missing some pins (take a look at pin {})",
                    units.locate(self.tag),
                    self.name,
                    pin_num
                ));
            }
        }

        Ok(())
    }
}
