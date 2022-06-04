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

use hun_law::{
    structure::{
        Act, ActChild, AlphabeticPoint, AlphabeticPointChildren, AlphabeticSubpoint, Article,
        NumericPoint, Paragraph, ParagraphChildren, SAEBody, StructuralElement,
        StructuralElementType,
    },
    util::date::Date,
};
use rstest::rstest;

fn get_test_structure() -> Act {
    Act {
        identifier: "2345. évi XD. törvény".into(),
        publication_date: Date {
            year: 2345,
            month: 6,
            day: 7,
        },
        subject: "A tesztelésről".into(),
        preamble: "A tesztelés nagyon fontos, és egyben kötelező".into(),
        children: vec![
            ActChild::StructuralElement(StructuralElement {
                identifier: "1".into(),
                title: "Egyszerű dolgok".into(),
                element_type: StructuralElementType::Book,
            }),
            ActChild::Article(Article {
                identifier: "1:1".into(),
                title: Some("Az egyetlen cikk, aminek cime van.".into()),
                children: vec![Paragraph {
                    identifier: "".into(),
                    body: SAEBody::Text("Meg szövege".into()),
                }],
            }),
            ActChild::Article(Article {
                identifier: "1:2".into(),
                title: None,
                children: vec![
                    Paragraph {
                        identifier: "1".into(),
                        body: SAEBody::Text("Valami valami".into()),
                    },
                    Paragraph {
                        identifier: "2".into(),
                        body: SAEBody::Children {
                            intro: "Egy felsorolás legyen".into(),
                            wrap_up: "minden esetben.".into(),
                            children: ParagraphChildren::AlphabeticPoint(vec![
                                AlphabeticPoint {
                                    identifier: "a".into(),
                                    body: SAEBody::Text("többelemű".into()),
                                },
                                AlphabeticPoint {
                                    identifier: "b".into(),
                                    body: SAEBody::Children {
                                        intro: "kellően".into(),
                                        wrap_up: "".into(),
                                        children: AlphabeticPointChildren::AlphabeticSubpoint(
                                            vec![
                                                AlphabeticSubpoint {
                                                    identifier: "ba".into(),
                                                    body: SAEBody::Text("átláthatatlan".into()),
                                                },
                                                AlphabeticSubpoint {
                                                    identifier: "bb".into(),
                                                    body: SAEBody::Text("komplex".into()),
                                                },
                                            ],
                                        ),
                                    },
                                },
                            ]),
                        },
                    },
                ],
            }),
            ActChild::StructuralElement(StructuralElement {
                identifier: "2".into(),
                title: "Amended stuff in english".into(),
                element_type: StructuralElementType::Book,
            }),
            ActChild::Article(Article {
                identifier: "2:1".into(),
                title: None,
                children: vec![Paragraph {
                    identifier: "".into(),
                    body: SAEBody::Text("Nothing fancy yet".into()),
                }],
            }),
            ActChild::Article(Article {
                identifier: "2:1/A".into(),
                title: None,
                children: vec![Paragraph {
                    identifier: "".into(),
                    body: SAEBody::Text("Added after the fact".into()),
                }],
            }),
            ActChild::Article(Article {
                identifier: "2:2".into(),
                title: None,
                children: vec![Paragraph {
                    identifier: "1".into(),
                    body: SAEBody::Children {
                        intro: "This can legally be after 2:1/A. Also, ".into(),
                        wrap_up: "Can also be amended".into(),
                        children: ParagraphChildren::NumericPoint(vec![
                            NumericPoint {
                                identifier: "1".into(),
                                body: SAEBody::Text("Paragraphs".into()),
                            },
                            NumericPoint {
                                identifier: "1a".into(),
                                body: SAEBody::Text("Numeric points".into()),
                            },
                            NumericPoint {
                                identifier: "2".into(),
                                body: SAEBody::Text("Alphabetic points".into()),
                            },
                        ]),
                    },
                }],
            }),
        ],
    }
}

const YAML_SERIALIZED: &str = r#"---
identifier: 2345. évi XD. törvény
subject: A tesztelésről
preamble: "A tesztelés nagyon fontos, és egyben kötelező"
publication_date:
  year: 2345
  month: 6
  day: 7
children:
  - StructuralElement:
      identifier: "1"
      title: Egyszerű dolgok
      element_type: Book
  - Article:
      identifier: "1:1"
      title: "Az egyetlen cikk, aminek cime van."
      children:
        - body: Meg szövege
  - Article:
      identifier: "1:2"
      children:
        - identifier: "1"
          body: Valami valami
        - identifier: "2"
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
      identifier: "2"
      title: Amended stuff in english
      element_type: Book
  - Article:
      identifier: "2:1"
      children:
        - body: Nothing fancy yet
  - Article:
      identifier: "2:1/A"
      children:
        - body: Added after the fact
  - Article:
      identifier: "2:2"
      children:
        - identifier: "1"
          body:
            intro: "This can legally be after 2:1/A. Also, "
            children:
              NumericPoint:
                - identifier: "1"
                  body: Paragraphs
                - identifier: 1a
                  body: Numeric points
                - identifier: "2"
                  body: Alphabetic points
            wrap_up: Can also be amended
"#;

#[rstest]
fn test_yaml_serialization() {
    let act = get_test_structure();
    let yaml = serde_yaml::to_string(&act).unwrap();
    let roundtrip: Act = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(act, roundtrip);
    assert_eq!(yaml, YAML_SERIALIZED);
}

#[rstest]
fn test_json_serialization() {
    let act = get_test_structure();
    let json = serde_json::to_string(&act).unwrap();
    let roundtrip: Act = serde_json::from_str(&json).unwrap();
    assert_eq!(act, roundtrip);
}
