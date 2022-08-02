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
use hun_law_grammar::PegParser;

use self::{
    abbreviation::{get_new_abbreviations, AbbreviationCache},
    article_title_amendment::convert_article_title_amendment,
    block_amendment::{
        convert_block_amendment, convert_structural_block_amendment,
        convert_subtitle_block_amendment,
    },
    enforcement_date::convert_enforcement_date,
    reference::GetOutgoingReferences,
    repeal::{convert_repeal, convert_structural_repeal},
    text_amendment::convert_text_amendment,
};
use crate::{
    identifier::IsNextFrom,
    semantic_info::{OutgoingReference, SemanticInfo, SpecialPhrase},
    structure::{
        Act, ActChild, AlphabeticPointChildren, AlphabeticSubpointChildren, Article,
        NumericPointChildren, NumericSubpointChildren, Paragraph, ParagraphChildren, SAEBody,
        SubArticleElement,
    },
    util::IsDefault,
};

pub mod abbreviation;
pub mod article_title_amendment;
pub mod block_amendment;
pub mod enforcement_date;
pub mod reference;
pub mod repeal;
pub mod structural_reference;
pub mod text_amendment;

impl Act {
    pub fn add_semantic_info(self) -> Result<Self> {
        // TODO: If there's any Act-level semantic info caching, this is the place for that.
        let mut abbreviation_cache = AbbreviationCache::new();
        Ok(Self {
            children: self
                .children
                .into_iter()
                .map(|child| child.add_semantic_info(&mut abbreviation_cache))
                .collect::<Result<Vec<ActChild>>>()?,
            ..self
        })
    }
}

impl ActChild {
    pub fn add_semantic_info(self, abbreviation_cache: &mut AbbreviationCache) -> Result<Self> {
        match self {
            ActChild::Article(x) => Ok(ActChild::Article(x.add_semantic_info(abbreviation_cache)?)),
            ActChild::StructuralElement(_) | ActChild::Subtitle(_) => Ok(self),
        }
    }
}

impl Article {
    pub fn add_semantic_info(self, abbreviation_cache: &mut AbbreviationCache) -> Result<Self> {
        // TODO: If there's any Article-level semantic info caching, this is the place for that.
        Ok(Self {
            children: self
                .children
                .into_iter()
                .map(|p| p.add_semantic_info("", "", abbreviation_cache))
                .collect::<Result<Vec<Paragraph>>>()?,
            ..self
        })
    }
}

trait AddSemanticInfoSAE: Sized {
    fn add_semantic_info(
        self,
        prefix: &str,
        postfix: &str,
        abbreviation_cache: &mut AbbreviationCache,
    ) -> Result<Self>;
}

impl<IdentifierType, ChildrenType> AddSemanticInfoSAE
    for SubArticleElement<IdentifierType, ChildrenType>
where
    IdentifierType: IsNextFrom + IsDefault + Sized,
    ChildrenType: AddSemanticInfoSAE,
{
    fn add_semantic_info(
        self,
        prefix: &str,
        postfix: &str,
        abbreviation_cache: &mut AbbreviationCache,
    ) -> Result<Self> {
        Ok(match self.body {
            SAEBody::Text(body) => Self {
                semantic_info: Some(extract_semantic_info(
                    prefix,
                    &body,
                    postfix,
                    abbreviation_cache,
                )?),
                body: SAEBody::Text(body),
                ..self
            },
            SAEBody::Children {
                intro,
                children,
                wrap_up,
            } => {
                // First parse the intro of this element, because although we will
                // parse the same text when in context of the children, we throw away
                // the intro part of the result there.

                // TODO: Not using the children and postfix here is a huge hack. This code is reached for
                // Points and SubPoints, so most of the time these are partial sentences,
                // e.g
                // From now on
                //     a) things will change
                //     b) everything will be better.
                //
                // In this case, we hope that the string "From now on" can be parsed without
                // the second part of the sentence.
                let semantic_info = Some(extract_semantic_info(
                    prefix,
                    &intro,
                    "",
                    abbreviation_cache,
                )?);
                let new_prefix = format!("{}{} ", prefix, intro);
                let new_postfix = if let Some(wrap_up_contents) = &wrap_up {
                    format!(" {}{}", wrap_up_contents, postfix)
                } else {
                    postfix.to_owned()
                };
                let children =
                    children.add_semantic_info(&new_prefix, &new_postfix, abbreviation_cache)?;
                Self {
                    semantic_info,
                    body: SAEBody::Children {
                        intro,
                        children,
                        wrap_up,
                    },
                    ..self
                }
            }
        })
    }
}

