// Copyright (C) 2022, Alex Badics
//
// This file is part of Hun-Law.
//
// Hun-law is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3 of the License.
//
// Hun-law is distributed in the hope that it will be useful,
// but WITHOUT ANY Wut even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Hun-law. If not, see <http://www.gnu.org/licenses/>.
use lazy_regex::{regex, Regex};

use crate::{
    identifier::NumericIdentifier,
    structure::{StructuralElement, StructuralElementType},
    util::indentedline::IndentedLine,
};

pub struct StructuralElementParserFactory {
    element_type: StructuralElementType,
}

impl StructuralElementParserFactory {
    pub fn new(element_type: StructuralElementType) -> Self {
        Self { element_type }
    }

    fn get_title_regex(&self) -> &'static Regex {
        match self.element_type {
            StructuralElementType::Book => regex!("^(.*) KÖNYV$"),
            StructuralElementType::Part { .. } => regex!("^(.*) RÉSZ$"),
            StructuralElementType::Title => regex!("^(.*)\\. CÍM$"),
            StructuralElementType::Chapter => regex!("^(?i)(.*)\\. fejezet$"),
        }
    }

    pub fn try_create_from_header(&self, line: &IndentedLine) -> Option<StructuralElementParser> {
        let identifier_str = self
            .get_title_regex()
            .captures(line.content())?
            .get(1)?
            .as_str();
        Some(StructuralElementParser {
            identifier: self.element_type.parse_identifier(identifier_str).ok()?,
            title: String::new(),
            element_type: self.element_type,
        })
    }
}

#[derive(Debug)]
pub struct StructuralElementParser {
    pub identifier: NumericIdentifier,
    pub title: String,
    pub element_type: StructuralElementType,
}

impl StructuralElementParser {
    pub fn feed_line(&mut self, line: &IndentedLine) {
        line.append_to(&mut self.title);
    }
    pub fn finish(self) -> StructuralElement {
        StructuralElement {
            identifier: self.identifier,
            title: self.title,
            element_type: self.element_type,
            last_change: None,
        }
    }
}
