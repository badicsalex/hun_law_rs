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

use crate::reference::{StructuralReference, StructuralReferenceElement};
use crate::structure::StructuralElementType;
use hun_law_grammar::*;

impl TryFrom<&AnyStructuralReference> for StructuralReference {
    type Error = anyhow::Error;

    fn try_from(value: &AnyStructuralReference) -> Result<Self, Self::Error> {
        match value {
            AnyStructuralReference::ChapterReference(x) => x.try_into(),
            AnyStructuralReference::PartReference(x) => x.try_into(),
            AnyStructuralReference::SubtitleReference(x) => x.try_into(),
            AnyStructuralReference::SubtitleTitle(x) => x.try_into(),
            AnyStructuralReference::TitleReference(x) => x.try_into(),
        }
    }
}

impl TryFrom<&ChapterReference> for StructuralReference {
    type Error = anyhow::Error;

    fn try_from(value: &ChapterReference) -> Result<Self, Self::Error> {
        Ok(Self {
            act: None,
            book: None,
            structural_element: StructuralReferenceElement::Chapter(
                StructuralElementType::Chapter.parse_identifier(&value.id)?,
            ),
        })
    }
}

impl TryFrom<&PartReference> for StructuralReference {
    type Error = anyhow::Error;

    fn try_from(value: &PartReference) -> Result<Self, Self::Error> {
        Ok(Self {
            act: None,
            book: if let Some(book_id) = &value.book_id {
                Some(StructuralElementType::Book.parse_identifier(book_id)?)
            } else {
                None
            },
            structural_element: StructuralReferenceElement::Part(
                StructuralElementType::Part { is_special: false }.parse_identifier(&value.id)?,
            ),
        })
    }
}

impl TryFrom<&TitleReference> for StructuralReference {
    type Error = anyhow::Error;

    fn try_from(value: &TitleReference) -> Result<Self, Self::Error> {
        Ok(Self {
            act: None,
            book: if let Some(book_id) = &value.book_id {
                Some(StructuralElementType::Book.parse_identifier(book_id)?)
            } else {
                None
            },
            structural_element: StructuralReferenceElement::Title(
                StructuralElementType::Title.parse_identifier(&value.id)?,
            ),
        })
    }
}

impl TryFrom<&TitleInsertionWithBook> for StructuralReference {
    type Error = anyhow::Error;

    fn try_from(value: &TitleInsertionWithBook) -> Result<Self, Self::Error> {
        Ok(Self {
            act: None,
            book: Some(StructuralElementType::Book.parse_identifier(&value.book_id)?),
            structural_element: StructuralReferenceElement::Title(
                StructuralElementType::Title.parse_identifier(&value.id)?,
            ),
        })
    }
}

impl TryFrom<&SubtitleReference> for StructuralReference {
    type Error = anyhow::Error;

    fn try_from(value: &SubtitleReference) -> Result<Self, Self::Error> {
        Ok(Self {
            act: None,
            book: None,
            structural_element: StructuralReferenceElement::SubtitleId(value.id.parse()?),
        })
    }
}

impl TryFrom<&SubtitleTitle> for StructuralReference {
    type Error = anyhow::Error;

    fn try_from(value: &SubtitleTitle) -> Result<Self, Self::Error> {
        let id = match &value.title {
            SubtitleTitle_title::Quote(x) => x,
            SubtitleTitle_title::RawTitle(x) => x,
        };
        Ok(Self {
            act: None,
            book: None,
            structural_element: StructuralReferenceElement::SubtitleTitle(id.clone()),
        })
    }
}
