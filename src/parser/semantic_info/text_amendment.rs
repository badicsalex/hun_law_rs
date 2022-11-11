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
use crate::semantic_info::OutgoingReference;
use crate::semantic_info::{self, TextAmendmentSAEPart};

pub fn convert_text_amendment(
    abbreviation_cache: &AbbreviationCache,
    elem: &TextAmendment,
) -> Result<Vec<semantic_info::TextAmendment>> {
    let mut result = Vec::new();
    let mut ref_builder = OutgoingReferenceBuilder::new(abbreviation_cache);
    ref_builder.feed(&elem.act_reference)?;
    for ta_reference in &elem.references {
        ref_builder.feed(&ta_reference.reference)?;
        for OutgoingReference { reference, .. } in ref_builder.take_result() {
            if !reference.is_act_only() {
                for TextAmendmentPart { from, to } in &elem.parts {
                    result.push(semantic_info::TextAmendment {
                        reference: reference.clone(),
                        amended_part: convert_intro_wrapup_token(&ta_reference.token),
                        from: from.clone(),
                        to: to.clone(),
                    })
                }
            }
        }
    }

    Ok(result)
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
