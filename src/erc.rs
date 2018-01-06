//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use circuit::Node;
use error::{self, ErrorKind};
use parse::component::{Instance, PinType};
use parse::src_unit::SrcUnits;

#[derive(Debug, Eq, PartialEq)]
enum ValidationResult {
    Valid,
    Warning,
    Error,
}

pub fn check_connection(units: &SrcUnits, instance: &Instance, current: &Node, others: &[Node]) -> error::Result<()> {
    for other in others {
        match check(current.pin_type, other.pin_type) {
            ValidationResult::Valid => {}
            ValidationResult::Warning => {
                println!(
                    "WARN: {}: in instantiation of {}, pin {} ({:?}) connected to {} ({:?})",
                    units.locate(instance.tag),
                    instance.name,
                    current.pin_name,
                    current.pin_type,
                    other.pin_name,
                    other.pin_type,
                );
            }
            ValidationResult::Error => {
                bail!(ErrorKind::ERCError(format!(
                    "{}: in instantiation of {}, pin {} ({:?}) connected to {} ({:?})",
                    units.locate(instance.tag),
                    instance.name,
                    current.pin_name,
                    current.pin_type,
                    other.pin_name,
                    other.pin_type,
                )));
            }
        }
    }
    Ok(())
}

fn check(first: PinType, second: PinType) -> ValidationResult {
    use parse::component::PinType::*;
    use self::ValidationResult::*;

    match first {
        Input => match second {
            Input | Output | Passive | PowerIn | PowerOut | Tristate => Valid,
            NoConnect => Error,
        },
        Output => match second {
            Input | Passive | PowerIn => Valid,
            Tristate => Warning,
            NoConnect | PowerOut | Output => Error,
        },
        Passive => match second {
            Input | Output | Passive | PowerIn | PowerOut | Tristate => Valid,
            NoConnect => Error,
        },
        PowerIn => match second {
            Input | Output | Passive | PowerIn | PowerOut => Valid,
            Tristate => Warning,
            NoConnect => Error,
        },
        PowerOut => match second {
            Input | Passive | PowerIn => Valid,
            Tristate => Warning,
            NoConnect | Output | PowerOut => Error,
        },
        Tristate => match second {
            Input | Tristate | Passive => Valid,
            Output | PowerIn | PowerOut => Warning,
            NoConnect => Error,
        },
        NoConnect => Error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commutative() {
        use parse::component::PinType::*;

        let all = vec![
            Input, Output, Passive, PowerIn, PowerOut, Tristate, NoConnect
        ];

        for first in &all {
            for second in &all {
                assert_eq!(check(*first, *second), check(*second, *first));
            }
        }
    }
}
