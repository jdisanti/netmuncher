//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

extern crate error_chain;
extern crate netmuncher;

use error_chain::ChainedError;
use netmuncher::circuit::Circuit;

fn test(file_name: &str) -> String {
    Circuit::compile(file_name)
        .err()
        .expect("expected error, but there was none")
        .display_chain()
        .to_string()
}

#[test]
fn empty_file() {
    assert_eq!(
        "Error: tests/errors/empty_file.nm: unexpected end of file\n",
        test("tests/errors/empty_file.nm")
    );
}

#[test]
fn unexpected_token() {
    assert_eq!(
        "Error: tests/errors/unexpected_token.nm:2:7: unexpected token \"=\". Expected one of: \
         \";\", \"{\"\n",
        test("tests/errors/unexpected_token.nm")
    );
}

#[test]
fn duplicate_pin() {
    assert_eq!(
        "Error: tests/errors/duplicate_pin.nm:1:1: error in component Main\nCaused by: \
         tests/errors/duplicate_pin.nm:3:5: duplicate pin named FOO\n",
        test("tests/errors/duplicate_pin.nm")
    );
}

#[test]
fn wrong_start_pin() {
    assert_eq!(
        "Error: tests/errors/wrong_start_pin.nm:2:1: error in component Foo\nCaused by: \
         tests/errors/wrong_start_pin.nm:3:5: pin numbers must start at 1\n",
        test("tests/errors/wrong_start_pin.nm")
    );
}

#[test]
fn duplicate_pin_number() {
    assert_eq!(
        "Error: tests/errors/duplicate_pin_number.nm:1:1: error in component Foo\nCaused by: \
         tests/errors/duplicate_pin_number.nm:3:5: pin number 1 assigned to multiple names: BAR, \
         FOO\n",
        test("tests/errors/duplicate_pin_number.nm")
    );
}

#[test]
fn duplicate_net() {
    assert_eq!(
        "Error: tests/errors/duplicate_net.nm:1:1: error in component Main\nCaused by: \
         tests/errors/duplicate_net.nm:3:5: duplicate net named A\n",
        test("tests/errors/duplicate_net.nm")
    );
}

#[test]
fn footprint_required() {
    assert_eq!(
        "Error: tests/errors/footprint_required.nm:2:1: concrete component Foo must specify a \
         footprint\n",
        test("tests/errors/footprint_required.nm")
    );
}

#[test]
fn duplicate_footprint() {
    assert_eq!(
        "Error: tests/errors/duplicate_footprint.nm:1:1: error in component Foo\nCaused by: \
         tests/errors/duplicate_footprint.nm:3:5: component already has a footprint set\n",
        test("tests/errors/duplicate_footprint.nm")
    );
}

#[test]
fn prefix_required() {
    assert_eq!(
        "Error: tests/errors/prefix_required.nm:1:1: concrete component Foo must specify a \
         prefix\n",
        test("tests/errors/prefix_required.nm")
    );
}

#[test]
fn duplicate_prefix() {
    assert_eq!(
        "Error: tests/errors/duplicate_prefix.nm:1:1: error in component Foo\nCaused by: \
         tests/errors/duplicate_prefix.nm:3:5: component already has a prefix set\n",
        test("tests/errors/duplicate_prefix.nm")
    );
}

#[test]
fn no_instances_on_dumb_component() {
    assert_eq!(
        "Error: tests/errors/no_instances_on_dumb_component.nm:8:1: error in component \
         Invalid\nCaused by: tests/errors/no_instances_on_dumb_component.nm:12:5: concrete \
         components cannot have instances\n",
        test("tests/errors/no_instances_on_dumb_component.nm")
    );
}

#[test]
fn no_nets_on_dumb_component() {
    assert_eq!(
        "Error: tests/errors/no_nets_on_dumb_component.nm:8:1: error in component Invalid\nCaused \
         by: tests/errors/no_nets_on_dumb_component.nm:12:5: concrete components shouldn\'t have \
         nets\n",
        test("tests/errors/no_nets_on_dumb_component.nm")
    );
}

#[test]
fn missing_pins() {
    assert_eq!(
        "Error: tests/errors/missing_pins.nm:1:1: component Foo is missing some pins (take a look \
         at pin 3)\n",
        test("tests/errors/missing_pins.nm")
    );
}

#[test]
fn duplicate_component() {
    assert_eq!(
        "Error: component Foo is defined more than once\n",
        test("tests/errors/duplicate_component.nm")
    );
}

#[test]
fn no_pins_in_main() {
    assert_eq!(
        "Error: tests/errors/no_pins_in_main.nm:1:1: component Main cannot have pins\n",
        test("tests/errors/no_pins_in_main.nm")
    );
}

#[test]
fn missing_main() {
    assert_eq!(
        "Error: missing component Main\n",
        test("tests/errors/missing_main.nm")
    );
}

#[test]
fn missing_connection() {
    assert_eq!(
        "Error: tests/errors/missing_connection.nm:25:5: no connection stated for pin B on \
         component C\n",
        test("tests/errors/missing_connection.nm")
    );
}

#[test]
fn cannot_find_connection() {
    assert_eq!(
        "Error: tests/errors/cannot_find_connection.nm:25:5: cannot find pin or net named asdf in \
         instantiation of component C\n",
        test("tests/errors/cannot_find_connection.nm")
    );
}

#[test]
fn unmapped_pin() {
    assert_eq!(
        "Error: tests/errors/unmapped_pin.nm:27:5: no connection stated for pin G on component B\n",
        test("tests/errors/unmapped_pin.nm")
    );
}

#[test]
fn missing_component() {
    assert_eq!(
        "Error: tests/errors/missing_component.nm:2:5: cannot find component definition for Foo\n",
        test("tests/errors/missing_component.nm")
    );
}

