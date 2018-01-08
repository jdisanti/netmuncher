//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

/// Represents the character offset in the program code where something is located
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SrcTag {
    pub source: usize,
    pub offset: usize,
}

impl SrcTag {
    pub fn new(source: usize, offset: usize) -> SrcTag {
        SrcTag {
            source: source,
            offset: offset,
        }
    }

    /// Returns the (row, column) of this tag in the given program text
    pub fn row_col(&self, program: &str) -> (usize, usize) {
        let mut row: usize = 1;
        let mut col: usize = 1;

        for i in 0..self.offset {
            if &program[i..i + 1] == "\n" {
                row += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        (row, col)
    }
}

pub trait SrcTagged {
    fn src_tag(&self) -> SrcTag;
}

#[derive(Debug)]
pub struct Source {
    pub id: usize,
    pub name: String,
    pub code: String,
}

impl Source {
    pub fn new(id: usize, name: String, code: String) -> Source {
        Source {
            id: id,
            name: name,
            code: code,
        }
    }
}

#[derive(Debug, Default)]
pub struct Sources {
    sources: Vec<Source>,
}

impl Sources {
    pub fn new() -> Sources {
        Default::default()
    }

    pub fn name(&self, source_id: usize) -> &String {
        &self.sources[source_id].name
    }

    pub fn code(&self, source_id: usize) -> &String {
        &self.sources[source_id].code
    }

    pub fn source(&self, source_id: usize) -> &Source {
        &self.sources[source_id]
    }

    pub fn push_source(&mut self, name: String, source: String) -> usize {
        let source_id = self.sources.len();
        self.sources.push(Source::new(source_id, name, source));
        source_id
    }

    pub fn locate(&self, tag: SrcTag) -> String {
        let source = self.source(tag.source);
        let (row, col) = tag.row_col(&source.code);
        format!("{}:{}:{}", source.name, row, col)
    }
}

pub struct Locator<'a> {
    sources: &'a Sources,
    current_source: usize,
}

impl<'a> Locator<'a> {
    pub fn new(sources: &Sources, current_source: usize) -> Locator {
        Locator {
            sources: sources,
            current_source: current_source,
        }
    }

    pub fn name(&self) -> &String {
        self.sources.name(self.current_source)
    }

    pub fn locate(&self, offset: usize) -> String {
        self.sources
            .locate(SrcTag::new(self.current_source, offset))
    }

    pub fn tag(&self, offset: usize) -> SrcTag {
        SrcTag::new(self.current_source, offset)
    }
}
