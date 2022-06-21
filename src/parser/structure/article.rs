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

use crate::{
    structure::{Article, ArticleIdentifier, IsNextFrom, Paragraph},
    util::indentedline::IndentedLine,
};

use super::paragraph::ParagraphParser;

pub struct ArticleParserFactory {
    last_id: Option<ArticleIdentifier>,
    article_header_indent: Option<f64>,
}

impl ArticleParserFactory {
    pub fn new() -> Self {
        Self {
            last_id: None,
            article_header_indent: None,
        }
    }

    pub fn try_create_from_header(&mut self, line: &IndentedLine) -> Option<ArticleParser> {
        if let Some(expected_indent) = self.article_header_indent {
            if !line.indent_less_or_eq(expected_indent) {
                return None;
            }
        }

        let line_content = line.content();
        let header_regex = regex!("^(([0-9]+:)?([0-9]+(/[A-Z])?))\\. ?§ +(.*)$");
        let mut capture_locations = header_regex.capture_locations();

        // This is called for its side-effects, and the '?' is important.
        header_regex.captures_read(&mut capture_locations, line_content)?;

        let (identifier_from, identifier_to) = capture_locations.get(1).unwrap();
        let identifier: ArticleIdentifier = line_content[identifier_from..identifier_to]
            .to_string()
            .parse()
            .ok()?;

        if let Some(last_id) = self.last_id {
            if !identifier.is_next_from(last_id) {
                return None;
            }
        } else if !identifier.is_first() {
            return None;
        }

        let (content_from, content_to) = capture_locations.get(5).unwrap();
        let contents = vec![line.slice_bytes(content_from, Some(content_to))];

        self.last_id = Some(identifier);
        self.article_header_indent = Some(line.indent());
        Some(ArticleParser {
            identifier,
            lines: contents,
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
        Ok(Article {
            identifier: self.identifier,
            title,
            children: ParagraphParser::parse_article_body(&self.lines)?,
        })
    }

    fn extract_title(&mut self) -> Result<Option<String>> {
        if !self.lines[0].content().starts_with('[') {
            return Ok(None);
        };
        let mut title = self.lines.remove(0).content()[1..].to_string();
        while !title.ends_with(']') && !self.lines.is_empty() {
            title.push(' ');
            title.push_str(self.lines.remove(0).content());
        }
        if !title.ends_with(']') {
            Err(anyhow!("Could not find ']' for article title matching."))
        } else {
            title.pop();
            Ok(Some(title))
        }
    }
}
