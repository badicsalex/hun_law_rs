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

use std::collections::HashMap;

use crate::structure::ActIdentifier;

#[derive(Debug, Default)]
pub struct AbbreviationCache {
    cache: HashMap<String, ActIdentifier>,
}

impl AbbreviationCache {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add(&mut self, abbreviation: String, act_id: ActIdentifier) {
        self.cache.insert(abbreviation, act_id);
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
