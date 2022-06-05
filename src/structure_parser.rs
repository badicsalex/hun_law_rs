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
use anyhow::{Result, bail};

use crate::{
    mk_act_section_parser::ActRawText,
    structure::{Act, ActChild, Article, StructuralElement, StructuralElementType},
    util::indentedline::IndentedLine,
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
        StructuralElementParserFactory::new(StructuralElementType::Subtitle),
    ];
    let mut article_parser_factory = ArticleParserFactory::new();

    for line in lines {
        let new_state = se_parser_factories
            .iter_mut()
            .find_map(|fac| {
                fac.try_create_from_header(line)
                    .map(ParseState::StructuralElement)
            })
            .or_else(|| {
                article_parser_factory
                    .try_create_from_header(line)
                    .map(ParseState::Article)
            });
        if let Some(new_state) = new_state {
            match state {
                ParseState::Preamble => (),
                ParseState::Article(parser) => children.push(ActChild::Article(parser.finish())),
                ParseState::StructuralElement(parser) => {
                    children.push(ActChild::StructuralElement(parser.finish()))
                }
            }
            state = new_state;
        }
        match &mut state {
            ParseState::Preamble => preamble.push_str(line.content()),
            ParseState::Article(parser) => parser.feed_line(line),
            ParseState::StructuralElement(parser) => parser.feed_line(line),
        }
    }
    match state {
        ParseState::Preamble => bail!("Parsing ended with preamble state"),
        ParseState::Article(parser) => children.push(ActChild::Article(parser.finish())),
        ParseState::StructuralElement(parser) => {
            children.push(ActChild::StructuralElement(parser.finish()))
        }
    }
    Ok((preamble, children))
}

enum ParseState {
    Preamble,
    Article(ArticleParser),
    StructuralElement(StructuralElementParser),
}

struct StructuralElementParserFactory {
    last_id: Option<String>,
    element_type: StructuralElementType,
}

impl StructuralElementParserFactory {
    fn new(element_type: StructuralElementType) -> Self {
        Self {
            last_id: None,
            element_type,
        }
    }

    fn try_create_from_header(&mut self, line: &IndentedLine) -> Option<StructuralElementParser> {
        None
    }
}
struct StructuralElementParser {
    identifier: String,
    title: String,
    element_type: StructuralElementType,
}
impl StructuralElementParser {
    fn feed_line(&mut self, line: &IndentedLine) {
        self.title.push(' ');
        self.title.push_str(line.content())
    }
    fn finish(self) -> StructuralElement {
        StructuralElement {
            identifier: self.identifier,
            title: self.title,
            element_type: self.element_type,
        }
    }
}

struct ArticleParserFactory {
    last_id: Option<String>,
}

impl ArticleParserFactory {
    fn new() -> Self {
        Self { last_id: None }
    }

    fn try_create_from_header(&mut self, line: &IndentedLine) -> Option<ArticleParser> {
        None
    }
}

struct ArticleParser {}

impl ArticleParser {
    fn feed_line(&mut self, line: &IndentedLine) {
        todo!()
    }
    fn finish(self) -> Article {
        todo!()
    }
}
