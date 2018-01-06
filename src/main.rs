//
// Copyright 2017 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

extern crate clap;
extern crate lalrpop_util;
extern crate regex;

#[macro_use]
extern crate error_chain;

use std::fs::File;
use std::io::prelude::*;
use std::process;

mod circuit;
mod error;
mod parse;

fn main() {
    let matches = clap::App::new("netmuncher")
        .version("0.1")
        .author("John DiSanti")
        .about("Textual circuit definition to netlist transformer")
        .arg(
            clap::Arg::with_name("INPUT")
                .help("input source file")
                .required(true),
        )
        .get_matches();

    let input_source = {
        let mut file = match File::open(matches.value_of("INPUT").unwrap()) {
            Ok(file) => file,
            Err(e) => {
                println!("Failed to open input source file: {}", e);
                process::exit(1);
            }
        };
        let mut file_contents = String::new();
        match file.read_to_string(&mut file_contents) {
            Err(e) => {
                println!("Failed to read input source file: {}", e);
                process::exit(1);
            }
            _ => {}
        }
        file_contents
    };

    let components = parse::parse_components(&input_source).unwrap();
    let circuit = circuit::Circuit::from_components(components).unwrap();
    println!("{}", circuit);
}
