//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use parse::component::{PinNum, PinType, UnitPin};
use parse::source::SrcTag;

pub trait Tagged {
    fn tag(&self) -> SrcTag;
}

#[derive(Debug, new)]
pub struct AbstractPin {
    pub tag: SrcTag,
    pub name: String,
    pub typ: PinType,
}

impl Tagged for AbstractPin {
    fn tag(&self) -> SrcTag {
        self.tag
    }
}

#[derive(Debug, new)]
pub struct ComponentDef {
    pub tag: SrcTag,
    pub name: String,
    pub is_abstract: bool,
    pub params: Vec<Ast>,
}

impl Tagged for ComponentDef {
    fn tag(&self) -> SrcTag {
        self.tag
    }
}

#[derive(Debug, new)]
pub struct ConcretePin {
    pub tag: SrcTag,
    pub name: String,
    pub typ: PinType,
    pub num: PinNum,
}

impl Tagged for ConcretePin {
    fn tag(&self) -> SrcTag {
        self.tag
    }
}

#[derive(Debug, new)]
pub struct Footprint {
    pub tag: SrcTag,
    pub footprint: String,
}

impl Tagged for Footprint {
    fn tag(&self) -> SrcTag {
        self.tag
    }
}

#[derive(Debug, new)]
pub struct InstanceDef {
    pub tag: SrcTag,
    pub name: String,
    pub parameters: Vec<Ast>,
}

impl Tagged for InstanceDef {
    fn tag(&self) -> SrcTag {
        self.tag
    }
}

#[derive(Debug, new)]
pub struct ConnectionMap {
    pub tag: SrcTag,
    pub connections: Vec<(String, String)>,
}

impl Tagged for ConnectionMap {
    fn tag(&self) -> SrcTag {
        self.tag
    }
}

#[derive(Debug, new)]
pub struct Nets {
    pub tag: SrcTag,
    pub nets: Vec<String>,
}

impl Tagged for Nets {
    fn tag(&self) -> SrcTag {
        self.tag
    }
}

#[derive(Debug, new)]
pub struct Prefix {
    pub tag: SrcTag,
    pub prefix: String,
}

impl Tagged for Prefix {
    fn tag(&self) -> SrcTag {
        self.tag
    }
}

#[derive(Debug, new)]
pub struct Require {
    pub tag: SrcTag,
    pub module: String,
}

impl Tagged for Require {
    fn tag(&self) -> SrcTag {
        self.tag
    }
}

#[derive(Debug, new)]
pub struct Value {
    pub tag: SrcTag,
    pub value: String,
}

impl Tagged for Value {
    fn tag(&self) -> SrcTag {
        self.tag
    }
}

#[derive(Debug, new)]
pub struct Unit {
    pub tag: SrcTag,
    pub pins: Vec<UnitPin>,
}

impl Tagged for Unit {
    fn tag(&self) -> SrcTag {
        self.tag
    }
}

#[derive(Debug)]
pub enum Ast {
    AbstractPins(Vec<AbstractPin>),
    ComponentDef(ComponentDef),
    ConcretePins(Vec<ConcretePin>),
    ConnectionMap(ConnectionMap),
    Footprint(Footprint),
    InstanceDef(InstanceDef),
    Nets(Nets),
    Prefix(Prefix),
    Require(Require),
    Value(Value),
    Unit(Unit),
}

impl Tagged for Ast {
    fn tag(&self) -> SrcTag {
        use self::Ast::*;
        match *self {
            AbstractPins(ref pins) => pins[0].tag(),
            ComponentDef(ref def) => def.tag(),
            ConcretePins(ref pins) => pins[0].tag(),
            ConnectionMap(ref map) => map.tag(),
            Footprint(ref footprint) => footprint.tag(),
            InstanceDef(ref def) => def.tag(),
            Nets(ref nets) => nets.tag(),
            Prefix(ref prefix) => prefix.tag(),
            Require(ref require) => require.tag(),
            Value(ref value) => value.tag(),
            Unit(ref unit) => unit.tag(),
        }
    }
}
