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

use anyhow::Result;
use hun_law_grammar::PegParser;

use crate::structure::semantic_info::SemanticInfo;
use abbreviation::AbbreviationCache;
use reference::GetOutgoingReferences;

pub mod abbreviation;
pub mod reference;

pub fn extract_semantic_info(
    s: &str,
    abbreviation_cache: &mut AbbreviationCache,
) -> Result<SemanticInfo> {
    let parsed = hun_law_grammar::Root::parse(s)?;
    let new_abbreviations = Vec::new();
    let outgoing_references = parsed.get_outgoing_references(abbreviation_cache)?;
    let special_phrase = None;
    Ok(SemanticInfo {
        outgoing_references,
        new_abbreviations,
        special_phrase,
    })
}
