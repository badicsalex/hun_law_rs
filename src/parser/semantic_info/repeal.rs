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

use super::reference::convert_act_reference;
use super::text_amendment::convert_text_amendment;
use super::{abbreviation::AbbreviationCache, reference::GetOutgoingReferences};
use crate::reference::{self, structural::StructuralReference};
use crate::semantic_info;

pub fn convert_repeal(
    abbreviation_cache: &AbbreviationCache,
    elem: &Repeal,
) -> Result<semantic_info::SpecialPhrase> {
    if elem.texts.is_empty() {
        Ok(convert_element_repeal(abbreviation_cache, elem)?.into())
    } else {
        // Text repeals are jsut spicy text amendments
        let fake_ta = TextAmendment {
            act_reference: elem.act_reference.clone(),
            references: elem.references.clone(),
            parts: elem
                .texts
                .iter()
                .map(|p| TextAmendmentPart {
                    from: p.clone(),
                    to: String::new(),
                })
                .collect(),
        };
        Ok(convert_text_amendment(abbreviation_cache, &fake_ta)?.into())
    }
}

pub fn convert_element_repeal(
    abbreviation_cache: &AbbreviationCache,
    elem: &Repeal,
) -> Result<semantic_info::Repeal> {
    let mut positions: Vec<_> = elem
        .get_outgoing_references(abbreviation_cache)?
        .into_iter()
        .map(reference::Reference::from)
        .filter(|r| !r.is_act_only())
        .collect();
    if positions.is_empty() {
        let act_id = convert_act_reference(abbreviation_cache, &elem.act_reference)?;
        positions.push(act_id.into());
    }

    Ok(semantic_info::Repeal { positions })
}

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
}
