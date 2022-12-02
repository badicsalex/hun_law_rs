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

use anyhow::{anyhow, bail, Result};
use hun_law_grammar::*;

use super::abbreviation::AbbreviationCache;
use crate::{
    identifier::{
        range::IdentifierRangeFrom, ActIdentifier, AlphabeticIdentifier, ArticleIdentifier,
        NumericIdentifier, PrefixedAlphabeticIdentifier,
    },
    reference::{
        builder::{ReferenceBuilder, ReferenceBuilderSetPart},
        parts::{RefPartArticle, RefPartParagraph, RefPartPoint, RefPartSubpoint},
    },
    semantic_info::OutgoingReference,
};

pub trait GetOutgoingReferences {
    fn get_outgoing_references(
        &self,
        abbreviation_cache: &AbbreviationCache,
    ) -> Result<Vec<OutgoingReference>>;
}

impl GetOutgoingReferences for Root {
    fn get_outgoing_references(
        &self,
        abbreviation_cache: &AbbreviationCache,
    ) -> Result<Vec<OutgoingReference>> {
        match &self.content {
            Root_content::BlockAmendment(c) => c.get_outgoing_references(abbreviation_cache),
            Root_content::BlockAmendmentStructural(c) => {
                c.get_outgoing_references(abbreviation_cache)
            }
            Root_content::BlockAmendmentWithSubtitle(c) => {
                c.get_outgoing_references(abbreviation_cache)
            }
            Root_content::EnforcementDate(c) => c.get_outgoing_references(abbreviation_cache),
            Root_content::ListOfSimpleExpressions(c) => {
                c.get_outgoing_references(abbreviation_cache)
            }
            Root_content::Repeal(c) => c.get_outgoing_references(abbreviation_cache),
            Root_content::TextAmendment(c) => c.get_outgoing_references(abbreviation_cache),
        }
    }
}

impl GetOutgoingReferences for BlockAmendment {
    fn get_outgoing_references(
        &self,
        abbreviation_cache: &AbbreviationCache,
    ) -> Result<Vec<OutgoingReference>> {
        let mut ref_builder = OutgoingReferenceBuilder::new(abbreviation_cache);
        ref_builder.feed(&self.act_reference)?;
        ref_builder.feed(&self.amended_reference)?;
        ref_builder.feed(&self.inserted_reference)?;
        Ok(ref_builder.get_result())
    }
}

impl GetOutgoingReferences for BlockAmendmentStructural {
    fn get_outgoing_references(
        &self,
        abbreviation_cache: &AbbreviationCache,
    ) -> Result<Vec<OutgoingReference>> {
        let mut ref_builder = OutgoingReferenceBuilder::new(abbreviation_cache);
        ref_builder.feed(&self.act_reference)?;
        // TODO: the structural part, but it may not be worthwhile
        Ok(ref_builder.get_result())
    }
}

impl GetOutgoingReferences for BlockAmendmentWithSubtitle {
    fn get_outgoing_references(
        &self,
        abbreviation_cache: &AbbreviationCache,
    ) -> Result<Vec<OutgoingReference>> {
        let mut ref_builder = OutgoingReferenceBuilder::new(abbreviation_cache);
        ref_builder.feed(&self.act_reference)?;
        ref_builder.feed(&self.article_relative)?;
        ref_builder.feed(&self.reference)?;
        Ok(ref_builder.get_result())
    }
}

impl GetOutgoingReferences for EnforcementDate {
    fn get_outgoing_references(
        &self,
        abbreviation_cache: &AbbreviationCache,
    ) -> Result<Vec<OutgoingReference>> {
        let mut ref_builder = OutgoingReferenceBuilder::new(abbreviation_cache);
        for ed_ref in &self.references {
            if let EnforcementDateReference::Reference(normal_ref) = ed_ref {
                ref_builder.feed(normal_ref)?;
            }
        }
        Ok(ref_builder.get_result())
    }
}

impl GetOutgoingReferences for ListOfSimpleExpressions {
    fn get_outgoing_references(
        &self,
        abbreviation_cache: &AbbreviationCache,
    ) -> Result<Vec<OutgoingReference>> {
        Ok(self
            .contents
            .iter()
            .filter_map(|item| {
                if let AnySimpleExpression::CompoundReference(reference) = item {
                    // TODO: Errors are swallowed here. Maybe log it?
                    reference.get_outgoing_references(abbreviation_cache).ok()
                } else {
                    None
                }
            })
            .flatten()
            .collect())
    }
}

impl GetOutgoingReferences for Repeal {
    fn get_outgoing_references(
        &self,
        abbreviation_cache: &AbbreviationCache,
    ) -> Result<Vec<OutgoingReference>> {
        let mut ref_builder = OutgoingReferenceBuilder::new(abbreviation_cache);
        ref_builder.feed(&self.act_reference)?;
        ref_builder.feed(&self.references)?;
        Ok(ref_builder.get_result())
    }
}