impl<T: AddSemanticInfoSAE> AddSemanticInfoSAE for Vec<T> {
    fn add_semantic_info(
        self,
        prefix: &str,
        postfix: &str,
        abbreviation_cache: &mut AbbreviationCache,
    ) -> Result<Self> {
        self.into_iter()
            .map(|item| item.add_semantic_info(prefix, postfix, abbreviation_cache))
            .collect()
    }
}

impl AddSemanticInfoSAE for ParagraphChildren {
    fn add_semantic_info(
        self,
        prefix: &str,
        postfix: &str,
        abbreviation_cache: &mut AbbreviationCache,
    ) -> Result<Self> {
        match self {
            ParagraphChildren::AlphabeticPoint(x) => Ok(ParagraphChildren::AlphabeticPoint(
                x.add_semantic_info(prefix, postfix, abbreviation_cache)?,
            )),
            ParagraphChildren::NumericPoint(x) => Ok(ParagraphChildren::NumericPoint(
                x.add_semantic_info(prefix, postfix, abbreviation_cache)?,
            )),
            ParagraphChildren::QuotedBlock(_) | ParagraphChildren::BlockAmendment(_) => Ok(self),
        }
    }
}

impl AddSemanticInfoSAE for NumericPointChildren {
    fn add_semantic_info(
        self,
        prefix: &str,
        postfix: &str,
        abbreviation_cache: &mut AbbreviationCache,
    ) -> Result<Self> {
        match self {
            NumericPointChildren::AlphabeticSubpoint(x) => {
                Ok(NumericPointChildren::AlphabeticSubpoint(
                    x.add_semantic_info(prefix, postfix, abbreviation_cache)?,
                ))
            }
        }
    }
}

impl AddSemanticInfoSAE for AlphabeticPointChildren {
    fn add_semantic_info(
        self,
        prefix: &str,
        postfix: &str,
        abbreviation_cache: &mut AbbreviationCache,
    ) -> Result<Self> {
        match self {
            AlphabeticPointChildren::AlphabeticSubpoint(x) => {
                Ok(AlphabeticPointChildren::AlphabeticSubpoint(
                    x.add_semantic_info(prefix, postfix, abbreviation_cache)?,
                ))
            }
            AlphabeticPointChildren::NumericSubpoint(x) => {
                Ok(AlphabeticPointChildren::NumericSubpoint(
                    x.add_semantic_info(prefix, postfix, abbreviation_cache)?,
                ))
            }
        }
    }
}

impl AddSemanticInfoSAE for AlphabeticSubpointChildren {
    fn add_semantic_info(
        self,
        _prefix: &str,
        _postfix: &str,
        _abbreviation_cache: &mut AbbreviationCache,
    ) -> Result<Self> {
        // This is an empty enum, the function shall never run.
        match self {}
    }
}

impl AddSemanticInfoSAE for NumericSubpointChildren {
    fn add_semantic_info(
        self,
        _prefix: &str,
        _postfix: &str,
        _abbreviation_cache: &mut AbbreviationCache,
    ) -> Result<Self> {
        // This is an empty enum, the function shall never run.
        match self {}
    }
}

