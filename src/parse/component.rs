//
// Copyright 2017 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use std::fmt;

use error;

macro_rules! err {
    ($msg:expr) => {
        return Err(error::ErrorKind::ComponentError($msg.into()).into());
    }
}

#[derive(Copy, Clone, Debug)]
pub enum PinType {
    Input,
    Output,
    Passive,
    Power,
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
    pub name: String,
    pub connections: Vec<(String, String)>,
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

#[derive(Debug)]
pub struct Component {
    pub name: String,
    pub footprint: Option<String>,
    pub prefix: Option<String>,
    pub pins: PinMap,
    pub nets: Vec<String>,
    pub instances: Vec<Instance>,
}

impl Component {
    pub fn new(name: String) -> Component {
        Component {
            name: name,
            footprint: None,
            prefix: None,
            pins: Default::default(),
            nets: Vec::new(),
            instances: Vec::new(),
        }
    }

    pub fn validate_parameters(&self) -> error::Result<()> {
        if self.footprint.is_some() || self.prefix.is_some() {
            if !self.footprint.is_some() || self.footprint.as_ref().unwrap().is_empty() {
                err!(format!("component {} must specify a footprint", self.name));
            }
            if !self.prefix.is_some() || self.prefix.as_ref().unwrap().is_empty() {
                err!(format!(
                    "component {} must specify a reference prefix",
                    self.name
                ));
            }
        }
        if !self.instances.is_empty() && !self.pins.is_empty() {
            err!(format!(
                "component {} can have pins or instances of other components, \
                 but not both",
                self.name
            ));
        }
        Ok(())
    }

    pub fn validate_pins(&self) -> error::Result<()> {
        if self.pins.is_empty() {
            return Ok(());
        }

        for i in 0..self.pins.len() {
            let pin_num = PinNum((i + 1) as u32);
            if !self.pins.find_by_num(pin_num).is_some() {
                err!(format!(
                    "component {} is missing some pins (take a look at pin {})",
                    self.name, pin_num
                ));
            }
        }

        Ok(())
    }
}
