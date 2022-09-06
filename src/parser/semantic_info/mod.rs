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

use crate::{
    semantic_info::{EnforcementDate, SemanticInfo, SpecialPhrase},
    structure::Act,
    util::walker::{SAEVisitor, WalkSAE, WalkSAEMut},
};

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
    pub fn all_enforcement_dates(&self) -> Result<Vec<EnforcementDate>> {
        let mut visitor = EnforcementDateAccumulator::default();
        self.walk_saes(&mut visitor)?;
        Ok(visitor.result)
    }
}

#[derive(Debug, Default)]
struct EnforcementDateAccumulator {
    result: Vec<EnforcementDate>,
}

impl SAEVisitor for EnforcementDateAccumulator {
    // on_enter and on_exit not needed, since EnforcementDates are always in leaf nodes.
    fn on_text(&mut self, _text: &String, semantic_info: &SemanticInfo) -> Result<()> {
        if let Some(SpecialPhrase::EnforcementDate(ed)) = &semantic_info.special_phrase {
            self.result.push(ed.clone())
        }
        Ok(())
    }
}
