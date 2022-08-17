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

use anyhow::{anyhow, bail, ensure, Result};
use from_variants::FromVariants;
use serde::{Deserialize, Serialize};

use crate::identifier::{
    ActIdentifier, AlphabeticIdentifier, ArticleIdentifier, NumericIdentifier,
    PrefixedAlphabeticIdentifier,
};
use crate::util::is_default;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(from = "IdentifierRangeSerdeHelper<T>")]
#[serde(into = "IdentifierRangeSerdeHelper<T>")]
pub struct IdentifierRange<T: Clone + Eq> {
    start: T,
    end: T,
}

impl<T: Clone + Eq> IdentifierRange<T> {
    pub fn is_range(&self) -> bool {
        self.start != self.end
    }

    pub fn first_in_range(&self) -> T {
        self.start.clone()
    }

    pub fn last_in_range(&self) -> T {
        self.end.clone()
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, FromVariants, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, FromVariants, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "UncheckedReference")]
#[serde(into = "UncheckedReference")]
pub struct Reference {
    act: Option<ActIdentifier>,
    article: Option<RefPartArticle>,
    paragraph: Option<RefPartParagraph>,
    point: Option<RefPartPoint>,
    subpoint: Option<RefPartSubpoint>,
}

#[derive(Debug, Clone, PartialEq, Eq, FromVariants)]
pub enum AnyReferencePart {
    Act(ActIdentifier),
    Article(RefPartArticle),
    Paragraph(RefPartParagraph),
    Point(RefPartPoint),
    Subpoint(RefPartSubpoint),
}

impl Reference {
    pub fn get_last_part(&self) -> Option<AnyReferencePart> {
        self.subpoint
            .clone()
            .map(|x| x.into())
            .or_else(|| self.point.clone().map(|x| x.into()))
            .or_else(|| self.paragraph.clone().map(|x| x.into()))
            .or_else(|| self.article.clone().map(|x| x.into()))
            .or_else(|| self.act.map(|x| x.into()))
    }

    pub fn is_act_only(&self) -> bool {
        self.article.is_none()
    }

    pub fn first_in_range(&self) -> Self {
        Self {
            act: self.act,
            article: self
                .article
                .clone()
                .map(|x| RefPartFrom::from_single(x.first_in_range())),
            paragraph: self
                .paragraph
                .clone()
                .map(|x| RefPartFrom::from_single(x.first_in_range())),
            point: self.point.clone().map(|x| match x {
                RefPartPoint::Numeric(n) => RefPartFrom::from_single(n.first_in_range()),
                RefPartPoint::Alphabetic(a) => RefPartFrom::from_single(a.first_in_range()),
            }),
            subpoint: self.subpoint.clone().map(|x| match x {
                RefPartSubpoint::Numeric(n) => RefPartFrom::from_single(n.first_in_range()),
                RefPartSubpoint::Alphabetic(a) => RefPartFrom::from_single(a.first_in_range()),
            }),
        }
    }

    pub fn last_in_range(&self) -> Self {
        Self {
            act: self.act,
            article: self
                .article
                .clone()
                .map(|x| RefPartFrom::from_single(x.last_in_range())),
            paragraph: self
                .paragraph
                .clone()
                .map(|x| RefPartFrom::from_single(x.last_in_range())),
            point: self.point.clone().map(|x| match x {
                RefPartPoint::Numeric(n) => RefPartFrom::from_single(n.last_in_range()),
                RefPartPoint::Alphabetic(a) => RefPartFrom::from_single(a.last_in_range()),
            }),
            subpoint: self.subpoint.clone().map(|x| match x {
                RefPartSubpoint::Numeric(n) => RefPartFrom::from_single(n.last_in_range()),
                RefPartSubpoint::Alphabetic(a) => RefPartFrom::from_single(a.last_in_range()),
            }),
        }
    }

