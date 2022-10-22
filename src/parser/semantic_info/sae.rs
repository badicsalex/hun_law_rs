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

use anyhow::{Context, Result};
use hun_law_grammar::grammar_parse;

use super::{
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
    identifier::IdentifierCommon,
    reference::Reference,
    semantic_info::{OutgoingReference, SemanticInfo, SpecialPhrase},
    structure::{ChildrenCommon, SAEBody, SubArticleElement},
    util::walker::SAEVisitorMut,
};

#[derive(Debug)]
pub struct SemanticInfoAdder<'a> {
    // TODO: This does way mroe allocations than necessary
    prefix_stack: Vec<String>,
    postfix_stack: Vec<String>,
    abbreviation_cache: &'a mut AbbreviationCache,
}

impl<'a> SAEVisitorMut for SemanticInfoAdder<'a> {
    fn on_enter<IT: IdentifierCommon, CT: ChildrenCommon>(
        &mut self,
        _position: &Reference,
        element: &mut SubArticleElement<IT, CT>,
    ) -> Result<()> {
        // First parse the intro of this element, because although we will
        // parse the same text when in context of the children, we throw away
        // the intro part of the result there.

        // TODO: Not using the children here is a huge hack. This code is reached for
        // Points and SubPoints, so most of the time these are partial sentences,
        // e.g
        // From now on
        //     a) things will change
        //     b) everything will be better.
        //
        // In this case, we hope that the string "From now on" can be parsed without
        // the second part of the sentence.
        //
        // But we need to do something about references in the intro, so here we are
        match &element.body {
            SAEBody::Text(text) => {
                element.semantic_info = self.extract_semantic_info(text)?;
            }
            SAEBody::Children { intro, wrap_up, .. } => {
                element.semantic_info = self.extract_semantic_info(intro)?;
                if let Some(sp) = &element.semantic_info.special_phrase {
                    match sp {
                        // Only leave these in, the rest may not go into the intro section.
                        SpecialPhrase::BlockAmendment(_)
                        | SpecialPhrase::StructuralBlockAmendment(_) => (),
                        _ => element.semantic_info.special_phrase = None,
                    }
                }

                self.prefix_stack.push(format!("{}{intro} ", self.prefix()));
                self.postfix_stack
                    .push(if let Some(wrap_up_contents) = &wrap_up {
                        format!(" {wrap_up_contents}{}", self.postfix())
                    } else {
                        self.postfix().to_owned()
                    });
            }
        }
        Ok(())
    }

    fn on_exit<IT: IdentifierCommon, CT: ChildrenCommon>(
        &mut self,
        _position: &Reference,
        element: &mut SubArticleElement<IT, CT>,
    ) -> Result<()> {
        if let SAEBody::Children { .. } = element.body {
            self.prefix_stack.pop();
            self.postfix_stack.pop();
        }
        Ok(())
    }
}

impl<'a> SemanticInfoAdder<'a> {
    pub fn new(abbreviation_cache: &'a mut AbbreviationCache) -> SemanticInfoAdder<'a> {
        Self {
            prefix_stack: Vec::new(),
            postfix_stack: Vec::new(),
            abbreviation_cache,
        }
    }

    fn prefix(&self) -> &str {
        self.prefix_stack.last().map(|s| s as &str).unwrap_or("")
    }
    fn postfix(&self) -> &str {
        self.postfix_stack.last().map(|s| s as &str).unwrap_or("")
    }

    pub fn extract_semantic_info(&mut self, middle: &str) -> Result<SemanticInfo> {
        // TODO:
        // check for len(text) > 10000:
        // check for not any(s in text for s in (")", "§", "törvén", "hely", "hatály", "Hatály"))

        let s = assemble_to_be_parsed_text(self.prefix(), middle, self.postfix());
        let parsed = grammar_parse(&s)?;
        let new_abbreviations = get_new_abbreviations(&parsed)?;
        self.abbreviation_cache.add_multiple(&new_abbreviations);
        let outgoing_references = parsed
            .get_outgoing_references(self.abbreviation_cache)?
            .into_iter()
            .filter_map(|oref| {
                adjust_outgoing_reference(self.prefix().len(), s.len() - self.postfix().len(), oref)
            })
            .collect();

        let special_phrase = extract_special_phrase(self.abbreviation_cache, &parsed)
            .with_context(|| format!("Could not extract special phrase from '{s}'"))?;
        Ok(SemanticInfo {
            outgoing_references,
            new_abbreviations: new_abbreviations.into_iter().collect(),
            special_phrase,
        })
    }
}

fn assemble_to_be_parsed_text(prefix: &str, mut middle: &str, postfix: &str) -> String {
    // The order here matters, so as to handle the ", és"-style cases
    for junk_str in [
        " a",
        " és",
        " valamint",
        " illetve",
        " vagy",
        " továbbá",
        ";",
        ",",
    ] {
        if let Some(new_middle) = middle.strip_suffix(junk_str) {
            middle = new_middle;
        }
    }
    if postfix.is_empty() {
        if middle.ends_with(['.', ':', '!', '?']) {
            format!("{prefix}{middle}")
        } else {
            format!("{prefix}{middle}.")
        }
    } else {
        format!("{prefix}{middle}{postfix}")
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
            Some(convert_block_amendment(abbreviation_cache, x)?)
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
