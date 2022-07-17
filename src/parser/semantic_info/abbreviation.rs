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

use crate::structure::{semantic_info::ActIdAbbreviation, ActIdentifier};
use hun_law_grammar::*;

pub trait GetNewAbbreviations {
    fn get_new_abbreviations(&self) -> Result<Vec<ActIdAbbreviation>>;
}

impl GetNewAbbreviations for Root {
    fn get_new_abbreviations(&self) -> Result<Vec<ActIdAbbreviation>> {
        match &self.content {
            Root_content::ArticleTitleAmendment(c) => c.get_new_abbreviations(),
            Root_content::BlockAmendment(c) => c.get_new_abbreviations(),
            Root_content::BlockAmendmentStructural(c) => c.get_new_abbreviations(),
            Root_content::BlockAmendmentWithSubtitle(c) => c.get_new_abbreviations(),
            Root_content::EnforcementDate(c) => c.get_new_abbreviations(),
            Root_content::ListOfSimpleExpressions(c) => c.get_new_abbreviations(),
            Root_content::Repeal(c) => c.get_new_abbreviations(),
            Root_content::StructuralRepeal(c) => c.get_new_abbreviations(),
            Root_content::TextAmendment(c) => c.get_new_abbreviations(),
        }
    }
}

macro_rules! trivial_impl {
    ($type:ident) => {
        impl GetNewAbbreviations for $type {
            fn get_new_abbreviations(&self) -> Result<Vec<ActIdAbbreviation>> {
                self.act_reference.get_new_abbreviations()
            }
        }
    };
}

trivial_impl!(ArticleTitleAmendment);
trivial_impl!(BlockAmendment);
trivial_impl!(BlockAmendmentStructural);
trivial_impl!(BlockAmendmentWithSubtitle);
trivial_impl!(Repeal);
trivial_impl!(StructuralRepeal);
trivial_impl!(TextAmendment);

impl GetNewAbbreviations for EnforcementDate {
    fn get_new_abbreviations(&self) -> Result<Vec<ActIdAbbreviation>> {
        Ok(vec![])
    }
}

impl GetNewAbbreviations for ListOfSimpleExpressions {
    fn get_new_abbreviations(&self) -> Result<Vec<ActIdAbbreviation>> {
        Ok(self
            .contents
            .iter()
            .filter_map(|item| {
                if let AnySimpleExpression::CompoundReference(reference) = item {
                    // TODO: Errors are swallowed here. Maybe log it?
                    reference.get_new_abbreviations().ok()
                } else {
                    None
                }
            })
            .flatten()
            .collect())
    }
}

impl GetNewAbbreviations for CompoundReference {
    fn get_new_abbreviations(&self) -> Result<Vec<ActIdAbbreviation>> {
        if let Some(act_reference) = &self.act_reference {
            act_reference.get_new_abbreviations()
        } else {
            Ok(vec![])
        }
    }
}

impl GetNewAbbreviations for ActReference {
    fn get_new_abbreviations(&self) -> Result<Vec<ActIdAbbreviation>> {
        if let Self::ActIdWithFromNowOn(act_id_fno) = self {
            if let Some(abbreviation) = &act_id_fno.abbreviation {
                let act_id = &act_id_fno.act_id;
                return Ok(vec![ActIdAbbreviation {
                    act_id: act_id.try_into()?,
                    abbreviation: abbreviation.content.clone(),
                }]);
            }
        };
        Ok(vec![])
    }
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
