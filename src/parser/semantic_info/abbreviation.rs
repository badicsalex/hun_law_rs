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

use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use derive_visitor::{visitor_enter_fn, Drive};
use hun_law_grammar::*;

use crate::identifier::ActIdentifier;

pub fn get_new_abbreviations(root: &Root) -> Result<Vec<(String, ActIdentifier)>> {
    let mut result: Vec<Result<(String, ActIdentifier)>> = Vec::new();
    let act_id_visitor = |abbrev: &ActIdWithFromNowOn| {
        if let Some(abbrev_elem) = &abbrev.abbreviation {
            result.push(
                ActIdentifier::try_from(&abbrev.act_id)
                    .map(|act_id| (abbrev_elem.content.clone(), act_id)),
            )
        }
    };
    // Find and process all ActIdWithFromNowOn-s
    root.drive(&mut visitor_enter_fn(act_id_visitor));

    // Transpose Vec<Result> to Result<Vec>
    result.into_iter().collect()
}

#[derive(Debug, Default)]
pub struct AbbreviationCache {
    cache: BTreeMap<String, ActIdentifier>,
    has_changed: bool,
}

impl AbbreviationCache {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add(&mut self, abbreviation: String, act_id: ActIdentifier) {
        if let Some(v) = self.cache.get_mut(&abbreviation) {
            if *v != act_id {
                *v = act_id;
                self.has_changed = true;
            }
        } else {
            self.cache.insert(abbreviation, act_id);
            self.has_changed = true;
        }
    }

    pub fn add_multiple(&mut self, elements: &[(String, ActIdentifier)]) {
        for (abbreviation, act_id) in elements {
            self.add(abbreviation.clone(), *act_id);
        }
    }

    pub fn resolve(&self, abbreviation: &str) -> Result<ActIdentifier> {
        self.cache
            .get(abbreviation)
            .ok_or_else(|| anyhow!("{} not found in the abbreviations cache", abbreviation))
            .cloned()
    }

    pub fn has_changed(&self) -> bool {
        self.has_changed
    }
}

impl From<BTreeMap<String, ActIdentifier>> for AbbreviationCache {
    fn from(cache: BTreeMap<String, ActIdentifier>) -> Self {
        Self {
            cache,
            has_changed: false,
        }
    }
}

impl From<AbbreviationCache> for BTreeMap<String, ActIdentifier> {
    fn from(ac: AbbreviationCache) -> Self {
        ac.cache
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_abbreviation_cache() {
        let mut cache = AbbreviationCache::new();
        cache.add(
            "Lol.".into(),
            ActIdentifier {
                year: 2012,
                number: 13,
            },
        );
        cache.add(
            "Tv.".into(),
            ActIdentifier {
                year: 2012,
                number: 14,
            },
        );
        cache.add(
            "Lol.".into(),
            ActIdentifier {
                year: 2019,
                number: 19,
            },
        );
        assert!(cache.has_changed());
        let data = BTreeMap::<String, ActIdentifier>::from(cache);
        assert_eq!(data.len(), 2);
        assert_eq!(
            data.get("Tv."),
            Some(&ActIdentifier {
                year: 2012,
                number: 14
            })
        );

        let mut new_cache = AbbreviationCache::from(data.clone());
        assert!(!new_cache.has_changed());
        new_cache.add(
            "Tv.".into(),
            ActIdentifier {
                year: 2012,
                number: 14,
            },
        );
        assert!(!new_cache.has_changed());
        new_cache.add(
            "Tv.".into(),
            ActIdentifier {
                year: 2017,
                number: 17,
            },
        );
        assert!(new_cache.has_changed());

        let mut new_cache2 = AbbreviationCache::from(data);
        assert!(!new_cache2.has_changed());
        new_cache2.add(
            "Xd.".into(),
            ActIdentifier {
                year: 2042,
                number: 42,
            },
        );
        assert!(new_cache2.has_changed());
    }
}
