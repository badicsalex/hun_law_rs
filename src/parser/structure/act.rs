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
use anyhow::{bail, ensure, Result};

use super::{
    article::{ArticleParser, ArticleParserFactory},
    structural_element::{StructuralElementParser, StructuralElementParserFactory},
    subtitle::{SubtitleParser, SubtitleParserFactory},
};
use crate::{
    parser::mk_act_section::ActRawText,
    structure::{Act, ActChild, StructuralElementType},
    util::{indentedline::IndentedLine, QuoteCheck},
};

pub fn parse_act_structure(raw_act: &ActRawText) -> Result<Act> {
    let (preamble, children) = parse_complex_body(&raw_act.body, ParsingContext::FullAct)?;
    Ok(Act {
        identifier: raw_act.identifier,
        subject: raw_act.subject.clone(),
        preamble,
        publication_date: raw_act.publication_date,
        contained_abbreviations: Default::default(),
        children,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsingContext {
    FullAct,
    BlockAmendment,
}

pub fn parse_complex_body(
    lines: &[IndentedLine],
    context: ParsingContext,
) -> Result<(String, Vec<ActChild>)> {
    let mut preamble = String::new();
    let mut children: Vec<ActChild> = Vec::new();
    let mut state = ParseState::Preamble;
    let se_parser_factories = [
        StructuralElementParserFactory::new(StructuralElementType::Book),
        StructuralElementParserFactory::new(StructuralElementType::Part { is_special: false }),
        StructuralElementParserFactory::new(StructuralElementType::Part { is_special: true }),
        StructuralElementParserFactory::new(StructuralElementType::Title),
        StructuralElementParserFactory::new(StructuralElementType::Chapter),
    ];
    let mut article_parser_factory = ArticleParserFactory::new(context);
    let mut quote_checker = QuoteCheck::default();
    let mut prev_line_is_empty = true;
    for line in lines {
        quote_checker.update(line)?;
        let new_state = if !quote_checker.beginning_is_quoted {
            se_parser_factories
                .iter()
                .find_map(|fac| {
                    fac.try_create_from_header(line)
                        .map(ParseState::StructuralElement)
                })
                .or_else(|| {
                    SubtitleParserFactory::try_create_from_header(line, prev_line_is_empty, context)
                        .map(ParseState::Subtitle)
                })
                .or_else(|| {
                    article_parser_factory
                        .try_create_from_header(line, None)
                        .ok()
                        .map(ParseState::Article)
                })
        } else {
            None
        };
        if let Some(new_state) = new_state {
            match state {
                ParseState::Preamble => (),
                ParseState::Article(parser) => children.push(parser.finish()?.into()),
                ParseState::StructuralElement(parser) => children.push(parser.finish().into()),
                ParseState::Subtitle(parser) => children.push(parser.finish()?.into()),
            }
            state = new_state;
        } else {
            match &mut state {
                ParseState::Preamble => line.append_to(&mut preamble),
                ParseState::Article(parser) => parser.feed_line(line),
                ParseState::StructuralElement(parser) => parser.feed_line(line),
                ParseState::Subtitle(parser) => parser.feed_line(line),
            }
        }
        prev_line_is_empty = line.is_empty();
    }
    quote_checker.check_end()?;

    match state {
        ParseState::Preamble => bail!("Parsing ended with preamble state"),
        ParseState::Article(parser) => children.push(parser.finish()?.into()),
        ParseState::StructuralElement(parser) => children.push(parser.finish().into()),
        ParseState::Subtitle(parser) => children.push(parser.finish()?.into()),
    }

    if context != ParsingContext::FullAct {
        ensure!(
            preamble.is_empty(),
            "Junk detected at the beginning of a complex body"
        );
    }

    for child in &children {
        if let ActChild::Subtitle(st) = child {
            // 2011. évi CCI. törvény has a legit 1135 character subtitle.
            if st.title.len() > 1500 {
                bail!(
                    "Probable corrupted read: way too long ({:?}) subtitle title detected (Last article id: {:?}): {}...",
                    st.title.len(),
                    article_parser_factory.last_id,
                    st.title.chars().take(100).collect::<String>()
                )
            }
        }
    }

    Ok((preamble, children))
}

#[derive(Debug)]
enum ParseState {
    Preamble,
    Article(ArticleParser),
    Subtitle(SubtitleParser),
    StructuralElement(StructuralElementParser),
}