impl GetOutgoingReferences for TextAmendment {
    fn get_outgoing_references(
        &self,
        abbreviation_cache: &AbbreviationCache,
    ) -> Result<Vec<OutgoingReference>> {
        let mut ref_builder = OutgoingReferenceBuilder::new(abbreviation_cache);
        ref_builder.feed(&self.act_reference)?;
        ref_builder.feed(&self.references)?;
        Ok(ref_builder.get_result())
    }
}

impl GetOutgoingReferences for CompoundReference {
    fn get_outgoing_references(
        &self,
        abbreviation_cache: &AbbreviationCache,
    ) -> Result<Vec<OutgoingReference>> {
        let mut ref_builder = OutgoingReferenceBuilder::new(abbreviation_cache);
        ref_builder.feed(&self.act_reference)?;
        ref_builder.feed(&self.references)?;
        Ok(ref_builder.get_result())
    }
}

#[derive(Debug)]
pub struct OutgoingReferenceBuilder<'a> {
    ref_builder: ReferenceBuilder,
    abbreviation_cache: &'a AbbreviationCache,
    result: Vec<OutgoingReference>,
    start: Option<usize>,
    end: usize,
}

impl<'a> OutgoingReferenceBuilder<'a> {
    pub fn new(abbreviation_cache: &'a AbbreviationCache) -> Self {
        Self {
            ref_builder: ReferenceBuilder::new(),
            abbreviation_cache,
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

    pub fn take_result(&mut self) -> Vec<OutgoingReference> {
        std::mem::take(&mut self.result)
    }
}

pub trait FeedReferenceBuilder<T> {
    fn feed(&mut self, element: &T) -> Result<()>;
}

impl FeedReferenceBuilder<Vec<Reference>> for OutgoingReferenceBuilder<'_> {
    fn feed(&mut self, element: &Vec<Reference>) -> Result<()> {
        for reference in element {
            self.feed(reference)?;
        }
        Ok(())
    }
}

impl FeedReferenceBuilder<ActReference> for OutgoingReferenceBuilder<'_> {
    fn feed(&mut self, element: &ActReference) -> Result<()> {
        match element {
            ActReference::Abbreviation(abbrev) => {
                self.set_part(
                    abbrev.position.start,
                    abbrev.position.end,
                    self.abbreviation_cache.resolve(&abbrev.content)?,
                );
            }
            ActReference::ActIdWithFromNowOn(ActIdWithFromNowOn { act_id, .. }) => {
                self.set_part(
                    act_id.position.start,
                    act_id.position.end,
                    ActIdentifier::try_from(act_id)?,
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
    T: RefPartInGrammar,
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

impl FeedReferenceBuilder<InsertionReference> for OutgoingReferenceBuilder<'_> {
    fn feed(&mut self, element: &InsertionReference) -> Result<()> {
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

impl FeedReferenceBuilder<Vec<ReferenceWithIntroWrapup>> for OutgoingReferenceBuilder<'_> {
    fn feed(&mut self, element: &Vec<ReferenceWithIntroWrapup>) -> Result<()> {
        for part in element {
            self.feed(part)?
        }
        Ok(())
    }
}

impl FeedReferenceBuilder<ReferenceWithIntroWrapup> for OutgoingReferenceBuilder<'_> {
    fn feed(&mut self, element: &ReferenceWithIntroWrapup) -> Result<()> {
        self.feed(&element.reference)
    }
}

impl FeedReferenceBuilder<Vec<TextAmendmentReference>> for OutgoingReferenceBuilder<'_> {
    fn feed(&mut self, element: &Vec<TextAmendmentReference>) -> Result<()> {
        for part in element {
            self.feed(part)?
        }
        Ok(())
    }
}

impl FeedReferenceBuilder<TextAmendmentReference> for OutgoingReferenceBuilder<'_> {
    fn feed(&mut self, element: &TextAmendmentReference) -> Result<()> {
        match element {
            TextAmendmentReference::TextAmendmentStructuralReference(tasr) => match &tasr.child {
                TextAmendmentStructuralReference_child::AnyStructuralReference(_) => Ok(()),
                TextAmendmentStructuralReference_child::ArticleRelativePosition(arp) => {
                    self.feed(arp)
                }
            },
            TextAmendmentReference::ArticleTitleReference(x) => self.feed(x),
            TextAmendmentReference::SubtitlesReference(_) => Ok(()),
            TextAmendmentReference::ReferenceWithIntroWrapup(x) => self.feed(x),
        }
    }
}

impl FeedReferenceBuilder<ArticleTitleReference> for OutgoingReferenceBuilder<'_> {
    fn feed(&mut self, element: &ArticleTitleReference) -> Result<()> {
        self.feed(&element.article)
    }
}

trait RefPartInGrammar {}

macro_rules! impl_rcp {
    ($PartsT:ident, $RefPart:ident, $IdType:ident $(,)?) => {
        impl FeedReferenceBuilder<$PartsT> for OutgoingReferenceBuilder<'_> {
            fn feed(&mut self, element: &$PartsT) -> Result<()> {
                if let Some(id) = &element.id {
                    let part = $RefPart::from_single(id.parse::<$IdType>()?);
                    self.set_part(element.position.start, element.position.end, part);
                } else if let (Some(start), Some(end)) = (&element.start, &element.end) {
                    let part =
                        $RefPart::from_range(start.parse::<$IdType>()?, end.parse::<$IdType>()?);
                    self.set_part(element.position.start, element.position.end, part);
                } else {
                    bail!("Grammar somehow produced an invalid combination")
                }
                Ok(())
            }
        }

        impl RefPartInGrammar for $PartsT {}
    };
}