pub fn extract_semantic_info(
    prefix: &str,
    middle: &str,
    postfix: &str,
    abbreviation_cache: &mut AbbreviationCache,
) -> Result<SemanticInfo> {
    // TODO:
    // check for len(text) > 10000:
    // check for not any(s in text for s in (")", "§", "törvén", "hely", "hatály", "Hatály"))

    let s = assemble_to_be_parsed_text(prefix, middle, postfix);
    let parsed = hun_law_grammar::Root::parse(&s)?;
    let new_abbreviations = get_new_abbreviations(&parsed)?;
    abbreviation_cache.add_multiple(&new_abbreviations);
    let outgoing_references = parsed
        .get_outgoing_references(abbreviation_cache)?
        .into_iter()
        .filter_map(|oref| adjust_outgoing_reference(prefix.len(), s.len() - postfix.len(), oref))
        .collect();

    let special_phrase = extract_special_phrase(abbreviation_cache, &parsed)?;
    Ok(SemanticInfo {
        outgoing_references,
        new_abbreviations,
        special_phrase,
    })
}

fn assemble_to_be_parsed_text(prefix: &str, mut middle: &str, postfix: &str) -> String {
    // The order here matters, so as to handle the ", és"-style cases
    for junk_str in [" és", " valamint", " illetve", " vagy", ";", ","] {
        if let Some(new_middle) = middle.strip_suffix(junk_str) {
            middle = new_middle;
        }
    }
    if postfix.is_empty() {
        if middle.ends_with(['.', ':', '!', '?']) {
            format!("{}{}", prefix, middle)
        } else {
            format!("{}{}.", prefix, middle)
        }
    } else {
        format!("{}{}{}", prefix, middle, postfix)
    }
}

fn adjust_outgoing_reference(
    prefixlen: usize,
    textlen: usize,
    oref: OutgoingReference,
) -> Option<OutgoingReference> {
    // The end of the parsed reference is inside the target string
    // Checking for the end and not the beginning is important, because
    // we also want partial references to work here.
    if oref.end > prefixlen && oref.end <= textlen {
        Some(OutgoingReference {
            start: oref.start.saturating_sub(prefixlen),
            end: oref.end - prefixlen,
            reference: oref.reference,
        })
    } else {
        None
    }
}

pub fn extract_special_phrase(
    abbreviation_cache: &AbbreviationCache,
    root: &hun_law_grammar::Root,
) -> Result<Option<SpecialPhrase>> {
    Ok(match &root.content {
        hun_law_grammar::Root_content::ListOfSimpleExpressions(_) => None,

        hun_law_grammar::Root_content::ArticleTitleAmendment(x) => {
            Some(convert_article_title_amendment(abbreviation_cache, x)?.into())
        }
        hun_law_grammar::Root_content::BlockAmendment(x) => {
            Some(convert_block_amendment(abbreviation_cache, x)?.into())
        }
        hun_law_grammar::Root_content::BlockAmendmentStructural(x) => {
            Some(convert_structural_block_amendment(abbreviation_cache, x)?.into())
        }
        hun_law_grammar::Root_content::BlockAmendmentWithSubtitle(x) => {
            Some(convert_subtitle_block_amendment(abbreviation_cache, x)?.into())
        }
        hun_law_grammar::Root_content::EnforcementDate(x) => {
            Some(convert_enforcement_date(abbreviation_cache, x)?.into())
        }
        hun_law_grammar::Root_content::Repeal(x) => {
            Some(convert_repeal(abbreviation_cache, x)?.into())
        }
        hun_law_grammar::Root_content::StructuralRepeal(x) => {
            Some(convert_structural_repeal(abbreviation_cache, x)?.into())
        }
        hun_law_grammar::Root_content::TextAmendment(x) => {
            Some(convert_text_amendment(abbreviation_cache, x)?.into())
        }
    })
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_assemble_to_be_parsed_text() {
        assert_eq!(assemble_to_be_parsed_text("", "Xxx!", ""), "Xxx!");
        assert_eq!(assemble_to_be_parsed_text("A ", "b", " c."), "A b c.");
        assert_eq!(assemble_to_be_parsed_text("A ", "b", ""), "A b.");
        assert_eq!(assemble_to_be_parsed_text("A ", "b:", ""), "A b:");
        assert_eq!(
            assemble_to_be_parsed_text("A ", "b, és", " kell."),
            "A b kell."
        );
    }
}
