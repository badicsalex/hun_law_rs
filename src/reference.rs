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
use from_variants::FromVariants;
use serde::{Deserialize, Serialize};

use crate::structure::{
    ActIdentifier, AlphabeticIdentifier, ArticleIdentifier, NumericIdentifier,
    PrefixedAlphabeticIdentifier,
};
use crate::util::is_default;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "IdentifierRangeSerdeHelper<T>")]
#[serde(into = "IdentifierRangeSerdeHelper<T>")]
pub struct IdentifierRange<T: Clone + Eq> {
    pub start: T,
    pub end: T,
}

// I tried manually implementing Serialize and Deserialize for IdentifierRange,
// But it was some 200 lines of very error-prone code. This little trick is
// too cute for my taste, but it had to be done.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum IdentifierRangeSerdeHelper<T> {
    Single(T),
    Range { start: T, end: T },
}

impl<T: Clone + Eq> From<IdentifierRangeSerdeHelper<T>> for IdentifierRange<T> {
    fn from(helper: IdentifierRangeSerdeHelper<T>) -> Self {
        match helper {
            IdentifierRangeSerdeHelper::Single(val) => Self {
                start: val.clone(),
                end: val,
            },
            IdentifierRangeSerdeHelper::Range { start, end } => Self { start, end },
        }
    }
}
impl<T: Clone + Eq> From<IdentifierRange<T>> for IdentifierRangeSerdeHelper<T> {
    fn from(val: IdentifierRange<T>) -> Self {
        if val.start == val.end {
            Self::Single(val.start)
        } else {
            Self::Range {
                start: val.start,
                end: val.end,
            }
        }
    }
}

pub type RefPartArticle = IdentifierRange<ArticleIdentifier>;
pub type RefPartParagraph = IdentifierRange<NumericIdentifier>;

#[derive(Debug, Clone, PartialEq, Eq, FromVariants, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RefPartPoint {
    Numeric(IdentifierRange<NumericIdentifier>),
    Alphabetic(IdentifierRange<AlphabeticIdentifier>),
}

#[derive(Debug, Clone, PartialEq, Eq, FromVariants, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RefPartSubpoint {
    Numeric(IdentifierRange<NumericIdentifier>),
    Alphabetic(IdentifierRange<PrefixedAlphabeticIdentifier>),
}

pub trait RefPartFrom<T: Clone>: Sized {
    fn from_single(id: T) -> Self {
        Self::from_range(id.clone(), id)
    }

    fn from_range(start: T, end: T) -> Self;
}

impl RefPartFrom<ArticleIdentifier> for RefPartArticle {
    fn from_range(start: ArticleIdentifier, end: ArticleIdentifier) -> Self {
        Self { start, end }
    }
}
impl RefPartFrom<NumericIdentifier> for RefPartParagraph {
    fn from_range(start: NumericIdentifier, end: NumericIdentifier) -> Self {
        Self { start, end }
    }
}
impl RefPartFrom<NumericIdentifier> for RefPartPoint {
    fn from_range(start: NumericIdentifier, end: NumericIdentifier) -> Self {
        Self::Numeric(IdentifierRange { start, end })
    }
}
impl RefPartFrom<AlphabeticIdentifier> for RefPartPoint {
    fn from_range(start: AlphabeticIdentifier, end: AlphabeticIdentifier) -> Self {
        Self::Alphabetic(IdentifierRange { start, end })
    }
}
impl RefPartFrom<NumericIdentifier> for RefPartSubpoint {
    fn from_range(start: NumericIdentifier, end: NumericIdentifier) -> Self {
        Self::Numeric(IdentifierRange { start, end })
    }
}
impl RefPartFrom<PrefixedAlphabeticIdentifier> for RefPartSubpoint {
    fn from_range(start: PrefixedAlphabeticIdentifier, end: PrefixedAlphabeticIdentifier) -> Self {
        Self::Alphabetic(IdentifierRange { start, end })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reference {
    #[serde(default, skip_serializing_if = "is_default")]
    act: Option<ActIdentifier>,
    #[serde(default, skip_serializing_if = "is_default")]
    article: Option<RefPartArticle>,
    #[serde(default, skip_serializing_if = "is_default")]
    paragraph: Option<RefPartParagraph>,
    #[serde(default, skip_serializing_if = "is_default")]
    point: Option<RefPartPoint>,
    #[serde(default, skip_serializing_if = "is_default")]
    subpoint: Option<RefPartSubpoint>,
}

impl Reference {}

#[derive(Debug, Clone)]
pub struct ReferenceBuilder {
    r: Reference,
}

impl ReferenceBuilder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            r: Reference {
                act: None,
                article: None,
                paragraph: None,
                point: None,
                subpoint: None,
            },
        }
    }

    pub fn build(&self) -> Result<Reference> {
        // TODO: check the built reference
        Ok(self.r.clone())
    }

    pub fn reset_article(&mut self) -> &mut Self {
        self.r.article = None;
        self.reset_paragraph()
    }
    pub fn reset_paragraph(&mut self) -> &mut Self {
        self.r.paragraph = None;
        self.reset_point()
    }
    pub fn reset_point(&mut self) -> &mut Self {
        self.r.point = None;
        self.reset_subpoint()
    }
    pub fn reset_subpoint(&mut self) -> &mut Self {
        self.r.subpoint = None;
        self
    }
}

pub trait ReferenceBuilderSetPart<T> {
    fn set_part(&mut self, val: T) -> &mut Self;
}

impl ReferenceBuilderSetPart<ActIdentifier> for ReferenceBuilder {
    fn set_part(&mut self, val: ActIdentifier) -> &mut Self {
        self.r.act = Some(val);
        self.reset_article()
    }
}

impl ReferenceBuilderSetPart<RefPartArticle> for ReferenceBuilder {
    fn set_part(&mut self, val: RefPartArticle) -> &mut Self {
        self.r.article = Some(val);
        self.reset_paragraph()
    }
}

impl ReferenceBuilderSetPart<RefPartParagraph> for ReferenceBuilder {
    fn set_part(&mut self, val: RefPartParagraph) -> &mut Self {
        self.r.paragraph = Some(val);
        self.reset_point()
    }
}

impl ReferenceBuilderSetPart<RefPartPoint> for ReferenceBuilder {
    fn set_part(&mut self, val: RefPartPoint) -> &mut Self {
        self.r.point = Some(val);
        self.reset_subpoint()
    }
}

impl ReferenceBuilderSetPart<RefPartSubpoint> for ReferenceBuilder {
    fn set_part(&mut self, val: RefPartSubpoint) -> &mut Self {
        self.r.subpoint = Some(val);
        self
    }
}
