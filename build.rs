// Copyright (C) 2022, Alex Badics
//
// This file is part of Hun-Law.
//
// Hun-law is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3 of the License.
//
// Hun-law is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Hun-law. If not, see <http://www.gnu.org/licenses/>.

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

// https://stackoverflow.com/a/38406885
fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn generate_hun_strs() -> Vec<String> {
    let special_values = HashMap::from([
        (1, "első"),
        (2, "második"),
        (10, "tizedik"),
        (20, "huszadik"),
        (30, "harmincadik"),
        (40, "negyvenedik"),
        (50, "ötvenedik"),
        (60, "hatvanadik"),
        (70, "hetvenedik"),
        (80, "nyolcvanadik"),
        (90, "kilencvenedik"),
        (100, "századik"),
    ]);
    let ones_digit = [
        "nulladik",
        "egyedik",
        "kettedik",
        "harmadik",
        "negyedik",
        "ötödik",
        "hatodik",
        "hetedik",
        "nyolcadik",
        "kilencedik",
    ];
    let tens_digit = [
        "",
        "tizen",
        "huszon",
        "harminc",
        "negyven",
        "ötven",
        "hatvan",
        "hetven",
        "nyolcvan",
        "kilencven",
    ];
    let mut result = Vec::<String>::new();
    for (tens_val, tens_text) in tens_digit.iter().enumerate() {
        for (ones_val, ones_text) in ones_digit.iter().enumerate() {
            let value = tens_val * 10 + ones_val;
            if let Some(text) = special_values.get(&value) {
                result.push(text.to_string());
            } else {
                result.push(format!("{}{}", tens_text, ones_text));
            }
        }
    }
    result.push("századik".to_string());
    result
}

fn regen_phf() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("phf_generated.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    let mut phf_builder = phf_codegen::Map::<String>::new();

    let numerals = generate_hun_strs();
    // "Good enough for the demo, 1o1"
    for (val, s) in numerals.iter().enumerate() {
        phf_builder.entry(s.clone(), &val.to_string());
        phf_builder.entry(s.to_uppercase(), &val.to_string());
        phf_builder.entry(capitalize(s), &val.to_string());
    }

    writeln!(
        &mut file,
        "pub const STR_TO_INT_HUN: phf::Map<&'static str, u16> = \n{};",
        phf_builder.build()
    )
    .unwrap();

    writeln!(
        &mut file,
        "pub const INT_TO_STR_HUN: [&str; 101] = [{}];",
        numerals
            .iter()
            .map(|s| format!("{:?}, ", s))
            .collect::<String>()
    )
    .unwrap();
}

fn regen_grammar() {
    peginator::buildscript::Compile::file("src/grammar.ebnf")
        .destination("src/parser/grammar_generated.rs")
        .format()
        .run_exit_on_error();
    println!("cargo:rerun-if-changed=src/grammar.ebnf");
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    regen_phf();
    regen_grammar();
}
