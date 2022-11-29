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

#[allow(clippy::all)]
mod grammar_generated;

pub use grammar_generated::*;
use peginator::{NoopTracer, ParseError, PegParserAdvanced};

#[derive(Debug, Default)]
pub struct CustomParseState {
    known_abbreviations: Vec<String>,
}

pub fn grammar_parse(s: &str, known_abbreviations: Vec<String>) -> Result<Root, ParseError> {
    let mut parse_state = CustomParseState {
        known_abbreviations,
    };
    Root::parse_advanced::<NoopTracer>(s, &Default::default(), &mut parse_state)
}

pub fn store_abbreviation(c: &ActIdWithFromNowOn, state: &mut CustomParseState) -> bool {
    if let Some(abbrev) = &c.abbreviation {
        state.known_abbreviations.push(abbrev.clone())
    }
    true
}

pub fn parse_abbreviation(
    s: &str,
    state: &mut CustomParseState,
) -> Result<(String, usize), &'static str> {
    for known_abbrev in &state.known_abbreviations {
        if s.starts_with(known_abbrev) {
            return Ok((known_abbrev.clone(), known_abbrev.len()));
        }
    }
    Err("Not a known abbreviation")
}
