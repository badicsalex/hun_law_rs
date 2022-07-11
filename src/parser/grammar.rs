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
pub struct OutgoingReference {
    pub start: usize,
    pub end: usize,
    pub reference: reference::Reference,
}

pub type Abbreviations = HashMap<String, ActIdentifier>;

impl ListOfSimpleExpressions {
    pub fn get_outgoing_references(&self, abbreviations: &Abbreviations) -> Vec<OutgoingReference> {
        self.contents
            .iter()
            .filter_map(|item| {
                if let AnySimpleExpression::CompoundReference(reference) = item {
                    reference.get_outgoing_references(abbreviations).ok()
                } else {
                    None
                }
            })
            .flatten()
            .collect()
    }
}

impl CompoundReference {
    pub fn get_outgoing_references(
        &self,
        abbreviations: &Abbreviations,
    ) -> Result<Vec<OutgoingReference>> {
        let mut ref_builder = OutgoingReferenceBuilder::new(abbreviations);
        ref_builder.feed(&self.act_reference)?;
        for reference in &self.references {
            ref_builder.feed(reference)?;
        }
        Ok(ref_builder.get_result())
    }
}

#[derive(Debug)]
struct OutgoingReferenceBuilder<'a> {
    ref_builder: ReferenceBuilder,
    abbreviations: &'a Abbreviations,
    result: Vec<OutgoingReference>,
    start: Option<usize>,
    end: usize,
}

impl<'a> OutgoingReferenceBuilder<'a> {
    pub fn new(abbreviations: &'a Abbreviations) -> Self {
        Self {
            ref_builder: ReferenceBuilder::new(),
            abbreviations,
            result: Vec::new(),
            start: None,
            end: 0,
        }
    }

    fn record_one(&mut self) -> Result<()> {
        self.result.push(OutgoingReference {
            start: self.start.ok_or_else(|| {
                anyhow!("Trying to build an OutgoingReference before supplying any parts")
            })?,
            end: self.end,
            reference: self.ref_builder.build()?,
        });
        self.start = None;
        Ok(())
    }

    fn set_part<T>(&mut self, start: usize, end: usize, part: T)
    where
        ReferenceBuilder: ReferenceBuilderSetPart<T>,
    {
        if self.start.is_none() {
            self.start = Some(start)
        }
        self.end = end;
        self.ref_builder.set_part(part);
    }

    pub fn get_result(self) -> Vec<OutgoingReference> {
        self.result
    }
}

trait FeedReferenceBuilder<T> {
    fn feed(&mut self, element: &T) -> Result<()>;
}

impl FeedReferenceBuilder<ActReference> for OutgoingReferenceBuilder<'_> {
    fn feed(&mut self, element: &ActReference) -> Result<()> {
        match element {
            ActReference::Abbreviation(abbrev) => {
                self.set_part(
                    abbrev.position.start,
                    abbrev.position.end,
                    abbrev.resolve(self.abbreviations)?,
                );
            }
            ActReference::ActIdWithFromNowOn(ActIdWithFromNowOn { act_id, .. }) => {
                self.set_part(
                    act_id.position.start,
                    act_id.position.end,
                    ActIdentifier::try_from(act_id.clone())?,
                );
            }
        }
        self.record_one()?;
        Ok(())
    }
}

impl<'a, T> FeedReferenceBuilder<Option<T>> for OutgoingReferenceBuilder<'a>
where
    OutgoingReferenceBuilder<'a>: FeedReferenceBuilder<T>,
{
    fn feed(&mut self, element: &Option<T>) -> Result<()> {
        if let Some(val) = element {
            self.feed(val)
        } else {
            Ok(())
        }
    }
}

impl<'a, T> FeedReferenceBuilder<Vec<T>> for OutgoingReferenceBuilder<'a>
where
    OutgoingReferenceBuilder<'a>: FeedReferenceBuilder<T>,
{
    fn feed(&mut self, element: &Vec<T>) -> Result<()> {
        for (num, part) in element.iter().enumerate() {
            if num > 0 {
                self.record_one()?;
            }
            self.feed(part)?;
        }
        Ok(())
    }
}

impl FeedReferenceBuilder<Reference> for OutgoingReferenceBuilder<'_> {
    fn feed(&mut self, element: &Reference) -> Result<()> {
        self.feed(&element.article)?;
        self.feed(&element.paragraph)?;
        self.feed(&element.numeric_point)?;
        self.feed(&element.alphabetic_point)?;
        self.feed(&element.numeric_subpoint)?;
        self.feed(&element.alphabetic_subpoint)?;
        self.end = element.position.end;
        self.record_one()?;
        Ok(())
    }
}

trait ReferenceComponentPart {
    type RefPart;
    fn to_ref_part(&self) -> Result<Self::RefPart>;
    fn start(&self) -> usize;
    fn end(&self) -> usize;
}

macro_rules! impl_rcp {
    ($T:ident, $PartsT:ident, $RefPart:ident, $IdType:ident, $Range:ident, $Single:ident) => {
        impl FeedReferenceBuilder<$T> for OutgoingReferenceBuilder<'_> {
            fn feed(&mut self, element: &$T) -> Result<()> {
                self.feed(&element.parts)
            }
        }
        impl FeedReferenceBuilder<$PartsT> for OutgoingReferenceBuilder<'_> {
            fn feed(&mut self, element: &$PartsT) -> Result<()> {
                match element {
                    $PartsT::$Range(x) => {
                        let part = $RefPart::from_range(
                            x.start.parse::<$IdType>()?,
                            x.end.parse::<$IdType>()?,
                        );
                        self.set_part(x.position.start, x.position.end, part);
                    }
                    $PartsT::$Single(x) => {
                        let part = $RefPart::from_single(x.id.parse::<$IdType>()?);
                        self.set_part(x.position.start, x.position.end, part);
                    }
                }
                Ok(())
            }
        }
    };
}

impl_rcp!(
    ArticleReference,
    ArticleReference_parts,
    RefPartArticle,
    ArticleIdentifier,
    ArticleRange,
    ArticleSingle
);
impl_rcp!(
    ParagraphReference,
    ParagraphReference_parts,
    RefPartParagraph,
    NumericIdentifier,
    ParagraphRange,
    ParagraphSingle
);
impl_rcp!(
    NumericPointReference,
    NumericPointReference_parts,
    RefPartPoint,
    NumericIdentifier,
    NumericPointRange,
    NumericPointSingle
);
impl_rcp!(
    AlphabeticPointReference,
    AlphabeticPointReference_parts,
    RefPartPoint,
    AlphabeticIdentifier,
    AlphabeticPointRange,
    AlphabeticPointSingle
);
impl_rcp!(
    NumericSubpointReference,
    NumericSubpointReference_parts,
    RefPartSubpoint,
    NumericIdentifier,
    NumericSubpointRange,
    NumericSubpointSingle
);
impl_rcp!(
    AlphabeticSubpointReference,
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
