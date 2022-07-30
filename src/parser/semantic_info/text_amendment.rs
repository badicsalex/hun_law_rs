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
use hun_law_grammar::*;

use super::{abbreviation::AbbreviationCache, reference::GetOutgoingReferences};
use crate::reference;
use crate::semantic_info;

pub fn convert_text_amendment(
    abbreviation_cache: &AbbreviationCache,
    elem: &TextAmendment,
) -> Result<semantic_info::TextAmendment> {
    let positions = elem
        .get_outgoing_references(abbreviation_cache)?
        .into_iter()
        .map(reference::Reference::from)
        .filter(|r| !r.is_act_only())
        .collect();
    let replacements = elem.parts.iter().map(From::from).collect();

    Ok(semantic_info::TextAmendment {
        positions,
        replacements,
    })
}

impl From<&TextAmendmentPart> for semantic_info::TextAmendmentReplacement {
    fn from(tap: &TextAmendmentPart) -> Self {
        Self {
            from: tap.original_text.clone(),
            to: tap.replacement_text.clone(),
        }
    }
}
