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

use anyhow::{anyhow, Result};

use super::{abbreviation::AbbreviationCache, reference::GetOutgoingReferences};
use crate::reference;
use crate::structure::semantic_info;
use hun_law_grammar::*;

pub fn convert_block_amendment(
    abbreviation_cache: &AbbreviationCache,
    elem: &BlockAmendment,
) -> Result<semantic_info::BlockAmendment> {
    let all_positions: Vec<_> = elem
        .get_outgoing_references(abbreviation_cache)?
        .into_iter()
        .map(reference::Reference::from)
        .filter(|r| !r.is_act_only())
        .collect();

    let first = all_positions
        .first()
        .ok_or_else(|| anyhow!("No references found in block amendment"))?;
    let last = all_positions
        .last()
        .ok_or_else(|| anyhow!("No references found in block amendment"))?;
    let position = reference::Reference::make_range(first, last)?;

    Ok(semantic_info::BlockAmendment {
        position,
        pure_insertion: elem.amended_reference.is_none(),
    })
}
