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
    parser::semantic_info::{abbreviation::AbbreviationCache, extract_semantic_info},
    reference::Reference,
    structure::{
        semantic_info::{ActIdAbbreviation, OutgoingReference, SpecialPhrase},
        ActIdentifier,
    },
};

use datatest_stable::Result;
use serde::Deserialize;
use std::{collections::HashMap, path::Path};

use crate::test_utils::read_all;

macro_rules! ensure_eq {
    ($left: expr, $right: expr, $message: expr) => {
        if ($left) != ($right) {
            return Err(format!(
                "{}\n {}\n!=\n{}",
                $message,
                serde_yaml::to_string(&$left).unwrap(),
                serde_yaml::to_string(&$right).unwrap()
            )
            .into());
        };
    };
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TestCase {
    pub text: String,
    pub positions: String,
    #[serde(default)]
    pub abbreviations: HashMap<String, ActIdentifier>,
    #[serde(default)]
    pub expected_new_abbreviations: HashMap<String, ActIdentifier>,
    #[serde(default)]
    pub expected_references: Vec<Reference>,
    #[serde(default)]
    pub expected_special_phrase: Option<SpecialPhrase>,
}

use crate::declare_test;
declare_test!(dir = "data_semantic_parser", pattern = r"\.yml");
pub fn run_test(path: &Path) -> Result<()> {
    let test_case: TestCase = serde_yaml::from_slice(&read_all(path)?)?;
    let mut abbreviation_cache = AbbreviationCache::from(test_case.abbreviations);
    let semantic_info = extract_semantic_info(&test_case.text, &mut abbreviation_cache)?;

    check_references(
        &semantic_info.outgoing_references,
        &test_case.expected_references,
        &test_case.text,
        &test_case.positions,
    )?;
    check_abbreviations(
        &semantic_info.new_abbreviations,
        &test_case.expected_new_abbreviations,
    )?;
    ensure_eq!(
        semantic_info.special_phrase,
        test_case.expected_special_phrase,
        "Special phrase was not correct"
    );
    Ok(())
}

fn check_references(
    outgoing_references: &[OutgoingReference],
    expected: &[Reference],
    text: &str,
    expected_positions: &str,
) -> Result<()> {
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
    ensure_eq!(&parsed_refs, expected, "References were not the same");
    ensure_eq!(
        &parsed_positions,
        &expected_positions,
        "Reference positions were not the same"
    );
    Ok(())
}

fn check_abbreviations(
    new_abbreviations: &[ActIdAbbreviation],
    expected_abbreviations: &HashMap<String, ActIdentifier>,
) -> Result<()> {
    let expected_abbreviations: Vec<_> = expected_abbreviations
        .iter()
        .map(|(k, v)| ActIdAbbreviation {
            act_id: *v,
            abbreviation: k.clone(),
        })
        .collect();
    ensure_eq!(
        new_abbreviations,
        &expected_abbreviations,
        "Abbreviations were not the same"
    );
    Ok(())
}
