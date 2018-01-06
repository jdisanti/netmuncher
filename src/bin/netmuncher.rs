//
// Copyright 2018 netmuncher Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

extern crate clap;
extern crate error_chain;
extern crate netmuncher;

use std::process;
use std::fs::File;
use std::io::prelude::*;

use error_chain::ChainedError;
use netmuncher::circuit::Circuit;

fn main() {
    let matches = clap::App::new("netmuncher")
        .version("0.1")
        .author("John DiSanti")
        .about("Hierarchical circuit definition to netlist transformer")
        .arg(
            clap::Arg::with_name("INPUT")
                .help("input source file")
                .required(true),
        )
        .arg(
            clap::Arg::with_name("OUTPUT")
                .help("output file name")
                .short("o")
                .long("output")
                .value_name("OUTPUT")
                .takes_value(true),
        )
        .get_matches();

    let input_file_name = matches.value_of("INPUT").unwrap();
    let output_file_name: String = matches
        .value_of("OUTPUT")
        .map(|o| String::from(o))
        .unwrap_or_else(|| format!("{}.net", input_file_name));
    let circuit = match Circuit::compile(input_file_name) {
        Ok(circuit) => circuit,
        Err(err) => {
            println!("{}", err.display_chain().to_string());
            process::exit(1);
        }
    };

    let output = format!("{}", circuit);
    let mut file = match File::create(output_file_name) {
        Ok(file) => file,
        Err(err) => {
            println!("Failed to create output file: {}", err);
            process::exit(1);
        }
    };
    if let Err(err) = file.write_all(output.as_bytes()) {
        println!("Failed to write output file: {}", err);
        process::exit(1);
    }
}
