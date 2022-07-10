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

use crate::{
    reference::{
        self, RefPartArticle, RefPartFrom, RefPartParagraph, RefPartPoint, RefPartSubpoint,
        ReferenceBuilder, ReferenceBuilderSetPart,
    },
    structure::{
        ActIdentifier, AlphabeticIdentifier, ArticleIdentifier, NumericIdentifier,
        PrefixedAlphabeticIdentifier,
    },
};

use super::grammar_generated::*;

#[derive(Debug, Clone)]
pub struct InTextReference {
    pub start: usize,
    pub end: usize,
    pub reference: reference::Reference,
}

pub type Abbreviations = HashMap<String, ActIdentifier>;

impl ListOfSimpleExpressions {
    pub fn get_in_text_references(&self, abbreviations: &Abbreviations) -> Vec<InTextReference> {
        self.contents
            .iter()
            .filter_map(|item| {
                if let AnySimpleExpression::CompoundReference(reference) = item {
                    reference.get_in_text_references(abbreviations).ok()
                } else {
                    None
                }
            })
            .flatten()
            .collect()
    }
}

impl CompoundReference {
    pub fn get_in_text_references(
        &self,
        abbreviations: &Abbreviations,
    ) -> Result<Vec<InTextReference>> {
        let mut ref_builder = ReferenceBuilder::new();
        let mut result = Vec::new();
        if let Some(act_ref) = &self.act_reference {
            let (pos, act_id_maybe) = match act_ref {
                ActReference::Abbreviation(abbrev) => {
                    (&abbrev.position, abbrev.resolve(abbreviations))
                }
                ActReference::ActIdWithFromNowOn(act_id_fno) => (
                    &act_id_fno.act_id.position,
                    act_id_fno.act_id.clone().try_into(),
                ),
            };
            println!("Act ID: {:?}...", act_id_maybe);
            ref_builder.set_part(act_id_maybe?);
            result.push(InTextReference {
                start: pos.start,
                end: pos.end,
                reference: ref_builder.build()?,
            });
        }
        for reference in &self.references {
            result.append(&mut reference.get_in_text_references(&mut ref_builder.clone())?)
        }
        Ok(result)
    }
}

impl Reference {
    pub fn get_in_text_references(
        &self,
        ref_builder: &mut ReferenceBuilder,
    ) -> Result<Vec<InTextReference>> {
        let mut result = Vec::new();
        let mut start = 0;
        let mut end = 0;

        // XXX: This is absolutely horrifying.
        // I tried doing it with trait-based generics, but it was even worse.
        macro_rules! process {
            ($part_name:ident) => {
                if let Some($part_name) = &self.$part_name {
                    for (num, part) in $part_name.parts.iter().enumerate() {
                        if num > 0 {
                            result.push(InTextReference {
                                start,
                                end,
                                reference: ref_builder.build()?,
                            });
                        }
                        if num > 0 || start == 0 {
                            start = part.start();
                        }
                        ref_builder.set_part(part.to_ref_part()?);
                        end = part.end();
                    }
                }
            };
        }

        process!(article);
        process!(paragraph);
        process!(numeric_point);
        process!(alphabetic_point);
        process!(numeric_subpoint);
        process!(alphabetic_subpoint);

        result.push(InTextReference {
            start,
            end,
            reference: ref_builder.build()?,
        });
        Ok(result)
    }
}

trait ReferenceComponentPart {
    type RefPart;
    fn to_ref_part(&self) -> Result<Self::RefPart>;
    fn start(&self) -> usize;
    fn end(&self) -> usize;
}

macro_rules! impl_rcp {
    ($T:ident, $RefPart:ident, $IdType:ident, $Range:ident, $Single:ident) => {
        impl ReferenceComponentPart for $T {
            type RefPart = $RefPart;

            fn to_ref_part(&self) -> Result<Self::RefPart> {
                Ok(match self {
                    Self::$Range(x) => {
                        $RefPart::from_range(x.start.parse::<$IdType>()?, x.end.parse::<$IdType>()?)
                    }
                    Self::$Single(x) => $RefPart::from_single(x.id.parse::<$IdType>()?),
                })
            }

            fn start(&self) -> usize {
                match self {
                    Self::$Range(x) => x.position.start,
                    Self::$Single(x) => x.position.start,
                }
            }

            fn end(&self) -> usize {
                match self {
                    Self::$Range(x) => x.position.end,
                    Self::$Single(x) => x.position.end,
                }
            }
        }
    };
}

impl_rcp!(
    ArticleReference_parts,
    RefPartArticle,
    ArticleIdentifier,
    ArticleRange,
    ArticleSingle
);

impl_rcp!(
    ParagraphReference_parts,
    RefPartParagraph,
    NumericIdentifier,
    ParagraphRange,
    ParagraphSingle
);

impl_rcp!(
    NumericPointReference_parts,
    RefPartPoint,
    NumericIdentifier,
    NumericPointRange,
    NumericPointSingle
);
impl_rcp!(
    AlphabeticPointReference_parts,
    RefPartPoint,
    AlphabeticIdentifier,
    AlphabeticPointRange,
    AlphabeticPointSingle
);
impl_rcp!(
    NumericSubpointReference_parts,
    RefPartSubpoint,
    NumericIdentifier,
    NumericSubpointRange,
    NumericSubpointSingle
);
impl_rcp!(
    AlphabeticSubpointReference_parts,
    RefPartSubpoint,
    PrefixedAlphabeticIdentifier,
    AlphabeticSubpointRange,
    AlphabeticSubpointSingle
);

impl Abbreviation {
    pub fn resolve(&self, abbreviations: &Abbreviations) -> Result<ActIdentifier> {
        abbreviations
            .get(&self.content)
            .ok_or_else(|| anyhow!("{} not found in abbreviations", self.content))
            .cloned()
    }
}

impl TryFrom<ActId> for ActIdentifier {
    type Error = anyhow::Error;

    fn try_from(act_id: ActId) -> Result<Self, Self::Error> {
        Ok(ActIdentifier {
            year: act_id.year.parse()?,
            number: roman::from(&act_id.number).ok_or_else(|| {
                anyhow!("{} is not a valid suffixed roman numeral", act_id.number)
            })?,
        })
    }
}
