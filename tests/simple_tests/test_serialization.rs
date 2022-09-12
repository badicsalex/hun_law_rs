// Copyright {C} 2022, Alex Badics
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

use std::str::FromStr;

use chrono::NaiveDate;
use hun_law::{
    identifier::{
        range::IdentifierRangeFrom, ActIdentifier, AlphabeticIdentifier, NumericIdentifier,
        PrefixedAlphabeticIdentifier,
    },
    reference::{
        builder::{ReferenceBuilder, ReferenceBuilderSetPart},
        parts::{RefPartArticle, RefPartParagraph, RefPartPoint, RefPartSubpoint},
        Reference,
    },
    structure::{
        Act, AlphabeticPoint, AlphabeticSubpoint, Article, NumericPoint, Paragraph, SAEBody,
        StructuralElement, StructuralElementType, Subtitle,
    },
    util::singleton_yaml,
};
use pretty_assertions::assert_eq;

fn get_test_act() -> Act {
    Act {
        identifier: ActIdentifier {
            year: 2345,
            number: 0xd,
        },
        publication_date: NaiveDate::from_ymd(2345, 6, 7),
        subject: "A tesztelésről".into(),
        preamble: "A tesztelés nagyon fontos, és egyben kötelező".into(),
        children: vec![
            StructuralElement {
                identifier: "1".parse().unwrap(),
                title: "Egyszerű dolgok".into(),
                element_type: StructuralElementType::Book,
            }
            .into(),
            Subtitle {
                identifier: None,
                title: "Alcim id nelkul".into(),
            }
            .into(),
            Article {
                identifier: "1:1".parse().unwrap(),
                title: Some("Az egyetlen cikk, aminek cime van.".into()),
                children: vec![Paragraph {
                    identifier: None,
                    body: "Meg szövege".into(),
                    semantic_info: Default::default(),
                }],
            }
            .into(),
            Article {
                identifier: "1:2".parse().unwrap(),
                title: None,
                children: vec![
                    Paragraph {
                        identifier: Some(1.into()),
                        body: "Valami valami hosszu szoveg csak azert hogy leteszteljuk hogy a yaml szerializacio mennyire nez ki jol igy. Remelhetoleg nem tori tobb sorba.".into(),
                        semantic_info: Default::default(),
                    },
                    Paragraph {
                        identifier: Some(2.into()),
                        body: SAEBody::Children {
                            intro: "Egy felsorolás legyen".into(),
                            wrap_up: Some("minden esetben.".into()),
                            children: vec![
                                AlphabeticPoint {
                                    identifier: "a".parse().unwrap(),
                                    body: "többelemű".into(),
                                    semantic_info: Default::default(),
                                },
                                AlphabeticPoint {
                                    identifier: "b".parse().unwrap(),
                                    body: SAEBody::Children {
                                        intro: "kellően".into(),
                                        wrap_up: None,
                                        children: vec![
                                            AlphabeticSubpoint {
                                                identifier: "ba".parse().unwrap(),
                                                body: "átláthatatlan".into(),
                                                semantic_info: Default::default(),
                                            },
                                            AlphabeticSubpoint {
                                                identifier: "bb".parse().unwrap(),
                                                body: "komplex".into(),
                                                semantic_info: Default::default(),
                                            },
                                        ]
                                        .into(),
                                    },
                                    semantic_info: Default::default(),
                                },
                            ]
                            .into(),
                        },
                        semantic_info: Default::default(),
                    },
                ],
            }
            .into(),
            StructuralElement {
                identifier: "2".parse().unwrap(),
                title: "Amended stuff in english".into(),
                element_type: StructuralElementType::Book,
            }
            .into(),
            StructuralElement {
                identifier: "1".parse().unwrap(),
                title: "Az eleje".into(),
                element_type: StructuralElementType::Part { is_special: false },
            }
            .into(),
            Subtitle {
                identifier: Some("1".parse().unwrap()),
                title: "Alcim id-vel".into(),
            }
            .into(),
            Article {
                identifier: "2:1".parse().unwrap(),
                title: None,
                children: vec![Paragraph {
                    identifier: None,
                    body: "Nothing fancy yet".into(),
                    semantic_info: Default::default(),
                }],
            }
            .into(),
            StructuralElement {
                identifier: "1/A".parse().unwrap(),
                title: "A hozzaadott".into(),
                element_type: StructuralElementType::Part { is_special: false },
            }
            .into(),
            Subtitle {
                identifier: Some("1/A".parse().unwrap()),
                title: "Alcim amendelt id-vel".into(),
            }
            .into(),
            Article {
                identifier: "2:1/A".parse().unwrap(),
                title: None,
                children: vec![Paragraph {
                    identifier: None,
                    body: "Added after the fact".into(),
                    semantic_info: Default::default(),
                }],
            }
            .into(),
            Article {
                identifier: "2:2".parse().unwrap(),
                title: None,
                children: vec![Paragraph {
                    identifier: Some(1.into()),
                    body: SAEBody::Children {
                        intro: "This can legally be after 2:1/A. Also, ".into(),
                        wrap_up: Some("Can also be amended".into()),
                        children: vec![
                            NumericPoint {
                                identifier: 1.into(),
                                body: "Paragraphs".into(),
                                semantic_info: Default::default(),
                            },
                            NumericPoint {
                                identifier: "1a".parse().unwrap(),
                                body: "Numeric points".into(),
                                semantic_info: Default::default(),
                            },
                            NumericPoint {
                                identifier: 2.into(),
                                body: "Alphabetic points".into(),
                                semantic_info: Default::default(),
                            },
                        ]
                        .into(),
                    },
                    semantic_info: Default::default(),
                }],
            }
            .into(),
        ],
    }
}

