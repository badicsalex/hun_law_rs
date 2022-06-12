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
use regex::Regex;

use crate::{
    mk_act_section_parser::ActRawText,
    structure::{
        Act, ActChild, Article, NumericIdentifier, Paragraph, SAEBody, StructuralElement,
        StructuralElementType, Subtitle,
    },
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
                ParseState::Article(parser) => children.push(ActChild::Article(parser.finish())),
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
        ParseState::Article(parser) => children.push(ActChild::Article(parser.finish())),
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

struct StructuralElementParserFactory {
    last_id: Option<String>,
    element_type: StructuralElementType,
    title_regex: Regex,
}

impl StructuralElementParserFactory {
    fn new(element_type: StructuralElementType) -> Self {
        Self {
            last_id: None,
            element_type: element_type.clone(),
            title_regex: Self::create_title_regex(element_type),
        }
    }

    fn create_title_regex(element_type: StructuralElementType) -> Regex {
        Regex::new(match element_type {
            StructuralElementType::Book => "^(.*) KÖNYV$",
            StructuralElementType::Part { .. } => "^(.*) RÉSZ$",
            StructuralElementType::Title => "^(.*)\\. CÍM$",
            StructuralElementType::Chapter => "^(?i)(.*)\\. fejezet$",
        })
        .unwrap()
    }

    fn try_create_from_header(&mut self, line: &IndentedLine) -> Option<StructuralElementParser> {
        Some(StructuralElementParser {
            identifier: self
                .title_regex
                .captures(line.content())?
                .get(1)?
                .as_str()
                .parse()
                .ok()?,
            title: String::new(),
            element_type: self.element_type.clone(),
        })
    }
}
#[derive(Debug)]
struct StructuralElementParser {
    identifier: NumericIdentifier,
    title: String,
    element_type: StructuralElementType,
}
impl StructuralElementParser {
    fn feed_line(&mut self, line: &IndentedLine) {
        if !line.is_empty() {
            if !self.title.is_empty() {
                self.title.push(' ');
            }
            self.title.push_str(line.content())
        }
    }
    fn finish(self) -> StructuralElement {
        StructuralElement {
            identifier: self.identifier,
            title: self.title,
            element_type: self.element_type,
        }
    }
}

struct SubtitleParserFactory {
    last_id: Option<String>,
    title_regex: Regex,
}

impl SubtitleParserFactory {
    fn new() -> Self {
        Self {
            last_id: None,
            title_regex: Regex::new("^([0-9]+(/[A-Z])?)\\. (.*)$").unwrap(),
        }
    }
    fn try_create_from_header(
        &mut self,
        line: &IndentedLine,
        prev_line_is_empty: bool,
    ) -> Option<SubtitleParser> {
        if !line.is_bold() {
            None
        } else if let Some(captures) = self.title_regex.captures(line.content()) {
            Some(SubtitleParser {
                identifier: Some(captures.get(1).unwrap().as_str().parse().ok()?),
                title: captures.get(3).unwrap().as_str().to_string(),
            })
        } else if prev_line_is_empty && line.content().chars().next()?.is_uppercase() {
            Some(SubtitleParser {
                identifier: None,
                title: line.content().to_string(),
            })
        } else {
            None
        }
    }
}
#[derive(Debug)]
struct SubtitleParser {
    identifier: Option<NumericIdentifier>,
    title: String,
}
impl SubtitleParser {
    fn feed_line(&mut self, line: &IndentedLine) {
        if !line.is_empty() {
            if !self.title.is_empty() {
                self.title.push(' ');
            }
            self.title.push_str(line.content())
        }
    }
    fn finish(self) -> Subtitle {
        Subtitle {
            identifier: self.identifier,
            title: self.title,
        }
    }
}
struct ArticleParserFactory {
    last_id: Option<String>,
    header_regex: Regex,
}

impl ArticleParserFactory {
    fn new() -> Self {
        Self {
            last_id: None,
            header_regex: Regex::new("^(([0-9]+:)?([0-9]+(/[A-Z])?))\\. ?§ +(.*)$").unwrap(),
        }
    }

    fn try_create_from_header(&mut self, line: &IndentedLine) -> Option<ArticleParser> {
        if let Some(captures) = self.header_regex.captures(line.content()) {
            println!("{:?}", captures);
            Some(ArticleParser {
                identifier: captures.get(1).unwrap().as_str().to_string(),
                contents: captures.get(5).unwrap().as_str().to_string(),
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct ArticleParser {
    identifier: String,
    contents: String,
}

impl ArticleParser {
    fn feed_line(&mut self, line: &IndentedLine) {
        if !line.is_empty() {
            if !self.contents.is_empty() {
                self.contents.push(' ');
            }
            self.contents.push_str(line.content())
        }
        /* intentionally left blank */
    }
    fn finish(self) -> Article {
        Article {
            identifier: self.identifier,
            title: None,
            children: vec![Paragraph {
                identifier: "".to_string(),
                body: SAEBody::Text(self.contents),
            }],
        }
    }
}
