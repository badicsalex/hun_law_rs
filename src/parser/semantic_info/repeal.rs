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

use anyhow::{bail, ensure, Result};
use hun_law_grammar::*;

use super::abbreviation::AbbreviationCache;
use super::reference::{convert_act_reference, FeedReferenceBuilder, OutgoingReferenceBuilder};
use super::text_amendment::convert_text_amendment_references;
use crate::semantic_info::{self, RepealReference, TextAmendmentSAEPart};

pub fn convert_repeal(
    abbreviation_cache: &AbbreviationCache,
    elem: &Repeal,
) -> Result<semantic_info::SpecialPhrase> {
    if elem.texts.is_empty() {
        Ok(convert_element_repeal(abbreviation_cache, elem)?.into())
    } else {
        let mut result = Vec::new();
        let mut ref_builder = OutgoingReferenceBuilder::new(abbreviation_cache);
        ref_builder.feed(&elem.act_reference)?;
        for reference in convert_text_amendment_references(
            &elem.act_reference,
            &elem.references,
            abbreviation_cache,
        )? {
            for text in &elem.texts {
                result.push(semantic_info::TextAmendment {
                    reference: reference.clone(),
                    from: text.clone(),
                    to: String::new(),
                })
            }
        }
        Ok(result.into())
    }
}
pub fn convert_element_repeal(
    abbreviation_cache: &AbbreviationCache,
    elem: &Repeal,
) -> Result<Vec<RepealReference>> {
    let mut result = Vec::new();
    for ta_reference in convert_text_amendment_references(
        &elem.act_reference,
        &elem.references,
        abbreviation_cache,
    )? {
        result.push(match ta_reference {
            semantic_info::TextAmendmentReference::SAE {
                reference,
                amended_part,
            } => {
                ensure!(
                    amended_part == TextAmendmentSAEPart::All,
                    "Cannot repeal part of a SAE ({reference:?}, {amended_part:?})"
                );
                RepealReference::Reference(reference)
            }
            semantic_info::TextAmendmentReference::Structural(reference) => {
                RepealReference::StructuralReference(reference)
            }
            semantic_info::TextAmendmentReference::ArticleTitle(at) => {
                bail!("Cannot repeal article title {at:?}")
            }
        })
    }
    if result.is_empty() {
        let act_id = convert_act_reference(abbreviation_cache, &elem.act_reference)?;
        result.push(RepealReference::Reference(act_id.into()));
    }
    Ok(result)
}
/*
pub fn convert_structural_repeal(
    abbreviation_cache: &AbbreviationCache,
    elem: &StructuralRepeal,
) -> Result<semantic_info::StructuralRepeal> {
    let act = Some(convert_act_reference(
        abbreviation_cache,
        &elem.act_reference,
    )?);
    let position = match &elem.position {
        StructuralRepeal_position::AnyStructuralReference(asr) => {
            let mut sr = StructuralReference::try_from(asr)?;
            sr.act = act;
            sr
        }
        StructuralRepeal_position::ArticleRelativePosition(arp) => StructuralReference {
            act,
            book: None,
            parent: None,
            structural_element: arp.try_into()?,
            title_only: false,
        },
    };
    Ok(semantic_info::StructuralRepeal { position })
}*/
