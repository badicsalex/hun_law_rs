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

use hun_law::{
    parser::grammar::GetOutgoingReferences, reference::Reference, structure::ActIdentifier,
};
use hun_law_grammar::{ListOfSimpleExpressions, PegParser};

use serde::Deserialize;
use std::{collections::HashMap, path::Path};

use crate::test_utils::read_all;

use pretty_assertions::assert_eq;

#[derive(Debug, Deserialize)]
struct TestCase {
    pub text: String,
    pub positions: String,
    #[serde(default)]
    pub abbreviations: HashMap<String, ActIdentifier>,
    pub expected: Vec<Reference>,
}

use crate::declare_test;
declare_test!(dir = "data_reference_parsing", pattern = r"\.yml");
pub fn run_test(path: &Path) -> datatest_stable::Result<()> {
    let test_case: TestCase = serde_yaml::from_slice(&read_all(path)?)?;
    let parsed = ListOfSimpleExpressions::parse(&test_case.text)?;
    let mut parsed_refs = Vec::new();
    let mut parsed_positions = vec![b' '; test_case.positions.len()];

    for outgoing_reference in parsed.get_outgoing_references(&test_case.abbreviations)? {
        parsed_refs.push(outgoing_reference.reference.clone());
        let start_char_index = test_case
            .text
            .char_indices()
            .position(|(cp, _)| cp == outgoing_reference.start)
            .unwrap();
        let end_char_index = test_case
            .text
            .char_indices()
            .position(|(cp, _)| cp == outgoing_reference.end)
            .unwrap();
        parsed_positions[start_char_index] = b'<';
        parsed_positions[end_char_index - 1] = b'>';
    }

    assert_eq!(parsed_refs, test_case.expected);
    assert_eq!(String::from_utf8(parsed_positions)?, test_case.positions);
    Ok(())
}
