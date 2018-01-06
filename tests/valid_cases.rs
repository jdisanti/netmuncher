//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

extern crate netmuncher;
extern crate error_chain;

use std::fs::File;
use std::io::prelude::*;

use netmuncher::circuit::Circuit;

fn load(file_name: &str) -> String {
    let mut file = File::open(file_name).unwrap();
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents).unwrap();
    file_contents
}

fn compile(file_name: &str) -> String {
    format!("{}", Circuit::compile(file_name).unwrap())
}

#[test]
fn nand_indicator() {
    let expected = load("tests/valid/nand_indicator.net");
    let actual = compile("tests/valid/nand_indicator.nm");
    assert_eq!(expected, actual);
}

#[test]
fn module_test() {
    let expected = load("tests/valid/module_test/main.net");
    let actual = compile("tests/valid/module_test/main.nm");
    assert_eq!(expected, actual);
}

#[test]
fn no_connects() {
    let expected = load("tests/valid/no_connects.net");
    let actual = compile("tests/valid/no_connects.nm");
    assert_eq!(expected, actual);
}