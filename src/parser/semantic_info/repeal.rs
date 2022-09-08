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
use super::{abbreviation::AbbreviationCache, reference::GetOutgoingReferences};
use crate::reference::{self, structural::StructuralReference};
use crate::semantic_info;

pub fn convert_repeal(
    abbreviation_cache: &AbbreviationCache,
    elem: &Repeal,
) -> Result<semantic_info::Repeal> {
    let positions = elem
        .get_outgoing_references(abbreviation_cache)?
        .into_iter()
        .map(reference::Reference::from)
        .filter(|r| !r.is_act_only())
        .collect();

    Ok(semantic_info::Repeal {
        positions,
        texts: elem.texts.clone(),
    })
}

pub fn convert_structural_repeal(
    abbreviation_cache: &AbbreviationCache,
    elem: &StructuralRepeal,
) -> Result<semantic_info::StructuralRepeal> {
    let position = StructuralReference {
        act: Some(convert_act_reference(
            abbreviation_cache,
            &elem.act_reference,
        )?),
        book: None,
        structural_element: (&elem.reference).try_into()?,
    };
    Ok(semantic_info::StructuralRepeal { position })
}
