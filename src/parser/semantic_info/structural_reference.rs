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

use crate::{
    identifier::{
        range::{IdentifierRange, IdentifierRangeFrom},
        NumericIdentifier,
    },
    reference::structural::{
        StructuralReference, StructuralReferenceElement, StructuralReferenceParent,
    },
    structure::StructuralElementType,
};

impl TryFrom<&AnyStructuralReference> for StructuralReference {
    type Error = anyhow::Error;

    fn try_from(value: &AnyStructuralReference) -> Result<Self, Self::Error> {
        Ok(Self {
            act: None,
            book: value.book.as_ref().map(TryFrom::try_from).transpose()?,
            parent: None,
            structural_element: StructuralReferenceParent::try_from(&value.reference)?.into(),
            title_only: value.title_only.is_some(),
        })
    }
}

impl TryFrom<&AnyStructuralReference> for StructuralReferenceParent {
    type Error = anyhow::Error;

    fn try_from(value: &AnyStructuralReference) -> Result<Self, Self::Error> {
        // TODO: we lose book information
        ensure!(
            value.title_only.is_none(),
            "Found title_only flag in StructuralReferenceParent context"
        );
        StructuralReferenceParent::try_from(&value.reference)
    }
}

impl TryFrom<&AnyStructuralReference_reference> for StructuralReferenceParent {
    type Error = anyhow::Error;

    fn try_from(value: &AnyStructuralReference_reference) -> Result<Self, Self::Error> {
        match value {
            AnyStructuralReference_reference::ChapterReference(x) => x.try_into(),
            AnyStructuralReference_reference::PartReference(x) => x.try_into(),
            AnyStructuralReference_reference::SubtitleReferencePart(x) => x.try_into(),
            AnyStructuralReference_reference::SubtitleTitle(x) => x.try_into(),
            AnyStructuralReference_reference::TitleReference(x) => x.try_into(),
        }
    }
}

impl TryFrom<&ChapterReference> for StructuralReferenceParent {
    type Error = anyhow::Error;

    fn try_from(value: &ChapterReference) -> Result<Self, Self::Error> {
        Ok(StructuralReferenceParent::Chapter(
            StructuralElementType::Chapter.parse_identifier(&value.id)?,
        ))
    }
}

impl TryFrom<&PartReference> for StructuralReferenceParent {
    type Error = anyhow::Error;

    fn try_from(value: &PartReference) -> Result<Self, Self::Error> {
        Ok(StructuralReferenceParent::Part(
            StructuralElementType::Part { is_special: false }.parse_identifier(&value.id)?,
        ))
    }
}

impl TryFrom<&TitleReference> for StructuralReferenceParent {
    type Error = anyhow::Error;

    fn try_from(value: &TitleReference) -> Result<Self, Self::Error> {
        Ok(StructuralReferenceParent::Title(
            StructuralElementType::Title.parse_identifier(&value.id)?,
        ))
    }
}

impl TryFrom<&SubtitleReferencePart> for StructuralReferenceParent {
    type Error = anyhow::Error;

    fn try_from(value: &SubtitleReferencePart) -> Result<Self, Self::Error> {
        if let Some(id) = &value.id {
            Ok(StructuralReferenceParent::SubtitleId(id.parse()?))
        } else if let (Some(start), Some(end)) = (&value.start, &value.end) {
            Ok(StructuralReferenceParent::SubtitleRange(
                IdentifierRange::from_range(start.parse()?, end.parse()?),
            ))
        } else {
            Err(anyhow!(
                "Grammar produced an invalid combination in SubtitleReferencePart"
            ))
        }
    }
}

impl TryFrom<&SubtitleTitle> for StructuralReferenceParent {
    type Error = anyhow::Error;

    fn try_from(value: &SubtitleTitle) -> Result<Self, Self::Error> {
        let id = match &value.title {
            SubtitleTitle_title::Quote(x) => x,
            SubtitleTitle_title::RawTitle(x) => x,
        };
        Ok(StructuralReferenceParent::SubtitleTitle(id.clone()))
    }
}

impl TryFrom<&ArticleRelativePosition> for StructuralReferenceElement {
    type Error = anyhow::Error;

    fn try_from(value: &ArticleRelativePosition) -> Result<Self, Self::Error> {
        Ok(match value {
            ArticleRelativePosition::AfterArticle(x) => {
                StructuralReferenceElement::SubtitleAfterArticle(x.try_into()?)
            }
            ArticleRelativePosition::BeforeArticle(x) => {
                StructuralReferenceElement::SubtitleBeforeArticle(x.try_into()?)
            }
        })
    }
}

impl TryFrom<&BookReference> for NumericIdentifier {
    type Error = anyhow::Error;

    fn try_from(value: &BookReference) -> Result<Self, Self::Error> {
        StructuralElementType::Book.parse_identifier(&value.id)
    }
}
