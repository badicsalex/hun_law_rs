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

use anyhow::{anyhow, bail, Result};
use hun_law_grammar::*;

use super::{
    abbreviation::AbbreviationCache,
    reference::{convert_act_reference, GetOutgoingReferences},
};
use crate::{
    reference::{
        self,
        parts::AnyReferencePart,
        structural::{StructuralReference, StructuralReferenceElement, StructuralReferenceParent},
    },
    semantic_info,
};

pub fn convert_block_amendment(
    abbreviation_cache: &AbbreviationCache,
    elem: &BlockAmendment,
) -> Result<semantic_info::SpecialPhrase> {
    let all_positions: Vec<_> = elem
        .get_outgoing_references(abbreviation_cache)?
        .into_iter()
        .map(reference::Reference::from)
        .filter(|r| !r.is_act_only())
        .collect();

    let first = all_positions
        .iter()
        .map(|r| r.first_in_range())
        .min()
        .ok_or_else(|| anyhow!("No references found in block amendment"))?;
    let last = all_positions
        .iter()
        .map(|r| r.last_in_range())
        .max()
        .ok_or_else(|| anyhow!("No references found in block amendment"))?;
    let position = reference::Reference::make_range(&first, &last)?;

    if let AnyReferencePart::Article(article_id) = position.get_last_part() {
        Ok(semantic_info::StructuralBlockAmendment {
            position: StructuralReference {
                act: position.act(),
                book: None,
                parent: elem.parent.as_ref().map(|p| p.try_into()).transpose()?,
                structural_element: StructuralReferenceElement::Article(article_id),
                title_only: false,
            },
            pure_insertion: elem.amended_reference.is_none(),
        }
        .into())
    } else {
        Ok(semantic_info::BlockAmendment {
            position,
            pure_insertion: elem.amended_reference.is_none(),
        }
        .into())
    }
}

pub fn convert_structural_block_amendment(
    abbreviation_cache: &AbbreviationCache,
    elem: &BlockAmendmentStructural,
) -> Result<semantic_info::StructuralBlockAmendment> {
    let act = Some(convert_act_reference(
        abbreviation_cache,
        &elem.act_reference,
    )?);
    let mut position = if let Some(parent) = &elem.parent {
        // TODO: we lose book information!
        let parent = Some(StructuralReferenceParent::try_from(parent)?);
        StructuralReference {
            act,
            parent,
            ..StructuralReference::try_from(&elem.reference)?
        }
    } else {
        StructuralReference {
            act,
            ..StructuralReference::try_from(&elem.reference)?
        }
    };
    if let Some(book) = &elem.book {
        position.book = Some(book.try_into()?);
    }
    Ok(semantic_info::StructuralBlockAmendment {
        position,
        pure_insertion: elem.is_insertion.is_some(),
    })
}

pub fn convert_subtitle_block_amendment(
    abbreviation_cache: &AbbreviationCache,
    elem: &BlockAmendmentWithSubtitle,
) -> Result<semantic_info::StructuralBlockAmendment> {
    let pure_insertion = elem.is_insertion.is_some();
    let structural_element = if let Some(article) = &elem.reference.article {
        StructuralReferenceElement::SubtitleBeforeArticleInclusive(article.try_into()?)
    } else if let Some(article_relative) = &elem.article_relative {
        article_relative.try_into()?
    } else if let Some(title) = &elem.reference.title {
        StructuralReferenceElement::SubtitleTitle(title.clone())
    } else if let Some(id) = &elem.reference.id {
        StructuralReferenceElement::SubtitleId(id.parse()?)
    } else if pure_insertion {
        // This is a best effort thing and _might_ be caused by problems with the
        // grammar, but unfortunately this really is somewhat common
        StructuralReferenceElement::SubtitleUnknown
    } else {
        bail!("No article found at all for amendment-type BlockAmendmentWithSubtitle")
    };
    let parent = elem.parent.as_ref().map(|p| p.try_into()).transpose()?;

    let position = StructuralReference {
        act: Some(convert_act_reference(
            abbreviation_cache,
            &elem.act_reference,
        )?),
        book: None,
        parent,
        structural_element,
        title_only: false,
    };
    Ok(semantic_info::StructuralBlockAmendment {
        position,
        pure_insertion,
    })
}
