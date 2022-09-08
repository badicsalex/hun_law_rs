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

use crate::{
    identifier::{ActIdentifier, ArticleIdentifier, NumericIdentifier},
    util::is_default,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StructuralReference {
    #[serde(default, skip_serializing_if = "is_default")]
    pub act: Option<ActIdentifier>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub book: Option<NumericIdentifier>,
    pub structural_element: StructuralReferenceElement,
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
}