    pub fn make_range(start: &Self, end: &Self) -> Result<Self> {
        let mut builder = ReferenceBuilder::new();
        ensure!(
            start.act == end.act,
            "Reference ranges between acts are not allowed"
        );
        if let Some(act) = start.act {
            builder.set_part(act);
        }

        // --- article ---
        if start.article != end.article {
            ensure!(
                start.paragraph.is_none()
                    && end.paragraph.is_none()
                    && start.point.is_none()
                    && end.point.is_none()
                    && start.subpoint.is_none()
                    && end.subpoint.is_none(),
                "Trying to create a ref range where not only the last component differs (article)"
            );
            if let (Some(start_article), Some(end_article)) = (&start.article, &end.article) {
                builder.set_part(RefPartArticle::from_range(
                    start_article.first_in_range(),
                    end_article.last_in_range(),
                ));
                return builder.build();
            }
            bail!("Trying to create a ref range between different levels (article)")
        }

        if let Some(article) = &start.article {
            builder.set_part(article.clone());
        }

        // --- paragraph ---
        if start.paragraph != end.paragraph {
            ensure!(
                start.point.is_none()
                    && end.point.is_none()
                    && start.subpoint.is_none()
                    && end.subpoint.is_none(),
                "Trying to create a ref range where not only the last component differs (paragraph)"
            );

            if let (Some(start_paragraph), Some(end_paragraph)) = (&start.paragraph, &end.paragraph)
            {
                builder.set_part(RefPartParagraph::from_range(
                    start_paragraph.first_in_range(),
                    end_paragraph.last_in_range(),
                ));
                return builder.build();
            }
            bail!("Trying to create a ref range between different levels (paragraph)")
        }

        if let Some(paragraph) = &start.paragraph {
            builder.set_part(paragraph.clone());
        }

        // --- point ---
        if start.point != end.point {
            ensure!(
                start.subpoint.is_none() && end.subpoint.is_none(),
                "Trying to create a ref range where not only the last component differs (point)"
            );
            if let (Some(start_point), Some(end_point)) = (&start.point, &end.point) {
                match (start_point, end_point) {
                    (RefPartPoint::Numeric(sp), RefPartPoint::Numeric(ep)) => builder.set_part(
                        RefPartPoint::from_range(sp.first_in_range(), ep.last_in_range()),
                    ),
                    (RefPartPoint::Alphabetic(sp), RefPartPoint::Alphabetic(ep)) => builder
                        .set_part(RefPartPoint::from_range(
                            sp.first_in_range(),
                            ep.last_in_range(),
                        )),
                    _ => bail!("Point id types are different when creating a range."),
                };
                return builder.build();
            }
            bail!("Trying to create a ref range between different levels (point)")
        }

        if let Some(point) = &start.point {
            builder.set_part(point.clone());
        }

        // --- subpoint ---
        if start.subpoint != end.subpoint {
            if let (Some(start_subpoint), Some(end_subpoint)) = (&start.subpoint, &end.subpoint) {
                match (start_subpoint, end_subpoint) {
                    (RefPartSubpoint::Numeric(sp), RefPartSubpoint::Numeric(ep)) => builder
                        .set_part(RefPartSubpoint::from_range(
                            sp.first_in_range(),
                            ep.last_in_range(),
                        )),
                    (RefPartSubpoint::Alphabetic(sp), RefPartSubpoint::Alphabetic(ep)) => builder
                        .set_part(RefPartSubpoint::from_range(
                            sp.first_in_range(),
                            ep.last_in_range(),
                        )),
                    _ => bail!("subpoint id types are different when creating a range."),
                };
                return builder.build();
            }
            bail!("Trying to create a ref range between different levels (subpoint)")
        }

        if let Some(subpoint) = &start.subpoint {
            builder.set_part(subpoint.clone());
        }
        builder.build()
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
            if article.is_range() {
                ensure!(
                    self.paragraph.is_none() && self.point.is_none() && self.subpoint.is_none(),
                    "Reference parts found after article range"
                );
            }
        }
        if let Some(paragraph) = &self.paragraph {
            if paragraph.is_range() {
                ensure!(
                    self.point.is_none() && self.subpoint.is_none(),
                    "Reference parts found after paragraph range"
                );
            }
        }
        if let Some(point) = &self.point {
            if point.is_range() {
                ensure!(
                    self.subpoint.is_none(),
                    "Reference parts found after point range"
                );
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_builder_happy_cases() {
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
    fn test_builder_unhappy_cases() {
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

    #[test]
    fn test_make_range() {
        let mut builder = ReferenceBuilder::new();
        builder.set_part(ActIdentifier {
            year: 2001,
            number: 420,
        });
        let ref_act = builder.build().unwrap();

        builder.set_part(RefPartArticle::from_range(
            "4:20".parse().unwrap(),
            "4:80".parse().unwrap(),
        ));
        let ref_articler = builder.build().unwrap();
        builder.set_part(RefPartArticle::from_single("4:20".parse().unwrap()));
        let ref_article1 = builder.build().unwrap();
        builder.set_part(RefPartArticle::from_single("4:80".parse().unwrap()));
        let ref_article2 = builder.build().unwrap();

        builder.set_part(RefPartParagraph::from_range(
            "20".parse().unwrap(),
            "80".parse().unwrap(),
        ));
        let ref_paragraphr = builder.build().unwrap();
        builder.set_part(RefPartParagraph::from_single("20".parse().unwrap()));
        let ref_paragraph1 = builder.build().unwrap();
        builder.set_part(RefPartParagraph::from_single("80".parse().unwrap()));
        let ref_paragraph2 = builder.build().unwrap();

        builder.set_part(RefPartPoint::from_range(
            NumericIdentifier::from_str("20").unwrap(),
            NumericIdentifier::from_str("22").unwrap(),
        ));
        let ref_point_numr = builder.build().unwrap();
        builder.set_part(RefPartPoint::from_single(
            NumericIdentifier::from_str("20").unwrap(),
        ));
        let ref_point_num1 = builder.build().unwrap();
        builder.set_part(RefPartPoint::from_single(
            NumericIdentifier::from_str("22").unwrap(),
        ));
        let ref_point_num2 = builder.build().unwrap();

        builder.set_part(RefPartPoint::from_range(
            AlphabeticIdentifier::from_str("a").unwrap(),
            AlphabeticIdentifier::from_str("b").unwrap(),
        ));
        let ref_point_alphar = builder.build().unwrap();
        builder.set_part(RefPartPoint::from_single(
            AlphabeticIdentifier::from_str("a").unwrap(),
        ));
        let ref_point_alpha1 = builder.build().unwrap();
        builder.set_part(RefPartPoint::from_single(
            AlphabeticIdentifier::from_str("b").unwrap(),
        ));
        let ref_point_alpha2 = builder.build().unwrap();

        builder.set_part(RefPartSubpoint::from_range(
            NumericIdentifier::from_str("20").unwrap(),
            NumericIdentifier::from_str("22").unwrap(),
        ));
        let ref_subpoint_numr = builder.build().unwrap();
        builder.set_part(RefPartSubpoint::from_single(
            NumericIdentifier::from_str("20").unwrap(),
        ));
        let ref_subpoint_num1 = builder.build().unwrap();
        builder.set_part(RefPartSubpoint::from_single(
            NumericIdentifier::from_str("22").unwrap(),
        ));
        let ref_subpoint_num2 = builder.build().unwrap();

        builder.set_part(RefPartSubpoint::from_range(
            PrefixedAlphabeticIdentifier::from_str("ba").unwrap(),
            PrefixedAlphabeticIdentifier::from_str("bc").unwrap(),
        ));
        let ref_subpoint_alphar = builder.build().unwrap();
        builder.set_part(RefPartSubpoint::from_single(
            PrefixedAlphabeticIdentifier::from_str("ba").unwrap(),
        ));
        let ref_subpoint_alpha1 = builder.build().unwrap();
        builder.set_part(RefPartSubpoint::from_single(
            PrefixedAlphabeticIdentifier::from_str("bc").unwrap(),
        ));
        let ref_subpoint_alpha2 = builder.build().unwrap();

        // --- Idempotence ---
        assert_eq!(Reference::make_range(&ref_act, &ref_act).unwrap(), ref_act);
        assert_eq!(
            Reference::make_range(&ref_article1, &ref_article1).unwrap(),
            ref_article1
        );
        assert_eq!(
            Reference::make_range(&ref_paragraph1, &ref_paragraph1).unwrap(),
            ref_paragraph1
        );
        assert_eq!(
            Reference::make_range(&ref_point_num1, &ref_point_num1).unwrap(),
            ref_point_num1
        );
        assert_eq!(
            Reference::make_range(&ref_point_alpha1, &ref_point_alpha1).unwrap(),
            ref_point_alpha1
        );
        assert_eq!(
            Reference::make_range(&ref_subpoint_num1, &ref_subpoint_num1).unwrap(),
            ref_subpoint_num1
        );
        assert_eq!(
            Reference::make_range(&ref_subpoint_alpha1, &ref_subpoint_alpha1).unwrap(),
            ref_subpoint_alpha1
        );

        // --- Idempotence: range edition ---
        assert_eq!(Reference::make_range(&ref_act, &ref_act).unwrap(), ref_act);
        assert_eq!(
            Reference::make_range(&ref_articler, &ref_articler).unwrap(),
            ref_articler
        );
        assert_eq!(
            Reference::make_range(&ref_paragraphr, &ref_paragraphr).unwrap(),
            ref_paragraphr
        );
        assert_eq!(
            Reference::make_range(&ref_point_numr, &ref_point_numr).unwrap(),
            ref_point_numr
        );
        assert_eq!(
            Reference::make_range(&ref_point_alphar, &ref_point_alphar).unwrap(),
            ref_point_alphar
        );
        assert_eq!(
            Reference::make_range(&ref_subpoint_numr, &ref_subpoint_numr).unwrap(),
            ref_subpoint_numr
        );
        assert_eq!(
            Reference::make_range(&ref_subpoint_alphar, &ref_subpoint_alphar).unwrap(),
            ref_subpoint_alphar
        );

        // --- Actual range making ---

        assert_eq!(
            Reference::make_range(&ref_article1, &ref_article2).unwrap(),
            ref_articler
        );
        assert_eq!(
            Reference::make_range(&ref_paragraph1, &ref_paragraph2).unwrap(),
            ref_paragraphr
        );
        assert_eq!(
            Reference::make_range(&ref_point_num1, &ref_point_num2).unwrap(),
            ref_point_numr
        );
        assert_eq!(
            Reference::make_range(&ref_point_alpha1, &ref_point_alpha2).unwrap(),
            ref_point_alphar
        );
        assert_eq!(
            Reference::make_range(&ref_subpoint_num1, &ref_subpoint_num2).unwrap(),
            ref_subpoint_numr
        );
        assert_eq!(
            Reference::make_range(&ref_subpoint_alpha1, &ref_subpoint_alpha2).unwrap(),
            ref_subpoint_alphar
        );

        // --- Some Relative refs ---
        builder = ReferenceBuilder::new();

        builder.set_part(RefPartParagraph::from_single("80".parse().unwrap()));
        builder.set_part(RefPartPoint::from_range(
            AlphabeticIdentifier::from_str("a").unwrap(),
            AlphabeticIdentifier::from_str("b").unwrap(),
        ));
        let relative_1 = builder.build().unwrap();
        builder.set_part(RefPartPoint::from_range(
            AlphabeticIdentifier::from_str("f").unwrap(),
            AlphabeticIdentifier::from_str("g").unwrap(),
        ));
        let relative_2 = builder.build().unwrap();
        builder.set_part(RefPartPoint::from_range(
            AlphabeticIdentifier::from_str("a").unwrap(),
            AlphabeticIdentifier::from_str("g").unwrap(),
        ));
        let relative_expected = builder.build().unwrap();
        assert_eq!(
            Reference::make_range(&relative_1, &relative_2).unwrap(),
            relative_expected
        );

        // --- Some error cases ---

        assert!(Reference::make_range(&ref_article1, &ref_paragraph2).is_err());
        assert!(Reference::make_range(&ref_article2, &ref_paragraph2).is_err());
        assert!(Reference::make_range(&ref_point_num1, &ref_subpoint_num2).is_err());
        assert!(Reference::make_range(&ref_point_num2, &ref_subpoint_num2).is_err());

        assert!(Reference::make_range(&ref_point_num1, &ref_point_alpha1).is_err());
        assert!(Reference::make_range(&ref_subpoint_num1, &ref_subpoint_alpha1).is_err());

        assert!(Reference::make_range(&ref_point_alpha1, &relative_1).is_err());
    }

    #[test]
    fn test_ordering() {
        assert!(
            Reference {
                act: None,
                article: Some(RefPartArticle {
                    start: "2".parse().unwrap(),
                    end: "2".parse().unwrap()
                }),
                paragraph: None,
                point: None,
                subpoint: None,
            } > Reference {
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
                subpoint: None,
            }
        );
        assert!(
            Reference {
                act: Some(ActIdentifier {
                    year: 2000,
                    number: 1
                }),
                article: None,
                paragraph: None,
                point: None,
                subpoint: None,
            } > Reference {
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
                subpoint: None,
            }
        );
        assert!(
            Reference {
                act: None,
                article: Some(RefPartArticle {
                    start: "1".parse().unwrap(),
                    end: "1".parse().unwrap()
                }),
                paragraph: None,
                point: Some(RefPartPoint::Alphabetic(IdentifierRange {
                    start: "a".parse().unwrap(),
                    end: "a".parse().unwrap()
                })),
                subpoint: Some(RefPartSubpoint::Numeric(IdentifierRange {
                    start: "1".parse().unwrap(),
                    end: "2".parse().unwrap()
                }))
            } < Reference {
                act: None,
                article: Some(RefPartArticle {
                    start: "1".parse().unwrap(),
                    end: "1".parse().unwrap()
                }),
                paragraph: None,
                point: Some(RefPartPoint::Alphabetic(IdentifierRange {
                    start: "a".parse().unwrap(),
                    end: "a".parse().unwrap()
                })),
                subpoint: Some(RefPartSubpoint::Numeric(IdentifierRange {
                    start: "3".parse().unwrap(),
                    end: "4".parse().unwrap()
                }))
            }
        );
    }
}
