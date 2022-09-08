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
use std::{fmt::Debug, str::FromStr};

use pretty_assertions::assert_eq;

use crate::{
    identifier::{AlphabeticIdentifier, NumericIdentifier, PrefixedAlphabeticIdentifier},
    reference::parts::IdentifierRange,
};

use super::*;

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
            article: Some(RefPartArticle {
                start: "2".parse().unwrap(),
                end: "2".parse().unwrap()
            }),
            ..Default::default()
        } > Reference {
            article: Some(RefPartArticle {
                start: "1".parse().unwrap(),
                end: "1".parse().unwrap()
            }),
            point: Some(RefPartPoint::Alphabetic(IdentifierRange {
                start: "a".parse().unwrap(),
                end: "x".parse().unwrap()
            })),
            ..Default::default()
        }
    );
    assert!(
        Reference {
            act: Some(ActIdentifier {
                year: 2000,
                number: 1
            }),
            ..Default::default()
        } > Reference {
            article: Some(RefPartArticle {
                start: "1".parse().unwrap(),
                end: "1".parse().unwrap()
            }),
            point: Some(RefPartPoint::Alphabetic(IdentifierRange {
                start: "a".parse().unwrap(),
                end: "x".parse().unwrap()
            })),
            ..Default::default()
        }
    );
    assert!(
        Reference {
            article: Some(RefPartArticle {
                start: "1".parse().unwrap(),
                end: "1".parse().unwrap()
            }),
            point: Some(RefPartPoint::Alphabetic(IdentifierRange {
                start: "a".parse().unwrap(),
                end: "a".parse().unwrap()
            })),
            subpoint: Some(RefPartSubpoint::Numeric(IdentifierRange {
                start: "1".parse().unwrap(),
                end: "2".parse().unwrap()
            })),
            ..Default::default()
        } < Reference {
            article: Some(RefPartArticle {
                start: "1".parse().unwrap(),
                end: "1".parse().unwrap()
            }),
            point: Some(RefPartPoint::Alphabetic(IdentifierRange {
                start: "a".parse().unwrap(),
                end: "a".parse().unwrap()
            })),
            subpoint: Some(RefPartSubpoint::Numeric(IdentifierRange {
                start: "3".parse().unwrap(),
                end: "4".parse().unwrap()
            })),
            ..Default::default()
        }
    );
}

#[test]
fn test_contains() {
    fn convert_one<TR, TI>(s: &str) -> Option<TR>
    where
        TR: RefPartFrom<TI>,
        TI: Clone + FromStr,
        <TI as FromStr>::Err: Debug,
    {
        if s.is_empty() {
            None
        } else if let Some((start, end)) = s.split_once('-') {
            Some(TR::from_range(start.parse().unwrap(), end.parse().unwrap()))
        } else {
            Some(TR::from_single(s.parse().unwrap()))
        }
    }
    fn easy_contains(
        article_outer: &str,
        paragraph_outer: &str,
        article_inner: &str,
        paragraph_inner: &str,
    ) -> bool {
        let ref_outer = Reference {
            article: convert_one(article_outer),
            paragraph: convert_one(paragraph_outer),
            ..Default::default()
        };
        let ref_inner = Reference {
            article: convert_one(article_inner),
            paragraph: convert_one(paragraph_inner),
            ..Default::default()
        };
        println!("Outer: {:?}, inner: {:?}", ref_outer, ref_inner);
        ref_outer.contains(&ref_inner)
    }

    assert!(easy_contains("1", "", "1", ""));
    assert!(!easy_contains("1", "", "2", ""));

    assert!(!easy_contains("2-4", "", "1", ""));
    assert!(easy_contains("2-4", "", "2", ""));
    assert!(easy_contains("2-4", "", "3", ""));
    assert!(easy_contains("2-4", "", "4", ""));
    assert!(!easy_contains("2-4", "", "5", ""));

    assert!(!easy_contains("2-4", "", "1-3", ""));
    assert!(easy_contains("2-4", "", "2-3", ""));
    assert!(easy_contains("2-4", "", "2-4", ""));
    assert!(easy_contains("2-4", "", "3-4", ""));
    assert!(!easy_contains("2-4", "", "4-5", ""));
    assert!(!easy_contains("2-4", "", "14-15", ""));

    assert!(easy_contains("1", "", "1", "2"));
    assert!(!easy_contains("1", "1", "1", "2"));

    assert!(easy_contains("1", "", "1", "2-15"));
    assert!(easy_contains("1-2", "", "1", "2-15"));
    assert!(easy_contains("1-2", "", "2", "2"));
    assert!(easy_contains("1-3", "", "2", "2"));

    assert!(!easy_contains("1", "1", "1", ""));
    assert!(!easy_contains("1", "1", "2", ""));

    // Very pathological cases
    assert!(easy_contains("", "", "1", "")); // Empty reference "contains" everything.
    assert!(!easy_contains("1", "", "", ""));
    assert!(!easy_contains("1", "", "", "1"));

    // Different acts
    let ref_outer = Reference {
        act: Some(ActIdentifier {
            year: 2012,
            number: 1,
        }),
        article: convert_one("1"),
        paragraph: None,
        point: None,
        subpoint: None,
    };
    let ref_inner = Reference {
        act: Some(ActIdentifier {
            year: 2012,
            number: 2,
        }),
        article: convert_one("1"),
        paragraph: None,
        point: None,
        subpoint: None,
    };
    assert!(!ref_outer.contains(&ref_inner));

    // Points
    let ref_outer = Reference {
        act: Some(ActIdentifier {
            year: 2012,
            number: 1,
        }),
        article: convert_one("1"),
        paragraph: convert_one("1"),
        point: convert_one::<RefPartPoint, NumericIdentifier>("1-4"),
        subpoint: None,
    };
    let ref_inner = Reference {
        act: Some(ActIdentifier {
            year: 2012,
            number: 1,
        }),
        article: convert_one("1"),
        paragraph: convert_one("1"),
        point: convert_one::<RefPartPoint, NumericIdentifier>("2-3"),
        subpoint: None,
    };
    assert!(ref_outer.contains(&ref_outer));
    assert!(ref_outer.contains(&ref_inner));
    assert!(!ref_inner.contains(&ref_outer));

    // Subpoints
    let ref_outer = Reference {
        act: Some(ActIdentifier {
            year: 2012,
            number: 1,
        }),
        article: convert_one("1"),
        paragraph: convert_one("1"),
        point: convert_one::<RefPartPoint, NumericIdentifier>("1"),
        subpoint: convert_one::<RefPartSubpoint, NumericIdentifier>("1-4"),
    };
    let ref_inner = Reference {
        act: Some(ActIdentifier {
            year: 2012,
            number: 1,
        }),
        article: convert_one("1"),
        paragraph: convert_one("1"),
        point: convert_one::<RefPartPoint, NumericIdentifier>("1"),
        subpoint: convert_one::<RefPartSubpoint, NumericIdentifier>("2-3"),
    };
    assert!(ref_outer.contains(&ref_outer));
    assert!(ref_outer.contains(&ref_inner));
    assert!(!ref_inner.contains(&ref_outer));
}