const YAML_SERIALIZED_ACT: &str = r#"identifier:
  year: 2345
  number: 13
subject: A tesztelésről
preamble: A tesztelés nagyon fontos, és egyben kötelező
publication_date: 2345-06-07
children:
- StructuralElement:
    identifier: '1'
    title: Egyszerű dolgok
    element_type: Book
- Subtitle:
    title: Alcim id nelkul
- Article:
    identifier: 1:1
    title: Az egyetlen cikk, aminek cime van.
    children:
    - body: Meg szövege
- Article:
    identifier: 1:2
    children:
    - identifier: '1'
      body: Valami valami hosszu szoveg csak azert hogy leteszteljuk hogy a yaml szerializacio mennyire nez ki jol igy. Remelhetoleg nem tori tobb sorba.
    - identifier: '2'
      body:
        intro: Egy felsorolás legyen
        children:
          AlphabeticPoint:
          - identifier: a
            body: többelemű
          - identifier: b
            body:
              intro: kellően
              children:
                AlphabeticSubpoint:
                - identifier: ba
                  body: átláthatatlan
                - identifier: bb
                  body: komplex
        wrap_up: minden esetben.
- StructuralElement:
    identifier: '2'
    title: Amended stuff in english
    element_type: Book
- StructuralElement:
    identifier: '1'
    title: Az eleje
    element_type:
      Part: {}
- Subtitle:
    identifier: '1'
    title: Alcim id-vel
- Article:
    identifier: 2:1
    children:
    - body: Nothing fancy yet
- StructuralElement:
    identifier: 1a
    title: A hozzaadott
    element_type:
      Part: {}
- Subtitle:
    identifier: 1a
    title: Alcim amendelt id-vel
- Article:
    identifier: 2:1/A
    children:
    - body: Added after the fact
- Article:
    identifier: 2:2
    children:
    - identifier: '1'
      body:
        intro: 'This can legally be after 2:1/A. Also, '
        children:
          NumericPoint:
          - identifier: '1'
            body: Paragraphs
          - identifier: 1a
            body: Numeric points
          - identifier: '2'
            body: Alphabetic points
        wrap_up: Can also be amended
"#;

#[test]
fn test_act_yaml_serialization() {
    let act = get_test_act();
    let yaml = singleton_yaml::to_string(&act).unwrap();
    println!("{}", yaml);
    let roundtrip: Act = singleton_yaml::from_str(&yaml).unwrap();
    assert_eq!(act, roundtrip);
    assert_eq!(yaml, YAML_SERIALIZED_ACT);
}

