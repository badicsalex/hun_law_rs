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

use anyhow::{anyhow, bail, ensure, Context, Result};

use super::{
    act::{parse_complex_body, ParsingContext},
    article::ArticleParserFactory,
    sae::{NumericPointParser, ParagraphParser, SAEParser},
    structural_element::{StructuralElementParser, StructuralElementParserFactory},
    subtitle::SubtitleParserFactory,
};
use crate::{
    identifier::{ArticleIdentifier, NumericIdentifier},
    parser::structure::sae::{
        AlphabeticPointParser, AlphabeticSubpointParser, NumericSubpointParser, SAEParseParams,
    },
    reference::{
        parts::{AnyReferencePart, RefPartPoint, RefPartSubpoint},
        structural::{StructuralReference, StructuralReferenceElement},
        Reference,
    },
    semantic_info::SpecialPhrase,
    structure::{
        Act, ActChild, Article, BlockAmendment, BlockAmendmentChildren, Paragraph,
        ParagraphChildren, SAEBody, StructuralBlockAmendment, StructuralElementType,
    },
    util::{debug::WithElemContext, indentedline::IndentedLine},
};

impl Act {
    pub fn convert_block_amendments(&mut self) -> Result<()> {
        self.articles_mut().try_for_each(|article| {
            article
                .convert_block_amendments()
                .with_elem_context("Could not convert block amendments", article)
        })
    }
}

