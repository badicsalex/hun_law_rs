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

use anyhow::Result;
use lazy_regex::regex_captures;

use crate::{
    structure::{NumericIdentifier, Paragraph, SAEBody},
    util::indentedline::IndentedLine,
};

pub enum ParagraphParser {}

impl ParagraphParser {
    pub fn parse_article_body(lines: &[IndentedLine]) -> Result<Vec<Paragraph>> {
        assert!(!lines[0].is_empty());
        let first_line_parsed = Self::parse_header(&lines[0]);
        if first_line_parsed.is_none() {
            return Ok(vec![Paragraph {
                identifier: None,
                body: SAEBody::Text(
                    lines
                        .iter()
                        .filter(|l| !l.is_empty())
                        .map(|l| l.content())
                        .collect::<Vec<&str>>()
                        .join(" "),
                ),
            }]);
        }

        let mut result: Vec<Paragraph> = Vec::new();
        let (mut identifier, first_line_rest) = first_line_parsed.unwrap();
        let mut body = first_line_rest.to_string();
        for line in &lines[1..] {
            if let Some((new_identifier, rest)) = Self::parse_header(line) {
                result.push(Paragraph {
                    identifier: Some(identifier),
                    body: SAEBody::Text(body),
                });
                identifier = new_identifier;
                body = rest.to_string();
            } else if !line.is_empty() {
                body.push(' ');
                body.push_str(line.content());
            }
        }
        result.push(Paragraph {
            identifier: Some(identifier),
            body: SAEBody::Text(body),
        });

        Ok(result)
    }

    fn parse_header(line: &IndentedLine) -> Option<(NumericIdentifier, &str)> {
        let (_, id, rest) = regex_captures!("^\\(([0-9]+[a-z]?)\\) +(.*$)", line.content())?;
        Some((id.parse().ok()?, rest))
    }
}
