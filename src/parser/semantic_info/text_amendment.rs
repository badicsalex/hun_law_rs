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
use super::reference::{convert_act_reference, FeedReferenceBuilder, OutgoingReferenceBuilder};
use crate::reference::structural::{StructuralReference, StructuralReferenceParent};
use crate::semantic_info::OutgoingReference;
use crate::semantic_info::{self, TextAmendmentSAEPart};

pub fn convert_text_amendment(
    abbreviation_cache: &AbbreviationCache,
    elem: &TextAmendment,
) -> Result<Vec<semantic_info::TextAmendment>> {
    let mut result = Vec::new();
    let mut ref_builder = OutgoingReferenceBuilder::new(abbreviation_cache);
    ref_builder.feed(&elem.act_reference)?;
    for reference in convert_text_amendment_references(
        &elem.act_reference,
        &elem.references,
        abbreviation_cache,
    )? {
        for TextAmendmentPart { from, to } in &elem.parts {
            result.push(semantic_info::TextAmendment {
                reference: reference.clone(),
                from: from.clone(),
                to: to.clone(),
            })
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

pub fn convert_text_amendment_references(
    act_reference: &ActReference,
    ta_references: &[TextAmendmentReference],
    abbreviation_cache: &AbbreviationCache,
) -> Result<Vec<semantic_info::TextAmendmentReference>> {
    let mut result = Vec::new();
    let mut ref_builder = OutgoingReferenceBuilder::new(abbreviation_cache);
    let act_id = convert_act_reference(abbreviation_cache, act_reference)?;
    ref_builder.feed(act_reference)?;
    for ta_reference in ta_references {
        match ta_reference {
            TextAmendmentReference::TextAmendmentStructuralReference(raw_struct_ref) => {
                let mut struct_ref = StructuralReference::try_from(raw_struct_ref)?;
                struct_ref.act = Some(act_id);
                result.push(semantic_info::TextAmendmentReference::Structural(
                    struct_ref,
                ));
            }
            TextAmendmentReference::ArticleTitleReference(atr) => {
                ref_builder.feed(atr)?;
                for OutgoingReference { reference, .. } in ref_builder.take_result() {
                    if !reference.is_act_only() {
                        result.push(semantic_info::TextAmendmentReference::ArticleTitle(
                            reference.clone(),
                        ));
                    }
                }
            }
            TextAmendmentReference::ReferenceWithIntroWrapup(rwiw) => {
                ref_builder.feed(rwiw)?;
                for OutgoingReference { reference, .. } in ref_builder.take_result() {
                    if !reference.is_act_only() {
                        result.push(semantic_info::TextAmendmentReference::SAE {
                            reference: reference.clone(),
                            amended_part: convert_intro_wrapup_token(&rwiw.token),
                        });
                    }
                }
            }
        };
    }
    Ok(result)
}

impl TryFrom<&TextAmendmentStructuralReference> for StructuralReference {
    type Error = anyhow::Error;

    fn try_from(value: &TextAmendmentStructuralReference) -> Result<Self, Self::Error> {
        let parent = value
            .parent
            .as_ref()
            .map(StructuralReferenceParent::try_from)
            .transpose()?;

        Ok(match &value.child {
            TextAmendmentStructuralReference_child::AnyStructuralReference(asr) => {
                let mut sr = StructuralReference::try_from(asr)?;
                sr.parent = parent;
                sr
            }
            TextAmendmentStructuralReference_child::ArticleRelativePosition(arr) => {
                StructuralReference {
                    act: None,
                    book: None,
                    parent,
                    structural_element: arr.try_into()?,
                    // TODO:
                    title_only: false,
                }
            }
        })
    }
}
