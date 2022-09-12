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

use from_variants::FromVariants;
use serde::{Deserialize, Serialize};

use crate::identifier::{
    ActIdentifier, AlphabeticIdentifier, ArticleIdentifier, NumericIdentifier,
    PrefixedAlphabeticIdentifier,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(from = "IdentifierRangeSerdeHelper<T>")]
#[serde(into = "IdentifierRangeSerdeHelper<T>")]
pub struct IdentifierRange<T: Copy + Eq> {
    pub(super) start: T,
    pub(super) end: T,
}

impl<T: Copy + Eq> IdentifierRange<T> {
    pub fn is_range(&self) -> bool {
        self.start != self.end
    }

    pub fn first_in_range(&self) -> T {
        self.start
    }

    pub fn last_in_range(&self) -> T {
        self.end
    }
}

impl<T: Ord + Copy> IdentifierRange<T> {
    pub fn contains(&self, id: T) -> bool {
        self.start >= id && self.end <= id
    }
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

impl<T: Copy + Eq> From<IdentifierRangeSerdeHelper<T>> for IdentifierRange<T> {
    fn from(helper: IdentifierRangeSerdeHelper<T>) -> Self {
        match helper {
            IdentifierRangeSerdeHelper::Single(val) => Self {
                start: val,
                end: val,
            },
            IdentifierRangeSerdeHelper::Range { start, end } => Self { start, end },
        }
    }
}
impl<T: Copy + Eq> From<IdentifierRange<T>> for IdentifierRangeSerdeHelper<T> {
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

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromVariants, Serialize, Deserialize,
)]
#[serde(untagged)]
pub enum RefPartPoint {
    Numeric(IdentifierRange<NumericIdentifier>),
    Alphabetic(IdentifierRange<AlphabeticIdentifier>),
}

impl RefPartPoint {
    pub fn is_range(&self) -> bool {
        match self {
            RefPartPoint::Numeric(x) => x.is_range(),
            RefPartPoint::Alphabetic(x) => x.is_range(),
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromVariants, Serialize, Deserialize,
)]
#[serde(untagged)]
pub enum RefPartSubpoint {
    Numeric(IdentifierRange<NumericIdentifier>),
    Alphabetic(IdentifierRange<PrefixedAlphabeticIdentifier>),
}

impl RefPartSubpoint {
    pub fn is_range(&self) -> bool {
        match self {
            RefPartSubpoint::Numeric(x) => x.is_range(),
            RefPartSubpoint::Alphabetic(x) => x.is_range(),
        }
    }
}

pub trait RefPartFrom<T: Copy>: Sized {
    fn from_single(id: T) -> Self {
        Self::from_range(id, id)
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

#[derive(Debug, Clone, PartialEq, Eq, FromVariants)]
pub enum AnyReferencePart {
    Empty,
    Act(ActIdentifier),
    Article(RefPartArticle),
    Paragraph(RefPartParagraph),
    Point(RefPartPoint),
    Subpoint(RefPartSubpoint),
}
