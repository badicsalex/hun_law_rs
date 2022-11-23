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

use serde::{Deserialize, Serialize};

use crate::identifier::{
    range::IdentifierRange, ActIdentifier, ArticleIdentifier, NumericIdentifier,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StructuralReference {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub act: Option<ActIdentifier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub book: Option<NumericIdentifier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<StructuralReferenceParent>,
    pub structural_element: StructuralReferenceElement,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub title_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StructuralReferenceElement {
    Part(NumericIdentifier),
    Title(NumericIdentifier),
    Chapter(NumericIdentifier),
    SubtitleId(NumericIdentifier),
    SubtitleRange(IdentifierRange<NumericIdentifier>),
    SubtitleTitle(String),
    SubtitleAfterArticle(ArticleIdentifier),
    /// Only the subtitle before the article (for amendment purposes)
    SubtitleBeforeArticle(ArticleIdentifier),
    /// Both the subtitle and the article that follows (for amendments)
    SubtitleBeforeArticleInclusive(ArticleIdentifier),
    /// All we know is that it's a subtitle, and its position is defined by
    /// the parent. Should only be used with pure insertions.
    SubtitleUnknown,
    Article(IdentifierRange<ArticleIdentifier>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StructuralReferenceParent {
    Part(NumericIdentifier),
    Title(NumericIdentifier),
    Chapter(NumericIdentifier),
    SubtitleId(NumericIdentifier),
    SubtitleRange(IdentifierRange<NumericIdentifier>),
    SubtitleTitle(String),
}

impl From<StructuralReferenceParent> for StructuralReferenceElement {
    fn from(srp: StructuralReferenceParent) -> Self {
        match srp {
            StructuralReferenceParent::Part(x) => StructuralReferenceElement::Part(x),
            StructuralReferenceParent::Title(x) => StructuralReferenceElement::Title(x),
            StructuralReferenceParent::Chapter(x) => StructuralReferenceElement::Chapter(x),
            StructuralReferenceParent::SubtitleId(x) => StructuralReferenceElement::SubtitleId(x),
            StructuralReferenceParent::SubtitleRange(x) => {
                StructuralReferenceElement::SubtitleRange(x)
            }
            StructuralReferenceParent::SubtitleTitle(x) => {
                StructuralReferenceElement::SubtitleTitle(x)
            }
        }
    }
}
