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

use self::{
    abbreviation::{get_new_abbreviations, AbbreviationCache},
    enforcement_date::convert_enforcement_date,
    reference::GetOutgoingReferences,
    repeal::convert_repeal,
    text_amendment::convert_text_amendment,
};
use crate::structure::semantic_info::{SemanticInfo, SpecialPhrase};

pub mod abbreviation;
pub mod enforcement_date;
pub mod reference;
pub mod repeal;
pub mod text_amendment;

pub fn extract_semantic_info(
    s: &str,
    abbreviation_cache: &mut AbbreviationCache,
) -> Result<SemanticInfo> {
    let parsed = hun_law_grammar::Root::parse(s)?;
    let new_abbreviations = get_new_abbreviations(&parsed)?;
    abbreviation_cache.add_multiple(&new_abbreviations);
    let outgoing_references = parsed.get_outgoing_references(abbreviation_cache)?;
    let special_phrase = extract_special_phrase(abbreviation_cache, &parsed)?;
    Ok(SemanticInfo {
        outgoing_references,
        new_abbreviations,
        special_phrase,
    })
}

pub fn extract_special_phrase(
    abbreviation_cache: &AbbreviationCache,
    root: &hun_law_grammar::Root,
) -> Result<Option<SpecialPhrase>> {
    Ok(match &root.content {
        hun_law_grammar::Root_content::ArticleTitleAmendment(_) => None, // TODO
        hun_law_grammar::Root_content::BlockAmendment(_) => None,        // TODO
        hun_law_grammar::Root_content::BlockAmendmentStructural(_) => None, // TODO
        hun_law_grammar::Root_content::BlockAmendmentWithSubtitle(_) => None, // TODO
        hun_law_grammar::Root_content::EnforcementDate(x) => {
            Some(convert_enforcement_date(abbreviation_cache, x)?.into())
        }
        hun_law_grammar::Root_content::ListOfSimpleExpressions(_) => None,
        hun_law_grammar::Root_content::Repeal(x) => {
            Some(convert_repeal(abbreviation_cache, x)?.into())
        }
        hun_law_grammar::Root_content::StructuralRepeal(_) => None, // TODO
        hun_law_grammar::Root_content::TextAmendment(x) => {
            Some(convert_text_amendment(abbreviation_cache, x)?.into())
        }
    })
}
