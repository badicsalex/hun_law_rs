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

use std::collections::HashMap;

use anyhow::{anyhow, Result};
use derive_visitor::{visitor_enter_fn, Drive};
use hun_law_grammar::*;

use crate::{identifier::ActIdentifier, semantic_info::ActIdAbbreviation};

pub fn get_new_abbreviations(root: &Root) -> Result<Vec<ActIdAbbreviation>> {
    let mut result: Vec<Result<ActIdAbbreviation>> = Vec::new();
    let act_id_visitor = |abbrev: &ActIdWithFromNowOn| {
        if let Some(abbrev_elem) = &abbrev.abbreviation {
            result.push(
                ActIdentifier::try_from(&abbrev.act_id).map(|act_id| ActIdAbbreviation {
                    act_id,
                    abbreviation: abbrev_elem.content.clone(),
                }),
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
    cache: HashMap<String, ActIdentifier>,
}

impl AbbreviationCache {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add(&mut self, element: &ActIdAbbreviation) {
        self.cache
            .insert(element.abbreviation.clone(), element.act_id);
    }

    pub fn add_multiple(&mut self, elements: &[ActIdAbbreviation]) {
        for element in elements {
            self.add(element);
        }
    }

    pub fn resolve(&self, abbreviation: &str) -> Result<ActIdentifier> {
        self.cache
            .get(abbreviation)
            .ok_or_else(|| anyhow!("{} not found in the abbreviations cache", abbreviation))
            .cloned()
    }
}

// Should only be used by integration tests, as a shorthand
impl From<HashMap<String, ActIdentifier>> for AbbreviationCache {
    fn from(cache: HashMap<String, ActIdentifier>) -> Self {
        Self { cache }
    }
}
