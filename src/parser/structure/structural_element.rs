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
    structure::{NumericIdentifier, StructuralElement, StructuralElementType},
    util::indentedline::IndentedLine,
};

pub struct StructuralElementParserFactory {
    last_id: Option<String>,
    element_type: StructuralElementType,
    title_regex: Regex,
}

impl StructuralElementParserFactory {
    pub fn new(element_type: StructuralElementType) -> Self {
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

    pub fn try_create_from_header(
        &mut self,
        line: &IndentedLine,
    ) -> Option<StructuralElementParser> {
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
pub struct StructuralElementParser {
    identifier: NumericIdentifier,
    title: String,
    element_type: StructuralElementType,
}

impl StructuralElementParser {
    pub fn feed_line(&mut self, line: &IndentedLine) {
        if !line.is_empty() {
            if !self.title.is_empty() {
                self.title.push(' ');
            }
            self.title.push_str(line.content())
        }
    }
    pub fn finish(self) -> StructuralElement {
        StructuralElement {
            identifier: self.identifier,
            title: self.title,
            element_type: self.element_type,
        }
    }
}