#[test]
fn missing_mapped_net() {
    assert_eq!(
        "Error: tests/errors/missing_mapped_net.nm:14:5: cannot find pin or net named C in \
         instantiation of component Foo\n",
        test("tests/errors/missing_mapped_net.nm")
    );
}

#[test]
fn erc_global_net_error() {
    assert_eq!(
        "Error: tests/errors/erc_global_net_error.nm:11:5: in instantiation of Foo, pin VCC \
         (PowerOut) is connected by net VCC to pin VCC (PowerOut) of instantiation Foo at \
         tests/errors/erc_global_net_error.nm:12:5\n",
        test("tests/errors/erc_global_net_error.nm")
    );
}

#[test]
fn erc_local_net_error() {
    assert_eq!(
        "Error: tests/errors/erc_local_net_error.nm:11:5: in instantiation of Foo, pin A \
         (PowerOut) is connected by net VCC to pin A (PowerOut) of instantiation Foo at \
         tests/errors/erc_local_net_error.nm:15:5\n",
        test("tests/errors/erc_local_net_error.nm")
    );
}

#[test]
fn erc_pin_to_pin_error() {
    assert_eq!(
        "Error: tests/errors/erc_pin_to_pin_error.nm:13:5: in instantiation of ConcreteThing, pin \
         ABSTRACT_IN (Input) mapped to OUT (Output)\n",
        test("tests/errors/erc_pin_to_pin_error.nm")
    );
}

#[test]
fn empty_circuit() {
    assert_eq!(
        "Error: tests/errors/empty_circuit.nm:5:1: empty circuit: no concrete components\n",
        test("tests/errors/empty_circuit.nm")
    );
}

#[test]
fn single_node_in_net() {
    assert_eq!(
        "Error: net named SOLO.Main1 needs to have more than one connection\n",
        test("tests/errors/single_node_in_net.nm")
    );
}

#[test]
fn invalid_quoted_symbol() {
    assert_eq!(
        "Error: tests/errors/invalid_quoted_symbol.nm:1:20: invalid character ' ' in symbol. \
         Symbols must be alphanumeric with underscores.\n",
        test("tests/errors/invalid_quoted_symbol.nm")
    );
}

#[test]
fn multiple_unit_specs() {
    assert_eq!(
        "Error: tests/errors/multiple_unit_specs.nm:1:1: error in component Foo\nCaused by: \
         tests/errors/multiple_unit_specs.nm:10:5: cannot have multiple unit specifications in \
         component Foo\n",
        test("tests/errors/multiple_unit_specs.nm")
    );
}

#[test]
fn unit_minimum_one_pin() {
    assert_eq!(
        "Error: tests/errors/unit_minimum_one_pin.nm:1:1: error in component Foo\nCaused by: \
         tests/errors/unit_minimum_one_pin.nm:8:5: unit definition in Foo must have at least one \
         pin\n",
        test("tests/errors/unit_minimum_one_pin.nm")
    );
}

#[test]
fn uneven_units() {
    assert_eq!(
        "Error: tests/errors/uneven_units.nm:1:1: error in component Foo\nCaused by: \
         tests/errors/uneven_units.nm:8:5: unit definition in Foo doesn\'t have an equal number \
         of pin numbers for each pin\n",
        test("tests/errors/uneven_units.nm")
    );
}

#[test]
fn concrete_component_requires_pin_num() {
    assert_eq!(
        "Error: tests/errors/concrete_component_require_pin_num.nm:1:1: error in component \
         Foo\nCaused by: tests/errors/concrete_component_require_pin_num.nm:5:5: concrete \
         components must state pin numbers for pins\n",
        test("tests/errors/concrete_component_require_pin_num.nm")
    );
}

#[test]
fn abstract_component_no_pin_num() {
    assert_eq!(
        "Error: tests/errors/abstract_component_no_pin_num.nm:1:1: error in component Foo\nCaused \
         by: tests/errors/abstract_component_no_pin_num.nm:2:5: abstract components shouldn\'t \
         state pin numbers for pins\n",
        test("tests/errors/abstract_component_no_pin_num.nm")
    );
}

#[test]
fn abstract_prefix() {
    assert_eq!(
        "Error: tests/errors/abstract_prefix.nm:1:1: error in component Main\nCaused by: \
         tests/errors/abstract_prefix.nm:2:5: abstract components shouldn\'t have prefixes\n",
        test("tests/errors/abstract_prefix.nm")
    );
}

#[test]
fn abstract_footprint() {
    assert_eq!(
        "Error: tests/errors/abstract_footprint.nm:1:1: error in component Main\nCaused by: \
         tests/errors/abstract_footprint.nm:2:5: abstract components shouldn\'t have footprints\n",
        test("tests/errors/abstract_footprint.nm")
    );
}

#[test]
fn duplicate_instance_value() {
    assert_eq!(
        "Error: tests/errors/duplicate_instance_value.nm:9:1: error in component Main\nCaused by: \
         tests/errors/duplicate_instance_value.nm:10:5: multiple values specified for instance\n",
        test("tests/errors/duplicate_instance_value.nm")
    );
}

#[test]
fn connect_nets() {
    assert_eq!(
        "Error: tests/errors/connect_nets.nm:1:1: could not find pin named \'C\' to connect to \
         \'A\'\n",
        test("tests/errors/connect_nets.nm")
    );
}

#[test]
fn require_doesnt_exist() {
    assert_eq!(
        "Error: tests/errors/require_doesnt_exist.nm:1:1: cannot find file named \
         \"a_file_that_doesnt_exist.nm\"\n",
        test("tests/errors/require_doesnt_exist.nm")
    );
}
