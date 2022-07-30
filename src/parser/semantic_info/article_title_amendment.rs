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

use anyhow::{anyhow, ensure, Result};
use hun_law_grammar::*;

use super::abbreviation::AbbreviationCache;
use super::reference::GetOutgoingReferences;
use crate::reference;
use crate::semantic_info;

pub fn convert_article_title_amendment(
    abbreviation_cache: &AbbreviationCache,
    elem: &ArticleTitleAmendment,
) -> Result<semantic_info::ArticleTitleAmendment> {
    let mut positions_iter = elem
        .get_outgoing_references(abbreviation_cache)?
        .into_iter()
        .map(reference::Reference::from)
        .filter(|r| !r.is_act_only());

    let position = positions_iter
        .next()
        .ok_or_else(|| anyhow!("Too many references found in article titel amendment"))?;
    ensure!(
        positions_iter.next().is_none(),
        "Too many references found in article title amendment"
    );

    Ok(semantic_info::ArticleTitleAmendment {
        position,
        replacement: semantic_info::TextAmendmentReplacement {
            from: elem.original_text.clone(),
            to: elem.replacement_text.clone(),
        },
    })
}