impl Article {
    pub fn convert_block_amendments(&mut self) -> Result<()> {
        self.children.iter_mut().try_for_each(|paragraph| {
            paragraph
                .convert_block_amendments()
                .with_elem_context("Could not convert block amendments", paragraph)
        })
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
            if let Some(special_phrase) = &self.semantic_info.special_phrase {
                match special_phrase {
                    SpecialPhrase::BlockAmendment(ba) => {
                        *children = BlockAmendment {
                            intro: std::mem::take(&mut quoted_block.intro),
                            children: convert_simple_block_amendment(
                                &ba.position,
                                &quoted_block.lines,
                            )
                            .with_context(|| "Could not parse simple block amendment")?,
                            wrap_up: std::mem::take(&mut quoted_block.wrap_up),
                        }
                        .into()
                    }
                    SpecialPhrase::StructuralBlockAmendment(sba) => {
                        *children = StructuralBlockAmendment {
                            intro: std::mem::take(&mut quoted_block.intro),
                            children: convert_structural_block_amendment(
                                &sba.position,
                                &quoted_block.lines,
                            )
                            .with_context(|| "Could not parse structural block amendment")?,
                            wrap_up: std::mem::take(&mut quoted_block.wrap_up),
                        }
                        .into()
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

fn convert_structural_block_amendment(
    position: &StructuralReference,
    lines: &[IndentedLine],
) -> Result<Vec<ActChild>> {
    if let StructuralReferenceElement::Article(article_id) = position.structural_element {
        convert_articles(article_id.first_in_range(), lines)
    } else if position.title_only {
        convert_title_only_block_amendment(position, lines)
    } else {
        // TODO: Absolutely no checks on the result here, we are basically hoping for the best.
        Ok(parse_complex_body(lines, ParsingContext::BlockAmendment)?.1)
    }
}

fn convert_simple_block_amendment(
    position: &Reference,
    lines: &[IndentedLine],
) -> Result<BlockAmendmentChildren> {
    ensure!(!lines.is_empty());

    Ok(match position.get_last_part() {
        AnyReferencePart::Paragraph(id) => {
            ParagraphParser
                .extract_multiple(lines, &BA_PARSE_PARAMS, Some(id.first_in_range().into()))?
                .elements
        }
        AnyReferencePart::Point(RefPartPoint::Numeric(id)) => {
            NumericPointParser
                .extract_multiple(lines, &BA_PARSE_PARAMS, Some(id.first_in_range()))?
                .elements
        }
        AnyReferencePart::Point(RefPartPoint::Alphabetic(id)) => {
            AlphabeticPointParser
                .extract_multiple(lines, &BA_PARSE_PARAMS, Some(id.first_in_range()))?
                .elements
        }
        AnyReferencePart::Subpoint(RefPartSubpoint::Numeric(id)) => {
            NumericSubpointParser
                .extract_multiple(lines, &BA_PARSE_PARAMS, Some(id.first_in_range()))?
                .elements
        }
        AnyReferencePart::Subpoint(RefPartSubpoint::Alphabetic(id)) => {
            let prefix = id.first_in_range().get_prefix();
            AlphabeticSubpointParser { prefix }
                .extract_multiple(lines, &BA_PARSE_PARAMS, Some(id.first_in_range()))?
                .elements
        }
        AnyReferencePart::Article(_) | AnyReferencePart::Act(_) | AnyReferencePart::Empty => {
            bail!("Invalid reference type in phrase during BlockAmendment conversion: {position:?}")
        }
    })
}

fn convert_articles(first_id: ArticleIdentifier, lines: &[IndentedLine]) -> Result<Vec<ActChild>> {
    let mut factory = ArticleParserFactory::new(ParsingContext::BlockAmendment);
    let mut result = Vec::new();
    let mut parser = factory
        .try_create_from_header(&lines[0], Some(first_id))
        .with_context(|| {
            anyhow!("First line of amendment could not be parsed into an article header",)
        })?;
    for line in &lines[1..] {
        if let Ok(new_parser) = factory.try_create_from_header(line, None) {
            result.push(parser.finish()?.into());
            parser = new_parser;
        } else {
            parser.feed_line(line);
        }
    }
    result.push(parser.finish()?.into());
    Ok(result)
}

fn convert_title_only_block_amendment(
    position: &StructuralReference,
    lines: &[IndentedLine],
) -> Result<Vec<ActChild>> {
    let result: ActChild = match position.structural_element {
        StructuralReferenceElement::Part(identifier) => convert_title_only_se(
            identifier,
            StructuralElementType::Part { is_special: false },
            lines,
        ),
        StructuralReferenceElement::Title(identifier) => {
            convert_title_only_se(identifier, StructuralElementType::Title, lines)
        }
        StructuralReferenceElement::Chapter(identifier) => {
            convert_title_only_se(identifier, StructuralElementType::Chapter, lines)
        }
        StructuralReferenceElement::SubtitleId(_)
        | StructuralReferenceElement::SubtitleTitle(_)
        | StructuralReferenceElement::SubtitleAfterArticle(_)
        | StructuralReferenceElement::SubtitleBeforeArticle(_)
        | StructuralReferenceElement::SubtitleBeforeArticleInclusive(_) => {
            convert_subtitle_only_se(lines)?
        }
        StructuralReferenceElement::InPart(_)
        | StructuralReferenceElement::InTitle(_)
        | StructuralReferenceElement::InChapter(_)
        | StructuralReferenceElement::EndOfAct
        | StructuralReferenceElement::Article(_) => {
            bail!("Invalid strucutral reference for a title only amendment: {position:?}")
        }
    };
    Ok(vec![result])
}

fn convert_title_only_se(
    identifier: NumericIdentifier,
    element_type: StructuralElementType,
    lines: &[IndentedLine],
) -> ActChild {
    let mut parser = if let Some(parser) =
        StructuralElementParserFactory::new(element_type).try_create_from_header(&lines[0])
    {
        parser
    } else {
        let mut parser = StructuralElementParser {
            identifier,
            title: "".to_string(),
            element_type,
        };
        parser.feed_line(&lines[0]);
        parser
    };
    for line in &lines[1..] {
        parser.feed_line(line);
    }
    parser.finish().into()
}

fn convert_subtitle_only_se(lines: &[IndentedLine]) -> Result<ActChild> {
    let mut parser = SubtitleParserFactory::try_create_from_header(
        &lines[0],
        true,
        ParsingContext::BlockAmendment,
    )
    .ok_or_else(|| anyhow!("Title-only subtitle amendment could not be parsed into a subtitle"))?;

    for line in &lines[1..] {
        parser.feed_line(line);
    }
    Ok(parser.finish()?.into())
}

const BA_PARSE_PARAMS: SAEParseParams = SAEParseParams {
    parse_wrap_up: false,
    check_children_count: false,
    context: ParsingContext::BlockAmendment,
};
