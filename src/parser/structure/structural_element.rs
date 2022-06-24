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
use anyhow::{anyhow, Result};
use lazy_regex::{regex, Regex};

use crate::{
    structure::{NumericIdentifier, StructuralElement, StructuralElementType},
    util::indentedline::IndentedLine,
    util::str_to_int_hun,
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
            identifier: self.parse_identifier(identifier_str).ok()?,
            title: String::new(),
            element_type: self.element_type.clone(),
        })
    }

    fn parse_identifier(&self, id: &str) -> Result<NumericIdentifier> {
        match self.element_type {
            StructuralElementType::Part { is_special: true } => {
                Self::parse_special_part_identifier(id)
            }
            StructuralElementType::Book | StructuralElementType::Part { is_special: false } => {
                str_to_int_hun(id)
                    .map(NumericIdentifier::from)
                    .ok_or_else(|| anyhow!("Invalid hungarian numeral {}", id))
            }
            StructuralElementType::Title | StructuralElementType::Chapter => {
                NumericIdentifier::from_roman(id)
            }
        }
    }

    fn parse_special_part_identifier(id: &str) -> Result<NumericIdentifier> {
        match id {
            "ÁLTALÁNOS" => Ok(1.into()),
            "KÜLÖNÖS" => Ok(2.into()),
            "ZÁRÓ" => Ok(3.into()),
            _ => Err(anyhow!("{} is not a special part", id)),
        }
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
