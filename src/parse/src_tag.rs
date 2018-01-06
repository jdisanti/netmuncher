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
    pub unit: usize,
    pub offset: usize,
}

impl SrcTag {
    pub fn new(unit: usize, offset: usize) -> SrcTag {
        SrcTag {
            unit: unit,
            offset: offset,
        }
    }

    pub fn invalid() -> SrcTag {
        SrcTag {
            unit: usize::max_value(),
            offset: usize::max_value(),
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
