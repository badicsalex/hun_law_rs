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
    range::{IdentifierRange, IdentifierRangeFrom},
    ActIdentifier, AlphabeticIdentifier, ArticleIdentifier, NumericIdentifier,
    PrefixedAlphabeticIdentifier,
};

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

impl IdentifierRangeFrom<NumericIdentifier> for RefPartPoint {
    fn from_range(start: NumericIdentifier, end: NumericIdentifier) -> Self {
        Self::Numeric(IdentifierRange::from_range(start, end))
    }
}
impl IdentifierRangeFrom<AlphabeticIdentifier> for RefPartPoint {
    fn from_range(start: AlphabeticIdentifier, end: AlphabeticIdentifier) -> Self {
        Self::Alphabetic(IdentifierRange::from_range(start, end))
    }
}
impl IdentifierRangeFrom<NumericIdentifier> for RefPartSubpoint {
    fn from_range(start: NumericIdentifier, end: NumericIdentifier) -> Self {
        Self::Numeric(IdentifierRange::from_range(start, end))
    }
}
impl IdentifierRangeFrom<PrefixedAlphabeticIdentifier> for RefPartSubpoint {
    fn from_range(start: PrefixedAlphabeticIdentifier, end: PrefixedAlphabeticIdentifier) -> Self {
        Self::Alphabetic(IdentifierRange::from_range(start, end))
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

impl AnyReferencePart {
    pub fn act(&self) -> Option<ActIdentifier> {
        if let Self::Act(result) = self {
            Some(*result)
        } else {
            None
        }
    }
    pub fn article(&self) -> Option<IdentifierRange<ArticleIdentifier>> {
        if let Self::Article(result) = self {
            Some(*result)
        } else {
            None
        }
    }
    pub fn paragraph(&self) -> Option<IdentifierRange<Option<NumericIdentifier>>> {
        if let Self::Paragraph(result) = self {
            Some(IdentifierRange::from_range(
                Some(result.first_in_range()),
                Some(result.last_in_range()),
            ))
        } else {
            None
        }
    }
    pub fn numeric_point(&self) -> Option<IdentifierRange<NumericIdentifier>> {
        if let Self::Point(RefPartPoint::Numeric(result)) = self {
            Some(*result)
        } else {
            None
        }
    }
    pub fn alphabetic_point(&self) -> Option<IdentifierRange<AlphabeticIdentifier>> {
        if let Self::Point(RefPartPoint::Alphabetic(result)) = self {
            Some(*result)
        } else {
            None
        }
    }
    pub fn numeric_subpoint(&self) -> Option<IdentifierRange<NumericIdentifier>> {
        if let Self::Subpoint(RefPartSubpoint::Numeric(result)) = self {
            Some(*result)
        } else {
            None
        }
    }
    pub fn alphabetic_subpoint(&self) -> Option<IdentifierRange<PrefixedAlphabeticIdentifier>> {
        if let Self::Subpoint(RefPartSubpoint::Alphabetic(result)) = self {
            Some(*result)
        } else {
            None
        }
    }
}
