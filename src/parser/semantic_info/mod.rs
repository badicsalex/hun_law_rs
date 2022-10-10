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

use crate::structure::Act;

use self::sae::SemanticInfoAdder;

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
    pub fn add_semantic_info(&mut self) -> Result<()> {
        let mut abbreviation_cache = self.contained_abbreviations.clone().into();
        let mut visitor = SemanticInfoAdder::new(&mut abbreviation_cache);
        self.walk_saes_mut(&mut visitor)?;
        self.contained_abbreviations = abbreviation_cache.into();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    use crate::{
        identifier::ActIdentifier,
        structure::{Article, Paragraph},
    };

    use super::*;

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
                    },
                    Paragraph {
                        identifier: 2.into(),
                        body: "A Cstv. 5. §-a fontos.".into(),
                        semantic_info: Default::default(),
                    },
                ],
            }
            .into()],
        };
        test_act.add_semantic_info().unwrap();
        let expected_abbreviations = [(
            "Cstv.".to_string(),
            ActIdentifier {
                year: 2022,
                number: 22,
            },
        )]
        .into();
        assert_eq!(test_act.contained_abbreviations, expected_abbreviations);
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
        };
        test_act.add_semantic_info().unwrap();
        assert_eq!(test_act.contained_abbreviations, expected_abbreviations);
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
        };
        test_act.add_semantic_info().unwrap();
        let expected_abbreviations = [(
            "Cstv.".to_string(),
            ActIdentifier {
                year: 2033,
                number: 33,
            },
        )]
        .into();
        assert_eq!(test_act.contained_abbreviations, expected_abbreviations);
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
}
