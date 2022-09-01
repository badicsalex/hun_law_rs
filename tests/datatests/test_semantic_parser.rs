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

use std::{collections::HashMap, path::Path};

use datatest_stable::Result;
use hun_law::{
    identifier::ActIdentifier,
    parser::semantic_info::{abbreviation::AbbreviationCache, extract_semantic_info},
    reference::Reference,
    semantic_info::{ActIdAbbreviation, OutgoingReference, SpecialPhrase},
    util::is_default,
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
    #[serde(default, skip_serializing_if = "is_default")]
    pub abbreviations: HashMap<String, ActIdentifier>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub expected_new_abbreviations: HashMap<String, ActIdentifier>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub expected_references: Vec<Reference>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub expected_special_phrase: Option<SpecialPhrase>,
}

pub fn run_test(path: &Path) -> Result<()> {
    let test_case: TestCase = serde_yaml::from_slice(&read_all(path)?)?;
    let mut abbreviation_cache = AbbreviationCache::from(test_case.abbreviations.clone());
    let semantic_info = extract_semantic_info("", &test_case.text, "", &mut abbreviation_cache)?;

    let (expected_references, positions) =
        convert_references(&semantic_info.outgoing_references, &test_case.text);
    let expected_new_abbreviations = convert_abbreviations(&semantic_info.new_abbreviations);

    let result = TestCase {
        text: test_case.text.clone(),
        positions,
        abbreviations: test_case.abbreviations.clone(),
        expected_new_abbreviations,
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

fn convert_abbreviations(
    new_abbreviations: &[ActIdAbbreviation],
) -> HashMap<String, ActIdentifier> {
    new_abbreviations
        .iter()
        .map(|a| (a.abbreviation.clone(), a.act_id))
        .collect()
}
