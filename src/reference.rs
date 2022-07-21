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

impl<T: Clone + Eq> IdentifierRange<T> {
    pub fn is_range(&self) -> bool {
        self.start != self.end
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

impl RefPartPoint {
    pub fn is_range(&self) -> bool {
        match self {
            RefPartPoint::Numeric(x) => x.is_range(),
            RefPartPoint::Alphabetic(x) => x.is_range(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, FromVariants, Serialize, Deserialize)]
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

/// Reference to an Act, article or SAE. Possibly relative.
///
/// Guarantees:
/// - There are no 'gaps' in the parts, apart from a potentially missing paragraph
///   (in that case, it means the 'default paragraph' of the article
/// - It might be a range, but the range part is always the last part of the reference
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "UncheckedReference")]
#[serde(into = "UncheckedReference")]
pub struct Reference {
    act: Option<ActIdentifier>,
    article: Option<RefPartArticle>,
    paragraph: Option<RefPartParagraph>,
    point: Option<RefPartPoint>,
    subpoint: Option<RefPartSubpoint>,
}

impl Reference {
    pub fn is_act_only(&self) -> bool {
        self.article.is_none()
    }
}

/// Helper to create Reference isntances from parts.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UncheckedReference {
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

impl UncheckedReference {
    fn check_part_combination(&self) -> Result<()> {
        let filled = [
            self.act.is_some(),
            self.article.is_some(),
            self.paragraph.is_some(),
            self.point.is_some(),
            self.subpoint.is_some(),
        ];
        match filled {
            // Just act ref
            [true, false, false, false, false] => Ok(()),
            // Act with article, maybe point, no subpoint. Paragraph can be missing
            [true, true, _, _, false] => Ok(()),
            // Subpoint ref
            [true, true, _, true, true] => Ok(()),
            // Relative article, maybe point, no subpoint. Paragraph can be missing
            [false, true, _, _, false] => Ok(()),
            // Relative subpoint ref
            [false, true, _, true, true] => Ok(()),
            // Relative paragraph,
            [false, false, true, false, false] => Ok(()),
            // Relative point or subpoint.
            [false, false, _, true, _] => Ok(()),
            // Just subpoint. In this case paragraph is not allowed, as point would be a gap
            [false, false, false, false, true] => Ok(()),
            _ => Err(anyhow!("Invalid reference part combination: {:?}", self)),
        }
    }

    fn check_ranges(&self) -> Result<()> {
        if let Some(article) = &self.article {
            if article.is_range()
                && (self.paragraph.is_some() || self.point.is_some() || self.subpoint.is_some())
            {
                bail!("Reference parts found after article range");
            }
        }
        if let Some(paragraph) = &self.paragraph {
            if paragraph.is_range() && (self.point.is_some() || self.subpoint.is_some()) {
                bail!("Reference parts found after paragraph range");
            }
        }
        if let Some(point) = &self.point {
            if point.is_range() && self.subpoint.is_some() {
                bail!("Reference parts found after point range");
            }
        }
        Ok(())
    }
}

impl From<Reference> for UncheckedReference {
    fn from(r: Reference) -> Self {
        Self {
            act: r.act,
            article: r.article,
            paragraph: r.paragraph,
            point: r.point,
            subpoint: r.subpoint,
        }
    }
}

impl TryFrom<UncheckedReference> for Reference {
    type Error = anyhow::Error;

    fn try_from(r: UncheckedReference) -> Result<Self, Self::Error> {
        r.check_part_combination()?;
        r.check_ranges()?;
        Ok(Self {
            act: r.act,
            article: r.article,
            paragraph: r.paragraph,
            point: r.point,
            subpoint: r.subpoint,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct ReferenceBuilder {
    r: UncheckedReference,
}

impl ReferenceBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn build(&self) -> Result<Reference> {
        self.r.clone().try_into()
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_happy_cases() {
        let mut builder = ReferenceBuilder::new();
        builder.set_part(ActIdentifier {
            year: 2001,
            number: 420,
        });
        builder.set_part(RefPartArticle::from_single("4:20".parse().unwrap()));
        // Test chaining
        builder
            .set_part(RefPartParagraph::from_single("20".parse().unwrap()))
            .set_part(RefPartPoint::from_single(
                NumericIdentifier::from_str("20").unwrap(),
            ))
            .set_part(RefPartSubpoint::from_single(
                PrefixedAlphabeticIdentifier::from_str("sz").unwrap(),
            ));
        let ref1 = builder.build().unwrap();
        assert_eq!(
            ref1,
            Reference {
                act: Some(ActIdentifier {
                    year: 2001,
                    number: 420,
                }),
                article: Some(RefPartArticle {
                    start: "4:20".parse().unwrap(),
                    end: "4:20".parse().unwrap()
                }),
                paragraph: Some(RefPartParagraph {
                    start: "20".parse().unwrap(),
                    end: "20".parse().unwrap()
                }),
                point: Some(RefPartPoint::Numeric(IdentifierRange {
                    start: "20".parse().unwrap(),
                    end: "20".parse().unwrap()
                })),
                subpoint: Some(RefPartSubpoint::Alphabetic(IdentifierRange {
                    start: "sz".parse().unwrap(),
                    end: "sz".parse().unwrap()
                })),
            }
        );
        // Test reseting some of the fields, and also second build.
        builder.set_part(RefPartArticle::from_range(
            "1:10".parse().unwrap(),
            "1:10/C".parse().unwrap(),
        ));
        let ref2 = builder.build().unwrap();
        assert_eq!(
            ref2,
            Reference {
                act: Some(ActIdentifier {
                    year: 2001,
                    number: 420,
                }),
                article: Some(RefPartArticle {
                    start: "1:10".parse().unwrap(),
                    end: "1:10/C".parse().unwrap()
                }),
                paragraph: None,
                point: None,
                subpoint: None
            }
        );
        let ref3 = ReferenceBuilder::new()
            .set_part(RefPartArticle::from_single("1".parse().unwrap()))
            .set_part(RefPartPoint::from_range(
                AlphabeticIdentifier::from_str("a").unwrap(),
                AlphabeticIdentifier::from_str("x").unwrap(),
            ))
            .build()
            .unwrap();
        assert_eq!(
            ref3,
            Reference {
                act: None,
                article: Some(RefPartArticle {
                    start: "1".parse().unwrap(),
                    end: "1".parse().unwrap()
                }),
                paragraph: None,
                point: Some(RefPartPoint::Alphabetic(IdentifierRange {
                    start: "a".parse().unwrap(),
                    end: "x".parse().unwrap()
                })),
                subpoint: None
            }
        );
    }

    #[test]
    fn test_unhappy_cases() {
        assert!(
            ReferenceBuilder::new()
                .set_part(RefPartParagraph::from_single("20".parse().unwrap()))
                .set_part(RefPartSubpoint::from_single(
                    PrefixedAlphabeticIdentifier::from_str("sz").unwrap(),
                ))
                .build()
                .is_err(),
            "'Gaps' in reference parts are not allowed"
        );
        assert!(
            ReferenceBuilder::new()
                .set_part(RefPartParagraph::from_range(
                    "20".parse().unwrap(),
                    "21".parse().unwrap()
                ))
                .set_part(RefPartPoint::from_range(
                    AlphabeticIdentifier::from_str("f").unwrap(),
                    AlphabeticIdentifier::from_str("u").unwrap(),
                ))
                .build()
                .is_err(),
            "Multiple ranges in reference parts are not allowed"
        );
        assert!(
            ReferenceBuilder::new()
                .set_part(RefPartParagraph::from_range(
                    "20".parse().unwrap(),
                    "21".parse().unwrap()
                ))
                .set_part(RefPartPoint::from_single(
                    AlphabeticIdentifier::from_str("f").unwrap(),
                ))
                .set_part(RefPartSubpoint::from_single(
                    PrefixedAlphabeticIdentifier::from_str("fu").unwrap(),
                ))
                .build()
                .is_err(),
            "The range can only be the last part."
        );
    }
}
