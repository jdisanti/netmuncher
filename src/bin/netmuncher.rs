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

use std::fs::File;
use std::io::prelude::*;
use std::process;

use error_chain::ChainedError;
use netmuncher::circuit::{Circuit, DotSerializer, KicadNetListSerializer, SerializeCircuit};
use netmuncher::diagram::Diagram;

fn main() {
    let matches = clap::App::new("netmuncher")
        .version("0.5.0")
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
        .arg(
            clap::Arg::with_name("FORMAT")
                .help("set output format")
                .short("f")
                .long("format")
                .value_name("FORMAT")
                .takes_value(true),
        )
        .get_matches();

    let input_file_name = matches.value_of("INPUT").unwrap();
    let output_file_name = matches.value_of("OUTPUT");
    let format = matches.value_of("FORMAT").unwrap_or("kicad");

    match format {
        "dot" | "kicad" => compile_circuit(input_file_name, output_file_name, format),
        "diagram_json" => compile_diagram(input_file_name, output_file_name, format),
        _ => {
            println!("Unknown output format: {}", format);
            process::exit(1);
        }
    }
}

fn compile_circuit(input_file_name: &str, optional_output_file_name: Option<&str>, format: &str) {
    let circuit = match Circuit::compile(input_file_name) {
        Ok(circuit) => circuit,
        Err(err) => {
            println!("{}", err.display_chain().to_string());
            process::exit(1);
        }
    };

    let output_file_name = optional_output_file_name
        .map(|n| String::from(n))
        .unwrap_or_else(|| format!("{}.net", input_file_name));

    let output_result = match format {
        "kicad" => KicadNetListSerializer::new().serialize(&circuit),
        "dot" => DotSerializer::new().serialize(&circuit),
        _ => unreachable!(),
    };
    let output = match output_result {
        Ok(out) => out,
        Err(err) => {
            println!("Failed to serialize: {}", err);
            process::exit(1);
        }
    };

    write_file(&output_file_name, output);
}

fn compile_diagram(input_file_name: &str, optional_output_file_name: Option<&str>, _format: &str) {
    let diagram = match Diagram::compile(input_file_name) {
        Ok(diagram) => diagram,
        Err(err) => {
            println!("{}", err.display_chain().to_string());
            process::exit(1);
        }
    };

    let output_file_name = optional_output_file_name
        .map(|n| String::from(n))
        .unwrap_or_else(|| format!("{}.json", input_file_name));

    let output = diagram.to_json_bytes().unwrap();
    write_file(&output_file_name, output);
}

fn write_file(file_name: &str, contents: Vec<u8>) {
    let mut file = match File::create(file_name) {
        Ok(file) => file,
        Err(err) => {
            println!("Failed to create output file: {}", err);
            process::exit(1);
        }
    };
    if let Err(err) = file.write_all(&contents) {
        println!("Failed to write output file: {}", err);
        process::exit(1);
    }
}
