//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use std::fmt;

use error;
use parse::source::{Locator, Sources, SrcTag};
use serde::{Serialize, Serializer};

macro_rules! err {
    ($msg:expr) => {
        return Err(error::ErrorKind::ComponentError($msg.into()).into());
    };
    ($msg:expr $(, $prm:expr)*) => {
        return Err(error::ErrorKind::ComponentError(format!($msg, $($prm,)*)).into());
    };
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
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

impl Serialize for PinNum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.0)
    }
}

impl fmt::Display for PinNum {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct UnitPin {
    pub name: String,
    pub typ: PinType,
    pub nums: Vec<PinNum>,
}

impl UnitPin {
    pub fn new(name: String, typ: PinType, nums: Vec<PinNum>) -> UnitPin {
        UnitPin {
            name: name,
            typ: typ,
            nums: nums,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Pin {
    pub name: String,
    pub typ: PinType,
    pub num: PinNum,
}

impl Pin {
    pub fn new(name: String, typ: PinType, num: PinNum) -> Pin {
        Pin {
            name: name,
            typ: typ,
            num: num,
        }
    }
}

#[derive(Debug)]
pub struct Instance {
    pub tag: SrcTag,
    pub name: String,
    pub value: Option<String>,
    pub connections: Vec<(String, String)>,
}

impl Instance {
    pub fn new(tag: SrcTag, name: String) -> Instance {
        Instance {
            tag: tag,
            name: name,
            value: None,
            connections: Vec::new(),
        }
    }

    pub fn find_connection(&self, pin_name: &str) -> Option<&String> {
        self.connections
            .iter()
            .find(|&&(ref name, _)| *name == pin_name)
            .map(|tup| &tup.1)
    }

    pub fn value(&self) -> Option<&str> {
        self.value.as_ref().map(|v| v as &str)
    }
}

#[derive(Clone, Default, Debug)]
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
            err!(
                "pin number {} assigned to multiple names: {}, {}",
                pin.num,
                pin.name,
                other.name
            );
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

impl IntoIterator for PinMap {
    type Item = Pin;
    type IntoIter = ::std::vec::IntoIter<Pin>;

    fn into_iter(self) -> Self::IntoIter {
        self.pins.into_iter()
    }
}

#[derive(Default, Debug)]
pub struct NetList {
    nets: Vec<String>,
}

impl NetList {
    pub fn add_net(&mut self, net: String) -> error::Result<()> {
        if self.exists(&net) {
            err!("duplicate net named {}", net)
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

#[derive(Clone, Debug, Default)]
pub struct Unit {
    pub pins: PinMap,
}

impl Unit {
    pub fn new() -> Unit {
        Default::default()
    }
}

#[derive(Debug)]
pub struct Component {
    pub tag: SrcTag,
    name: String,
    is_abstract: bool,
    footprint: Option<String>,
    prefix: Option<String>,
    default_value: String,
    pub nets: NetList,
    pub instances: Vec<Instance>,
    pub units: Vec<Unit>,
}

impl Component {
    pub fn new(tag: SrcTag, name: String, is_abstract: bool) -> Component {
        Component {
            tag: tag,
            name: name.clone(),
            is_abstract: is_abstract,
            footprint: None,
            prefix: None,
            default_value: name,
            nets: Default::default(),
            instances: Vec::new(),
            units: vec![Unit::new()],
        }
    }

    pub fn add_pin(&mut self, pin: Pin) -> error::Result<()> {
        for unit in &mut self.units {
            unit.pins.add_pin(pin.clone())?;
        }

        Ok(())
    }

    pub fn take_pins(&mut self) -> PinMap {
        let mut result: PinMap = Default::default();
        ::std::mem::swap(&mut result, &mut self.units[0].pins);
        result
    }

    pub fn abstract_pins(&self) -> &PinMap {
        assert!(self.is_abstract);
        &self.units[0].pins
    }

    pub fn pins(&self) -> &PinMap {
        assert!(!self.is_abstract && !self.has_units());
        &self.units[0].pins
    }

    pub fn has_units(&self) -> bool {
        self.units.len() > 1
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn default_value(&self) -> &str {
        &self.default_value
    }

    pub fn set_default_value(&mut self, value: String) {
        self.default_value = value;
    }

    pub fn is_abstract(&self) -> bool {
        self.is_abstract
    }

    pub fn footprint(&self) -> &str {
        assert!(!self.is_abstract);
        self.footprint.as_ref().unwrap()
    }

    pub fn set_footprint(&mut self, footprint: String) {
        self.footprint = Some(footprint);
    }

    pub fn prefix(&self) -> &str {
        assert!(!self.is_abstract);
        self.prefix.as_ref().unwrap()
    }

    pub fn set_prefix(&mut self, prefix: String) {
        self.prefix = Some(prefix);
    }

    pub fn validate_parameters(&self, units: &Sources) -> error::Result<()> {
        // short names to avoid line wrapping on errors
        let n = &self.name;
        let l = || units.locate(self.tag);

        if !self.is_abstract {
            if !self.footprint.is_some() || self.footprint.as_ref().unwrap().is_empty() {
                err!("{}: concrete component {} must specify a footprint", l(), n);
            }
            if !self.prefix.is_some() || self.prefix.as_ref().unwrap().is_empty() {
                err!("{}: concrete component {} must specify a prefix", l(), n);
            }
            if !self.instances.is_empty() {
                err!("{}: concrete component {} cannot have instances", l(), n);
            }
            if !self.nets.is_empty() {
                err!("{}: concrete component {} cannot have nets", l(), n);
            }
        } else {
            if self.footprint.is_some() {
                err!("{}: abstract component {} cannot have a footprint", l(), n);
            }
            if self.prefix.is_some() {
                err!("{}: abstract component {} cannot have a prefix", l(), n);
            }
        }
        Ok(())
    }

    pub fn validate_units(&self, sources: &Sources) -> error::Result<()> {
        let mut pin_nums = Vec::new();
        for unit in &self.units {
            pin_nums.extend((&unit.pins).into_iter().map(|p| p.num.0));
        }
        pin_nums.sort();

        for i in 0..pin_nums.len() {
            let pin_num = (i + 1) as u32;
            if pin_nums[i] != pin_num {
                err!(
                    "{}: component {} is missing some pins (take a look at pin {})",
                    sources.locate(self.tag),
                    self.name,
                    pin_num
                );
            }
        }
        Ok(())
    }

    pub fn add_unit_pins(
        &mut self,
        locator: &Locator,
        offset: usize,
        mut unit_pins: Vec<UnitPin>,
    ) -> error::Result<()> {
        if self.has_units() {
            err!(
                "{}: cannot have multiple unit specifications in component {}",
                locator.locate(offset),
                self.name
            );
        }

        if unit_pins.is_empty() {
            err!(
                "{}: unit definition in {} must have at least one pin",
                locator.locate(offset),
                self.name
            );
        }

        let mut pin_lens: Vec<usize> = unit_pins.iter().map(|pin| pin.nums.len()).collect();
        pin_lens.sort();
        pin_lens.dedup();
        if 1 != pin_lens.len() {
            err!(
                "{}: unit definition in {} doesn't have an equal number of pin numbers for each pin",
                locator.locate(offset),
                self.name
            );
        }

        let unit_count = pin_lens[0];
        let mut units = Vec::new();
        let mut current_pins = Some(self.take_pins());
        for _ in 0..unit_count {
            let mut unit = Unit::new();
            // Add the non-unit pins to the first unit
            if let Some(pins) = current_pins.take() {
                for pin in pins.into_iter() {
                    unit.pins.add_pin(pin)?;
                }
            }
            for unit_pin in &mut unit_pins {
                unit.pins.add_pin(Pin::new(
                    unit_pin.name.clone(),
                    unit_pin.typ,
                    unit_pin.nums.remove(0),
                ))?;
            }
            units.push(unit);
        }
        self.units = units;

        Ok(())
    }
}
