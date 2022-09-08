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

use self::{abbreviation::AbbreviationCache, sae::SemanticInfoAdder};

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
        let mut abbreviation_cache = AbbreviationCache::default();
        let mut visitor = SemanticInfoAdder::new(&mut abbreviation_cache);
        self.walk_saes_mut(&mut visitor)
    }
}