impl_rcp!(ParagraphReferencePart, RefPartParagraph, NumericIdentifier);
impl_rcp!(NumericPointReferencePart, RefPartPoint, NumericIdentifier);
impl_rcp!(
    AlphabeticPointReferencePart,
    RefPartPoint,
    AlphabeticIdentifier,
);
impl_rcp!(
    NumericSubpointReferencePart,
    RefPartSubpoint,
    NumericIdentifier,
);
impl_rcp!(
    AlphabeticSubpointReferencePart,
    RefPartSubpoint,
    PrefixedAlphabeticIdentifier,
);

impl TryFrom<&ArticleReferencePart> for RefPartArticle {
    type Error = anyhow::Error;

    fn try_from(element: &ArticleReferencePart) -> Result<Self, Self::Error> {
        let book = element.book.as_ref().map(|b| b.parse::<u8>()).transpose()?;
        if let Some(id) = &element.id_without_book {
            let article_id = ArticleIdentifier::from_book_and_id(book, id.parse()?);
            Ok(RefPartArticle::from_single(article_id))
        } else if let (Some(start), Some(end)) = (&element.start, &element.end) {
            let start_id = ArticleIdentifier::from_book_and_id(book, start.parse()?);
            let end_id = ArticleIdentifier::from_book_and_id(book, end.parse()?);
            Ok(RefPartArticle::from_range(start_id, end_id))
        } else {
            Err(anyhow!("Grammar somehow produced an invalid combination"))
        }
    }
}

impl FeedReferenceBuilder<ArticleReferencePart> for OutgoingReferenceBuilder<'_> {
    fn feed(&mut self, element: &ArticleReferencePart) -> Result<()> {
        self.set_part(
            element.position.start,
            element.position.end,
            RefPartArticle::try_from(element)?,
        );
        Ok(())
    }
}

impl RefPartInGrammar for ArticleReferencePart {}
impl RefPartInGrammar for Vec<ArticleReferencePart> {}

impl FeedReferenceBuilder<ArticleRelativePosition> for OutgoingReferenceBuilder<'_> {
    fn feed(&mut self, element: &ArticleRelativePosition) -> Result<()> {
        match element {
            ArticleRelativePosition::AfterArticle(x) => self.feed(x)?,
            ArticleRelativePosition::BeforeArticle(x) => self.feed(x)?,
        };
        Ok(())
    }
}

impl FeedReferenceBuilder<SingleArticleReference> for OutgoingReferenceBuilder<'_> {
    fn feed(&mut self, element: &SingleArticleReference) -> Result<()> {
        self.set_part(
            element.position.start,
            element.position.end,
            RefPartArticle::try_from(&element.part)?,
        );
        self.record_one()?;
        Ok(())
    }
}

impl FeedReferenceBuilder<ReferenceWithSubtitle> for OutgoingReferenceBuilder<'_> {
    fn feed(&mut self, element: &ReferenceWithSubtitle) -> Result<()> {
        self.feed(&element.article)
    }
}

impl TryFrom<&SingleArticleReference> for ArticleIdentifier {
    type Error = anyhow::Error;

    fn try_from(value: &SingleArticleReference) -> Result<Self, Self::Error> {
        // Only the first part of the range is parsed, as it's more of an insertion point
        // Maybe TODO?
        Ok(RefPartArticle::try_from(&value.part)?.first_in_range())
    }
}

impl TryFrom<&ActId> for ActIdentifier {
    type Error = anyhow::Error;

    fn try_from(act_id: &ActId) -> Result<Self, Self::Error> {
        Ok(ActIdentifier {
            year: act_id.year.parse()?,
            number: roman::from(&act_id.number).ok_or_else(|| {
                anyhow!("{} is not a valid suffixed roman numeral", act_id.number)
            })?,
        })
    }
}

pub fn convert_act_reference(
    abbreviation_cache: &AbbreviationCache,
    elem: &ActReference,
) -> Result<ActIdentifier> {
    match elem {
        ActReference::Abbreviation(abbrev) => abbreviation_cache.resolve(&abbrev.content),
        ActReference::ActIdWithFromNowOn(ActIdWithFromNowOn { act_id, .. }) => {
            ActIdentifier::try_from(act_id)
        }
    }
}
