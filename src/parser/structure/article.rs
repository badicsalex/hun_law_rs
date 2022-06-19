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
    structure::{Article, Paragraph, SAEBody},
    util::indentedline::IndentedLine,
};

pub struct ArticleParserFactory {
    last_id: Option<String>,
}

impl ArticleParserFactory {
    pub fn new() -> Self {
        Self { last_id: None }
    }

    pub fn try_create_from_header(&mut self, line: &IndentedLine) -> Option<ArticleParser> {
        let line_content = line.content();
        let header_regex = regex!("^(([0-9]+:)?([0-9]+(/[A-Z])?))\\. ?§ +(.*)$");
        let mut capture_locations = header_regex.capture_locations();
        let regex_match = header_regex.captures_read(&mut capture_locations, line_content);
        if regex_match.is_some() {
            let (identifier_from, identifier_to) = capture_locations.get(1).unwrap();
            let identifier = line_content[identifier_from..identifier_to].to_string();
            let (content_from, content_to) = capture_locations.get(5).unwrap();
            let contents = vec![line.slice_bytes(content_from, Some(content_to))];
            Some(ArticleParser {
                identifier,
                lines: contents,
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct ArticleParser {
    identifier: String,
    lines: Vec<IndentedLine>,
}

impl ArticleParser {
    pub fn feed_line(&mut self, line: &IndentedLine) {
        self.lines.push(line.clone())
    }
    pub fn finish(mut self) -> Result<Article> {
        let title = self.extract_title()?;
        Ok(Article {
            identifier: self.identifier,
            title,
            children: vec![Paragraph {
                identifier: "".to_string(),
                body: SAEBody::Text(
                    self.lines
                        .iter()
                        .filter(|l| !l.is_empty())
                        .map(|l| l.content())
                        .collect::<Vec<&str>>()
                        .join(" "),
                ),
            }],
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
