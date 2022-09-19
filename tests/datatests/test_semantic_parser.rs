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

use std::{collections::BTreeMap, path::Path};

use datatest_stable::Result;
use hun_law::{
    identifier::ActIdentifier,
    parser::semantic_info::{abbreviation::AbbreviationCache, sae::SemanticInfoAdder},
    reference::Reference,
    semantic_info::{OutgoingReference, SpecialPhrase},
    util::singleton_yaml,
};
use serde::{Deserialize, Serialize};

use crate::declare_test;
use crate::test_utils::{ensure_eq, read_all};

declare_test!(dir = "data_semantic_parser", pattern = r"\.yml");

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TestCase {
    pub text: String,
    pub positions: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub abbreviations: BTreeMap<String, ActIdentifier>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub expected_new_abbreviations: BTreeMap<String, ActIdentifier>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expected_references: Vec<Reference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_special_phrase: Option<SpecialPhrase>,
}

pub fn run_test(path: &Path) -> Result<()> {
    let test_case: TestCase = singleton_yaml::from_slice(&read_all(path)?)?;
    let mut abbreviation_cache = AbbreviationCache::from(test_case.abbreviations.clone());
    let mut visitor = SemanticInfoAdder::new(&mut abbreviation_cache);
    let semantic_info = visitor.extract_semantic_info(&test_case.text)?;

    let (expected_references, positions) =
        convert_references(&semantic_info.outgoing_references, &test_case.text);

    let result = TestCase {
        text: test_case.text.clone(),
        positions,
        abbreviations: test_case.abbreviations.clone(),
        expected_new_abbreviations: semantic_info.new_abbreviations,
        expected_references,
        expected_special_phrase: semantic_info.special_phrase,
    };
    ensure_eq(&test_case, &result, "Semantic info incorrect")?;
    Ok(())
}

fn convert_references(
    outgoing_references: &[OutgoingReference],
    text: &str,
) -> (Vec<Reference>, String) {
    let mut parsed_refs = Vec::new();
    let mut parsed_positions = vec![b' '; text.chars().count()];

    for outgoing_reference in outgoing_references {
        parsed_refs.push(outgoing_reference.reference.clone());
        let start_char_index = text
            .char_indices()
            .position(|(cp, _)| cp == outgoing_reference.start)
            .unwrap();
        let end_char_index = text
            .char_indices()
            .position(|(cp, _)| cp == outgoing_reference.end)
            .unwrap();
        parsed_positions[start_char_index] = b'<';
        parsed_positions[end_char_index - 1] = b'>';
    }

    let parsed_positions = String::from_utf8(parsed_positions).unwrap();
    (parsed_refs, parsed_positions)
}
