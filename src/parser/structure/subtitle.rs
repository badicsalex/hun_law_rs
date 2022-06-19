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
use lazy_regex::regex_captures;

use crate::{
    structure::{NumericIdentifier, Subtitle},
    util::indentedline::IndentedLine,
};

pub enum SubtitleParserFactory {}

impl SubtitleParserFactory {
    pub fn try_create_from_header(
        line: &IndentedLine,
        prev_line_is_empty: bool,
    ) -> Option<SubtitleParser> {
        if !line.is_bold() {
            None
        } else if let Some((_, identifier, _, title)) =
            regex_captures!("^([0-9]+(/[A-Z])?)\\. (.*)$", line.content())
        {
            Some(SubtitleParser {
                identifier: Some(identifier.parse().ok()?),
                title: title.to_string(),
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
pub struct SubtitleParser {
    identifier: Option<NumericIdentifier>,
    title: String,
}

impl SubtitleParser {
    pub fn feed_line(&mut self, line: &IndentedLine) {
        if !line.is_empty() {
            if !self.title.is_empty() {
                self.title.push(' ');
            }
            self.title.push_str(line.content())
        }
    }
    pub fn finish(self) -> Subtitle {
        Subtitle {
            identifier: self.identifier,
            title: self.title,
        }
    }
}
