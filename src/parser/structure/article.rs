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

use regex::Regex;

use crate::{
    structure::{Article, Paragraph, SAEBody},
    util::indentedline::IndentedLine,
};

pub struct ArticleParserFactory {
    last_id: Option<String>,
    header_regex: Regex,
}

impl ArticleParserFactory {
    pub fn new() -> Self {
        Self {
            last_id: None,
            header_regex: Regex::new("^(([0-9]+:)?([0-9]+(/[A-Z])?))\\. ?§ +(.*)$").unwrap(),
        }
    }

    pub fn try_create_from_header(&mut self, line: &IndentedLine) -> Option<ArticleParser> {
        let line_content = line.content();
        let mut capture_locations = self.header_regex.capture_locations();
        let regex_match = self
            .header_regex
            .captures_read(&mut capture_locations, line_content);
        if regex_match.is_some() {
            let (identifier_from, identifier_to) = capture_locations.get(1).unwrap();
            let identifier = line_content[identifier_from..identifier_to].to_string();
            let (content_from, content_to) = capture_locations.get(5).unwrap();
            let contents = vec![line.slice_bytes(content_from, Some(content_to))];
            Some(ArticleParser {
                identifier,
                contents,
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct ArticleParser {
    identifier: String,
    contents: Vec<IndentedLine>,
}

impl ArticleParser {
    pub fn feed_line(&mut self, line: &IndentedLine) {
        if !line.is_empty() {
            self.contents.push(line.clone())
        }
        /* intentionally left blank */
    }
    pub fn finish(self) -> Article {
        Article {
            identifier: self.identifier,
            title: None,
            children: vec![Paragraph {
                identifier: "".to_string(),
                body: SAEBody::Text(
                    self.contents
                        .iter()
                        .map(|l| l.content())
                        .collect::<Vec<&str>>()
                        .join(" "),
                ),
            }],
        }
    }
}
