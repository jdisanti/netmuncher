//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use parse::src_tag::SrcTag;

#[derive(Debug)]
pub struct SrcUnit {
    pub id: usize,
    pub name: String,
    pub source: String,
}

impl SrcUnit {
    pub fn new(id: usize, name: String, source: String) -> SrcUnit {
        SrcUnit {
            id: id,
            name: name,
            source: source,
        }
    }
}

#[derive(Debug, Default)]
pub struct SrcUnits {
    units: Vec<SrcUnit>,
}

impl SrcUnits {
    pub fn new() -> SrcUnits {
        Default::default()
    }

    pub fn name(&self, unit_id: usize) -> &String {
        &self.units[unit_id].name
    }

    pub fn source(&self, unit_id: usize) -> &String {
        &self.units[unit_id].source
    }

    pub fn unit(&self, unit_id: usize) -> &SrcUnit {
        &self.units[unit_id]
    }

    pub fn push_unit(&mut self, name: String, source: String) -> usize {
        let unit_id = self.units.len();
        self.units.push(SrcUnit::new(unit_id, name, source));
        unit_id
    }

    pub fn locate(&self, tag: SrcTag) -> String {
        let unit = self.unit(tag.unit);
        let (row, col) = tag.row_col(&unit.source);
        format!("{}:{}:{}", unit.name, row, col)
    }
}

pub struct Locator<'a> {
    src_units: &'a SrcUnits,
    current_unit: usize,
}

impl<'a> Locator<'a> {
    pub fn new(src_units: &SrcUnits, current_unit: usize) -> Locator {
        Locator {
            src_units: src_units,
            current_unit: current_unit,
        }
    }

    pub fn name(&self) -> &String {
        self.src_units.name(self.current_unit)
    }

    pub fn locate(&self, offset: usize) -> String {
        self.src_units
            .locate(SrcTag::new(self.current_unit, offset))
    }

    pub fn tag(&self, offset: usize) -> SrcTag {
        SrcTag::new(self.current_unit, offset)
    }
}