#[test]
fn test_act_json_serialization() {
    let act = get_test_act();
    let json = serde_json::to_string(&act).unwrap();
    let roundtrip: Act = serde_json::from_str(&json).unwrap();
    assert_eq!(act, roundtrip);
}

#[test]
fn test_reference_serialization() {
    let references = vec![
        ReferenceBuilder::new()
            .set_part(ActIdentifier {
                year: 2012,
                number: 123,
            })
            .set_part(RefPartArticle::from_single("1:23/B".parse().unwrap()))
            .set_part(RefPartParagraph::from_single("2b".parse().unwrap()))
            .set_part(RefPartPoint::from_single(
                NumericIdentifier::from_str("1").unwrap(),
            ))
            .set_part(RefPartSubpoint::from_single(
                PrefixedAlphabeticIdentifier::from_str("a").unwrap(),
            ))
            .build()
            .unwrap(),
        ReferenceBuilder::new()
            .set_part(RefPartPoint::from_single(
                AlphabeticIdentifier::from_str("sz").unwrap(),
            ))
            .set_part(RefPartSubpoint::from_single(
                NumericIdentifier::from_str("12").unwrap(),
            ))
            .build()
            .unwrap(),
        ReferenceBuilder::new()
            .set_part(RefPartArticle::from_range(
                "1".parse().unwrap(),
                "2".parse().unwrap(),
            ))
            .build()
            .unwrap(),
        ReferenceBuilder::new()
            .set_part(ActIdentifier {
                year: 2012,
                number: 123,
            })
            .build()
            .unwrap(),
        ReferenceBuilder::new()
            .set_part(RefPartParagraph::from_range(
                "1".parse().unwrap(),
                "5".parse().unwrap(),
            ))
            .build()
            .unwrap(),
        ReferenceBuilder::new()
            .set_part(RefPartPoint::from_range(
                NumericIdentifier::from_str("1").unwrap(),
                NumericIdentifier::from_str("5").unwrap(),
            ))
            .build()
            .unwrap(),
        ReferenceBuilder::new()
            .set_part(RefPartPoint::from_range(
                AlphabeticIdentifier::from_str("a").unwrap(),
                AlphabeticIdentifier::from_str("c").unwrap(),
            ))
            .build()
            .unwrap(),
        ReferenceBuilder::new()
            .set_part(RefPartSubpoint::from_range(
                NumericIdentifier::from_str("1").unwrap(),
                NumericIdentifier::from_str("5").unwrap(),
            ))
            .build()
            .unwrap(),
        ReferenceBuilder::new()
            .set_part(RefPartSubpoint::from_range(
                PrefixedAlphabeticIdentifier::from_str("ca").unwrap(),
                PrefixedAlphabeticIdentifier::from_str("cc").unwrap(),
            ))
            .build()
            .unwrap(),
    ];
    let expected_yaml = r#"- act:
    year: 2012
    number: 123
  article: 1:23/B
  paragraph: 2b
  point: '1'
  subpoint: a
- point: sz
  subpoint: '12'
- article:
    start: '1'
    end: '2'
- act:
    year: 2012
    number: 123
- paragraph:
    start: '1'
    end: '5'
- point:
    start: '1'
    end: '5'
- point:
    start: a
    end: c
- subpoint:
    start: '1'
    end: '5'
- subpoint:
    start: ca
    end: cc
"#;
    let yaml = singleton_yaml::to_string(&references).unwrap();
    let roundtrip: Vec<Reference> = singleton_yaml::from_str(&yaml).unwrap();
    assert_eq!(references, roundtrip);
    assert_eq!(yaml, expected_yaml);
    let json = serde_json::to_string(&references).unwrap();
    let roundtrip: Vec<Reference> = serde_json::from_str(&json).unwrap();
    assert_eq!(references, roundtrip);
}

#[test]
fn test_invalid_reference_deserialization() {
    assert!(singleton_yaml::from_str::<Reference>(
        r#"---
        paragraph:
            start: "1"
            end: "5"
        point:
            start: "1"
            end: "5"
    "#
    )
    .is_err());
    assert!(singleton_yaml::from_str::<Reference>(
        r#"---
        article: "1"
        subpoint: "a"
    "#
    )
    .is_err());
}
