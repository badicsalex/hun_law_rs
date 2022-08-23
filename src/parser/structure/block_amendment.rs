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

use std::fmt::Debug;

use anyhow::{anyhow, bail, ensure, Context, Result};

use crate::{
    identifier::{ArticleIdentifier, NumericIdentifier},
    parser::structure::sae::{AlphabeticPointParser, AlphabeticSubpointParser, SAEParseParams},
    reference::{AnyReferencePart, IdentifierRange, RefPartPoint, RefPartSubpoint, Reference},
    semantic_info::{SemanticInfo, SpecialPhrase},
    structure::{
        Act, ActChild, Article, BlockAmendment, BlockAmendmentChildren, Paragraph,
        ParagraphChildren, SAEBody,
    },
    util::{debug::WithElemContext, indentedline::IndentedLine},
};

use super::{
    act::{parse_complex_body, ParsingContext},
    article::ArticleParserFactory,
    sae::{NumericPointParser, ParagraphParser, SAEParser},
};

impl Act {
    pub fn convert_block_amendments(&mut self) -> Result<()> {
        for act_child in &mut self.children {
            if let ActChild::Article(article) = act_child {
                article
                    .convert_block_amendments()
                    .with_elem_context("Could not convert block amendments", article)?;
            }
        }
        Ok(())
    }
}

impl Article {
    pub fn convert_block_amendments(&mut self) -> Result<()> {
        for paragraph in &mut self.children {
            paragraph
                .convert_block_amendments()
                .with_elem_context("Could not convert block amendments", paragraph)?;
        }
        Ok(())
    }
}

impl Paragraph {
    pub fn convert_block_amendments(&mut self) -> Result<()> {
        let children = if let SAEBody::Children { children, .. } = &mut self.body {
            children
        } else {
            return Ok(());
        };

        if let ParagraphChildren::QuotedBlock(qbs) = children {
            if qbs.len() != 1 {
                return Ok(());
            }
            let quoted_block = &mut qbs[0];
            if let Some(SemanticInfo {
                special_phrase: Some(special_phrase),
                ..
            }) = &self.semantic_info
            {
                match special_phrase {
                    SpecialPhrase::BlockAmendment(ba) => {
                        *children = ParagraphChildren::BlockAmendment(BlockAmendment {
                            intro: std::mem::take(&mut quoted_block.intro),
                            children: convert_simple_block_amendment(
                                &ba.position,
                                &quoted_block.lines,
                            )
                            .with_context(|| "Could not parse simple block amendment")?,
                            wrap_up: std::mem::take(&mut quoted_block.wrap_up),
                        })
                    }
                    SpecialPhrase::StructuralBlockAmendment(_) => {
                        *children = ParagraphChildren::BlockAmendment(BlockAmendment {
                            intro: std::mem::take(&mut quoted_block.intro),
                            children: convert_structural_block_amendment(&quoted_block.lines)
                                .with_context(|| "Could not parse structural block amendment")?,
                            wrap_up: std::mem::take(&mut quoted_block.wrap_up),
                        })
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

fn convert_structural_block_amendment(lines: &[IndentedLine]) -> Result<BlockAmendmentChildren> {
    // TODO: Absolutely no checks on the result here, we are basically hoping for the best.
    Ok(parse_complex_body(lines, ParsingContext::BlockAmendment)?
        .1
        .into())
}

fn convert_simple_block_amendment(
    position: &Reference,
    lines: &[IndentedLine],
) -> Result<BlockAmendmentChildren> {
    ensure!(!lines.is_empty());

    Ok(match position.get_last_part() {
        Some(AnyReferencePart::Article(article_id)) => {
            convert_articles(article_id.first_in_range(), lines)?.into()
        }
        Some(AnyReferencePart::Paragraph(id)) => {
            ParagraphParser
                .extract_multiple(lines, create_parse_params_paragraph(id))?
                .0
        }
        Some(AnyReferencePart::Point(RefPartPoint::Numeric(id))) => {
            NumericPointParser
                .extract_multiple(lines, create_parse_params(id))?
                .0
        }
        Some(AnyReferencePart::Point(RefPartPoint::Alphabetic(id))) => {
            AlphabeticPointParser
                .extract_multiple(lines, create_parse_params(id))?
                .0
        }
        Some(AnyReferencePart::Subpoint(RefPartSubpoint::Numeric(id))) => {
            NumericPointParser
                .extract_multiple(lines, create_parse_params(id))?
                .0
        }
        Some(AnyReferencePart::Subpoint(RefPartSubpoint::Alphabetic(id))) => {
            let prefix = id.first_in_range().get_prefix();
            AlphabeticSubpointParser { prefix }
                .extract_multiple(lines, create_parse_params(id))?
                .0
        }
        Some(AnyReferencePart::Act(_)) | None => bail!(
            "Invalid reference type in phrase during BlockAmendment conversion: {:?}",
            position
        ),
    })
}

fn convert_articles(first_id: ArticleIdentifier, lines: &[IndentedLine]) -> Result<Vec<Article>> {
    let mut factory = ArticleParserFactory::new(ParsingContext::BlockAmendment);
    let mut result = Vec::new();
    let mut parser = factory
        .try_create_from_header(&lines[0], Some(first_id))
        .ok_or_else(|| anyhow!("First line could not be parsed into an article header"))?;
    for line in &lines[1..] {
        if let Some(new_parser) = factory.try_create_from_header(line, None) {
            result.push(parser.finish()?);
            parser = new_parser;
        } else {
            parser.feed_line(line);
        }
    }
    result.push(parser.finish()?);
    Ok(result)
}

fn create_parse_params_paragraph(
    id: IdentifierRange<NumericIdentifier>,
) -> SAEParseParams<Option<NumericIdentifier>> {
    SAEParseParams {
        expected_identifier: Some(Some(id.first_in_range())),
        parse_wrap_up: false,
        check_children_count: false,
        context: ParsingContext::BlockAmendment,
    }
}

fn create_parse_params<T>(id: IdentifierRange<T>) -> SAEParseParams<T>
where
    T: Eq + Debug + Clone,
{
    SAEParseParams {
        expected_identifier: Some(id.first_in_range()),
        parse_wrap_up: false,
        check_children_count: false,
        context: ParsingContext::BlockAmendment,
    }
}
