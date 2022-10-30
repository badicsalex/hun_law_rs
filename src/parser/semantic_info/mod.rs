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

use anyhow::{anyhow, Result};

use self::sae::SemanticInfoAdder;
use crate::{
    identifier::ArticleIdentifier, reference::to_element::ReferenceToElement, structure::Act,
    util::walker::WalkSAEMut,
};

pub mod abbreviation;
pub mod article_title_amendment;
pub mod block_amendment;
pub mod enforcement_date;
pub mod reference;
pub mod repeal;
pub mod sae;
pub mod structural_reference;
pub mod text_amendment;

impl Act {
    pub fn add_semantic_info(&mut self) -> Result<AbbreviationsChanged> {
        let mut abbreviation_cache = self.contained_abbreviations.clone().into();
        let mut visitor = SemanticInfoAdder::new(&mut abbreviation_cache);
        self.walk_saes_mut(&mut visitor)?;
        let abbreviations_changed = abbreviation_cache.has_changed().into();
        self.contained_abbreviations = abbreviation_cache.into();
        Ok(abbreviations_changed)
    }

    pub fn add_semantic_info_to_article(
        &mut self,
        article_id: ArticleIdentifier,
    ) -> Result<AbbreviationsChanged> {
        let act_id = self.identifier;
        let mut abbreviation_cache = self.contained_abbreviations.clone().into();
        let mut visitor = SemanticInfoAdder::new(&mut abbreviation_cache);
        let base_reference = self.reference();
        let article = self
            .article_mut(article_id)
            .ok_or_else(|| anyhow!("Could not find article {article_id} in act {act_id}"))?;
        article.walk_saes_mut(&base_reference, &mut visitor)?;
        let abbreviations_changed = abbreviation_cache.has_changed().into();
        self.contained_abbreviations = abbreviation_cache.into();
        Ok(abbreviations_changed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbbreviationsChanged {
    No,
    Yes,
}

impl From<bool> for AbbreviationsChanged {
    fn from(b: bool) -> Self {
        if b {
            Self::Yes
        } else {
            Self::No
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        identifier::ActIdentifier,
        structure::{Article, Paragraph},
    };

    #[test]
    fn test_add_semantic_abbrevs() {
        let mut test_act = Act {
            identifier: ActIdentifier {
                year: 2345,
                number: 0xd,
            },
            publication_date: NaiveDate::from_ymd(2345, 6, 7),
            subject: "A tesztelésről".into(),
            preamble: "A tesztelés nagyon fontos, és egyben kötelező".into(),
            contained_abbreviations: Default::default(),
            children: vec![Article {
                identifier: "1:1".parse().unwrap(),
                title: Some("Az egyetlen cikk, aminek cime van.".into()),
                children: vec![
                    Paragraph {
                        identifier: 1.into(),
                        body: "Létezik a csodákról szóló 2022. évi XXII. törvény (a továbbiakban: Cstv.)."
                            .into(),
                        semantic_info: Default::default(),
                        last_change: None,
                    },
                    Paragraph {
                        identifier: 2.into(),
                        body: "A Cstv. 5. §-a fontos.".into(),
                        semantic_info: Default::default(),
                        last_change: None,
                    },
                ],
                last_change: None,
            }
            .into()],
        };
        let abbrevs_changed = test_act.add_semantic_info().unwrap();
        let expected_abbreviations = [(
            "Cstv.".to_string(),
            ActIdentifier {
                year: 2022,
                number: 22,
            },
        )]
        .into();
        assert_eq!(test_act.contained_abbreviations, expected_abbreviations);
        assert_eq!(abbrevs_changed, AbbreviationsChanged::Yes);
        assert_eq!(
            test_act.articles().next().unwrap().children[1]
                .semantic_info
                .outgoing_references[0]
                .reference
                .act()
                .unwrap(),
            ActIdentifier {
                year: 2022,
                number: 22,
            }
        );

        test_act.articles_mut().next().unwrap().children[0] = Paragraph {
            identifier: 1.into(),
            body: "I am kill.".into(),
            semantic_info: Default::default(),
            last_change: None,
        };
        let abbrevs_changed = test_act.add_semantic_info().unwrap();
        assert_eq!(test_act.contained_abbreviations, expected_abbreviations);
        assert_eq!(abbrevs_changed, AbbreviationsChanged::No);
        assert_eq!(
            test_act.articles().next().unwrap().children[1]
                .semantic_info
                .outgoing_references[0]
                .reference
                .act()
                .unwrap(),
            ActIdentifier {
                year: 2022,
                number: 22,
            }
        );

        test_act.articles_mut().next().unwrap().children[0] = Paragraph {
            identifier: 1.into(),
            body: "Létezik a csodákról szóló 2033. évi XXXIII. törvény (a továbbiakban: Cstv.)."
                .into(),
            semantic_info: Default::default(),
            last_change: None,
        };
        let abbrevs_changed = test_act.add_semantic_info().unwrap();
        let expected_abbreviations = [(
            "Cstv.".to_string(),
            ActIdentifier {
                year: 2033,
                number: 33,
            },
        )]
        .into();
        assert_eq!(test_act.contained_abbreviations, expected_abbreviations);
        assert_eq!(abbrevs_changed, AbbreviationsChanged::Yes);
        assert_eq!(
            test_act.articles().next().unwrap().children[1]
                .semantic_info
                .outgoing_references[0]
                .reference
                .act()
                .unwrap(),
            ActIdentifier {
                year: 2033,
                number: 33,
            }
        );
    }

    #[test]
    fn test_reparse_single_article() {
        let mut test_act = Act {
            identifier: ActIdentifier {
                year: 2345,
                number: 0xd,
            },
            publication_date: NaiveDate::from_ymd(2345, 6, 7),
            subject: "A tesztelésről".into(),
            preamble: "A tesztelés nagyon fontos, és egyben kötelező".into(),
            contained_abbreviations: Default::default(),
            children: vec![
                Article {
                    identifier: "1".parse().unwrap(),
                    title: None,
                    children: vec![
                        Paragraph {
                            identifier: Default::default(),
                            body: "Létezik a csodákról szóló 2022. évi XXII. törvény (a továbbiakban: Cstv.)."
                                .into(),
                            semantic_info: Default::default(),
                            last_change: None,
                        },
                    ],
                    last_change: None,
                }
                .into(),
                Article {
                    identifier: "2".parse().unwrap(),
                    title: None,
                    children: vec![
                        Paragraph {
                            identifier: Default::default(),
                            body: "A Cstv. 5. §-a fontos."
                                .into(),
                            semantic_info: Default::default(),
                            last_change: None,
                        },
                    ],
                    last_change: None,
                }
                .into(),
            ],
        };
        test_act.add_semantic_info().unwrap();
        assert_eq!(
            test_act.add_semantic_info_to_article(1.into()).unwrap(),
            AbbreviationsChanged::No
        );
        assert_eq!(
            test_act.add_semantic_info_to_article(2.into()).unwrap(),
            AbbreviationsChanged::No
        );
        assert!(test_act.add_semantic_info_to_article(3.into()).is_err());

        test_act.article_mut(1.into()).unwrap().children[0].body =
            "Létezik a csodákról szóló 2033. évi XXXIII. törvény (a továbbiakban: Cstv.).".into();
        assert_eq!(
            test_act.add_semantic_info_to_article(2.into()).unwrap(),
            AbbreviationsChanged::No
        );
        assert_eq!(
            test_act.add_semantic_info_to_article(1.into()).unwrap(),
            AbbreviationsChanged::Yes
        );
    }
}
