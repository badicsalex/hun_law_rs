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

use super::abbreviation::AbbreviationCache;
use super::reference::{FeedReferenceBuilder, OutgoingReferenceBuilder};
use crate::semantic_info::TextAmendmentReference;
use crate::semantic_info::{self, TextAmendmentSAEPart};

pub fn convert_text_amendment(
    abbreviation_cache: &AbbreviationCache,
    elem: &TextAmendment,
) -> Result<semantic_info::TextAmendment> {
    let mut positions = Vec::new();
    let mut ref_builder = OutgoingReferenceBuilder::new(abbreviation_cache);
    ref_builder.feed(&elem.act_reference)?;
    for ta_reference in &elem.references {
        ref_builder.feed(&ta_reference.reference)?;
        for reference in ref_builder.take_result() {
            if !reference.reference.is_act_only() {
                positions.push(TextAmendmentReference {
                    reference: reference.reference,
                    amended_part: convert_intro_wrapup_token(&ta_reference.token),
                })
            }
        }
    }
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

fn convert_intro_wrapup_token(
    token: &Option<ReferenceWithIntroWrapup_token>,
) -> TextAmendmentSAEPart {
    match token {
        Some(ReferenceWithIntroWrapup_token::IntroToken(_)) => TextAmendmentSAEPart::IntroOnly,
        Some(ReferenceWithIntroWrapup_token::WrapUpToken(_)) => TextAmendmentSAEPart::WrapUpOnly,
        None => TextAmendmentSAEPart::All,
    }
}
