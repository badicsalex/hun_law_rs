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

use crate::identifier::ActIdentifier;

use super::{
    parts::{RefPartArticle, RefPartParagraph, RefPartPoint, RefPartSubpoint},
    unchecked::UncheckedReference,
    Reference,
};

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

    use pretty_assertions::assert_eq;

    use crate::identifier::{
        range::{IdentifierRange, IdentifierRangeFrom},
        AlphabeticIdentifier, NumericIdentifier, PrefixedAlphabeticIdentifier,
    };

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
                article: Some(RefPartArticle::from_single("4:20".parse().unwrap())),
                paragraph: Some(RefPartParagraph::from_single("20".parse().unwrap())),
                point: Some(RefPartPoint::Numeric(IdentifierRange::from_single(
                    "20".parse().unwrap()
                ))),
                subpoint: Some(RefPartSubpoint::Alphabetic(IdentifierRange::from_single(
                    "sz".parse().unwrap()
                ))),
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
                article: Some(RefPartArticle::from_range(
                    "1:10".parse().unwrap(),
                    "1:10/C".parse().unwrap(),
                )),
                ..Default::default()
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
                article: Some(RefPartArticle::from_single("1".parse().unwrap())),
                point: Some(RefPartPoint::from_range(
                    AlphabeticIdentifier::from_str("a").unwrap(),
                    AlphabeticIdentifier::from_str("x").unwrap(),
                )),
                ..Default::default()
            }
        );
        assert_eq!(
            ReferenceBuilder::new().build().unwrap(),
            Reference::default()
        )
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
}
