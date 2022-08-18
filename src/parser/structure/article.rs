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
use lazy_regex::regex;

use super::{
    act::ParsingContext,
    sae::{ParagraphParser, SAEParseParams, SAEParser},
};
use crate::{
    identifier::{ArticleIdentifier, IsNextFrom},
    structure::{Article, Paragraph},
    util::indentedline::IndentedLine,
};

pub struct ArticleParserFactory {
    last_id: Option<ArticleIdentifier>,
    article_header_indent: Option<f64>,
    context: ParsingContext,
}

impl ArticleParserFactory {
    pub fn new(context: ParsingContext) -> Self {
        Self {
            last_id: None,
            article_header_indent: None,
            context,
        }
    }

    pub fn try_create_from_header(
        &mut self,
        line: &IndentedLine,
        expected_identifier: Option<ArticleIdentifier>,
    ) -> Option<ArticleParser> {
        if let Some(expected_indent) = self.article_header_indent {
            if !line.indent_less_or_eq(expected_indent) {
                return None;
            }
        }

        let (identifier, rest) = line.parse_header::<ArticleIdentifier>(regex!(
            "^(([0-9]+:)?([0-9]+(/[A-Z])?))\\. ?§ +(.*)$"
        ))?;

        if let Some(expected_id) = expected_identifier {
            if expected_id != identifier {
                return None;
            }
        } else if let Some(last_id) = self.last_id {
            if !identifier.is_next_from(last_id) {
                return None;
            }
        } else if self.context == ParsingContext::FullAct && !identifier.is_first() {
            return None;
        }

        self.last_id = Some(identifier);
        self.article_header_indent = Some(line.indent());
        Some(ArticleParser {
            identifier,
            lines: vec![rest],
        })
    }
}

#[derive(Debug)]
pub struct ArticleParser {
    identifier: ArticleIdentifier,
    lines: Vec<IndentedLine>,
}

impl ArticleParser {
    pub fn feed_line(&mut self, line: &IndentedLine) {
        self.lines.push(line.clone())
    }
    pub fn finish(mut self) -> Result<Article> {
        let title = self.extract_title()?;

        // Pathological case where there is an empty line between the article title
        // and the actual content. Very very rare, basically only happens in an
        // amendment in 2013. évi CCLII. törvény 185. § (18)
        // There can only be at most 1 consecutive EMPTY_LINE because of previous
        // preprocessing in the PDF extractor.
        if self.lines[0].is_empty() {
            self.lines.remove(0);
        }
        let children: Vec<Paragraph> = if let Ok((extracted, wrap_up)) = ParagraphParser
            .extract_multiple(
                &self.lines,
                SAEParseParams {
                    check_children_count: true,
                    ..Default::default()
                },
            ) {
            assert!(wrap_up.is_none());
            extracted
        } else {
            vec![ParagraphParser.parse(None, &self.lines)?]
        };
        Ok(Article {
            identifier: self.identifier,
            title,
            children,
        })
    }

    fn extract_title(&mut self) -> Result<Option<String>> {
        if !self.lines[0].content().starts_with('[') {
            return Ok(None);
        };
        let mut title = self.lines.remove(0).content()[1..].to_string();
        while !title.ends_with(']') && !self.lines.is_empty() {
            self.lines.remove(0).append_to(&mut title)
        }
        if !title.ends_with(']') {
            Err(anyhow!("Could not find ']' for article title matching."))
        } else {
            title.pop();
            Ok(Some(title))
        }
    }
}
