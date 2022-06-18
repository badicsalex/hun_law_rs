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
use anyhow::{bail, Result};

use crate::{
    parser::mk_act_section::ActRawText,
    structure::{Act, ActChild, StructuralElementType},
    util::indentedline::IndentedLine,
};

use super::{
    article::{ArticleParser, ArticleParserFactory},
    structural_element::{StructuralElementParser, StructuralElementParserFactory},
    subtitle::{SubtitleParser, SubtitleParserFactory},
};

pub fn parse_act_structure(raw_act: ActRawText) -> Result<Act> {
    let (preamble, children) = parse_act_body(&raw_act.body)?;
    Ok(Act {
        identifier: raw_act.identifier,
        subject: raw_act.subject,
        preamble,
        publication_date: raw_act.publication_date,
        children,
    })
}

fn parse_act_body(lines: &[IndentedLine]) -> Result<(String, Vec<ActChild>)> {
    let mut preamble = String::new();
    let mut children: Vec<ActChild> = Vec::new();
    let mut state = ParseState::Preamble;
    let mut se_parser_factories = [
        StructuralElementParserFactory::new(StructuralElementType::Book),
        StructuralElementParserFactory::new(StructuralElementType::Part { is_special: false }),
        StructuralElementParserFactory::new(StructuralElementType::Title),
        StructuralElementParserFactory::new(StructuralElementType::Chapter),
    ];
    let mut subtitle_parser_factory = SubtitleParserFactory::new();
    let mut article_parser_factory = ArticleParserFactory::new();

    let mut prev_line_is_empty = true;
    for line in lines {
        let new_state = se_parser_factories
            .iter_mut()
            .find_map(|fac| {
                fac.try_create_from_header(line)
                    .map(ParseState::StructuralElement)
            })
            .or_else(|| {
                subtitle_parser_factory
                    .try_create_from_header(line, prev_line_is_empty)
                    .map(ParseState::Subtitle)
            })
            .or_else(|| {
                article_parser_factory
                    .try_create_from_header(line)
                    .map(ParseState::Article)
            });
        if let Some(new_state) = new_state {
            match state {
                ParseState::Preamble => (),
                ParseState::Article(parser) => children.push(ActChild::Article(parser.finish()?)),
                ParseState::StructuralElement(parser) => {
                    children.push(ActChild::StructuralElement(parser.finish()))
                }
                ParseState::Subtitle(parser) => children.push(ActChild::Subtitle(parser.finish())),
            }
            state = new_state;
        } else {
            match &mut state {
                ParseState::Preamble => {
                    if !line.is_empty() {
                        if !preamble.is_empty() {
                            preamble.push(' ')
                        }
                        preamble.push_str(line.content());
                    }
                }
                ParseState::Article(parser) => parser.feed_line(line),
                ParseState::StructuralElement(parser) => parser.feed_line(line),
                ParseState::Subtitle(parser) => parser.feed_line(line),
            }
        }
        prev_line_is_empty = line.is_empty();
    }
    match state {
        ParseState::Preamble => bail!("Parsing ended with preamble state"),
        ParseState::Article(parser) => children.push(ActChild::Article(parser.finish()?)),
        ParseState::StructuralElement(parser) => {
            children.push(ActChild::StructuralElement(parser.finish()))
        }
        ParseState::Subtitle(parser) => children.push(ActChild::Subtitle(parser.finish())),
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
