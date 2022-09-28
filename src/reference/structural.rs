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
    SubtitleTitle(String),
    SubtitleAfterArticle(ArticleIdentifier),
    SubtitleBeforeArticle(ArticleIdentifier),
    SubtitleBeforeArticleInclusive(ArticleIdentifier),
    AtTheEndOfPart(NumericIdentifier),
    AtTheEndOfTitle(NumericIdentifier),
    AtTheEndOfChapter(NumericIdentifier),
    AtTheEndOfAct,
    Article(IdentifierRange<ArticleIdentifier>),
}
