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

use lazy_regex::regex;

use crate::{
    structure::{
        AlphabeticPoint, AlphabeticPointChildren, AlphabeticSubpoint, IsNextFrom, NumericPoint,
        NumericPointChildren, NumericSubpoint, Paragraph, ParagraphChildren, SAEBody, SAECommon,
    },
    util::indentedline::IndentedLine,
};

pub trait SAEParser {
    type SAE: Sized + SAECommon;

    fn parse_header(
        &self,
        line: &IndentedLine,
    ) -> Option<(<Self::SAE as SAECommon>::IdentifierType, IndentedLine)>;
    fn try_extract_children(
        &self,
        body: &[IndentedLine],
    ) -> Option<(<Self::SAE as SAECommon>::ChildrenType, Option<String>)>;

    fn parse(
        &self,
        identifier: <Self::SAE as SAECommon>::IdentifierType,
        body: &[IndentedLine],
    ) -> Option<Self::SAE> {
        let mut intro = String::new();
        for i in 0..body.len() {
            if let Some((children, wrap_up)) = self.try_extract_children(&body[i..]) {
                return Some(<Self::SAE>::new(
                    identifier,
                    SAEBody::Children {
                        intro,
                        children,
                        wrap_up,
                    },
                ));
            }
            let line = &body[i];
            if !line.is_empty() {
                if !intro.is_empty() {
                    intro.push(' ');
                }
                intro.push_str(line.content());
            }
        }
        Some(<Self::SAE>::new(identifier, SAEBody::Text(intro)))
    }

    /// Extract multiple instances from the text. Fails if the first line is not a header
    fn extract_multiple<T, F>(
        &self,
        lines: &[IndentedLine],
        postprocess: F,
    ) -> Option<(T, Option<String>)>
    where
        F: FnOnce(Vec<Self::SAE>) -> T,
    {
        let (mut identifier, first_line_rest) = self.parse_header(&lines[0])?;
        if !identifier.is_first() {
            return None;
        };
        let mut result: Vec<Self::SAE> = Vec::new();
        let mut body: Vec<IndentedLine> = vec![first_line_rest];
        let expected_indent = lines[0].indent();

        for line in &lines[1..] {
            if let Some((new_identifier, rest)) =
                self.parse_and_check_header(&identifier, expected_indent, line)
            {
                result.push(self.parse(identifier, &body)?);
                identifier = new_identifier;
                body = vec![rest];
            } else if !line.is_empty() {
                body.push(line.clone())
            }
        }
        // TODO: Wrap-up
        result.push(self.parse(identifier, &body)?);

        if result.len() < 2 {
            return None;
        }
        Some((postprocess(result), None))
    }

    fn parse_and_check_header(
        &self,
        last_identifier: &<Self::SAE as SAECommon>::IdentifierType,
        expected_indent: f64,
        line: &IndentedLine,
    ) -> Option<(<Self::SAE as SAECommon>::IdentifierType, IndentedLine)> {
        if !line.indent_less_or_eq(expected_indent) {
            return None;
        }
        let (id, rest) = self.parse_header(line)?;
        if !id.is_next_from(last_identifier.clone()) {
            return None;
        }

        Some((id, rest))
    }
}

pub struct ParagraphParser;

impl SAEParser for ParagraphParser {
    type SAE = Paragraph;

    fn parse_header(
        &self,
        line: &IndentedLine,
    ) -> Option<(<Self::SAE as SAECommon>::IdentifierType, IndentedLine)> {
        let (id, rest) = line.parse_header(regex!("^\\(([0-9]+[a-z]?)\\) +(.*)$"))?;
        Some((Some(id), rest))
    }

    fn try_extract_children(
        &self,
        body: &[IndentedLine],
    ) -> Option<(<Self::SAE as SAECommon>::ChildrenType, Option<String>)> {
        NumericPointParser
            .extract_multiple(body, ParagraphChildren::NumericPoint)
            .or_else(|| {
                AlphabeticPointParser.extract_multiple(body, ParagraphChildren::AlphabeticPoint)
            })
    }
}

pub struct NumericPointParser;

impl SAEParser for NumericPointParser {
    type SAE = NumericPoint;

    fn parse_header(
        &self,
        line: &IndentedLine,
    ) -> Option<(<Self::SAE as SAECommon>::IdentifierType, IndentedLine)> {
        line.parse_header(regex!("^([0-9]+(/?[a-z])?)\\. +(.*)$"))
    }

    fn try_extract_children(
        &self,
        body: &[IndentedLine],
    ) -> Option<(<Self::SAE as SAECommon>::ChildrenType, Option<String>)> {
        AlphabeticSubpointParser.extract_multiple(body, NumericPointChildren::AlphabeticSubpoint)
    }
}

pub struct AlphabeticPointParser;

impl SAEParser for AlphabeticPointParser {
    type SAE = AlphabeticPoint;

    fn parse_header(
        &self,
        line: &IndentedLine,
    ) -> Option<(<Self::SAE as SAECommon>::IdentifierType, IndentedLine)> {
        line.parse_header(regex!("^([a-z]|cs|dz|gy|ly|ny|sz|ty)\\) +(.*)$"))
    }

    fn try_extract_children(
        &self,
        body: &[IndentedLine],
    ) -> Option<(<Self::SAE as SAECommon>::ChildrenType, Option<String>)> {
        NumericSubpointParser
            .extract_multiple(body, AlphabeticPointChildren::NumericSubpoint)
            .or_else(|| {
                AlphabeticSubpointParser
                    .extract_multiple(body, AlphabeticPointChildren::AlphabeticSubpoint)
            })
    }
}

pub struct NumericSubpointParser;

impl SAEParser for NumericSubpointParser {
    type SAE = NumericSubpoint;

    fn parse_header(
        &self,
        line: &IndentedLine,
    ) -> Option<(<Self::SAE as SAECommon>::IdentifierType, IndentedLine)> {
        line.parse_header(regex!("^([0-9]+(/?[a-z])?)\\. +(.*)$"))
    }

    fn try_extract_children(
        &self,
        _body: &[IndentedLine],
    ) -> Option<(<Self::SAE as SAECommon>::ChildrenType, Option<String>)> {
        /* Cannot have children :C */
        None
    }
}

pub struct AlphabeticSubpointParser;

impl SAEParser for AlphabeticSubpointParser {
    type SAE = AlphabeticSubpoint;
    fn parse_header(
        &self,
        line: &IndentedLine,
    ) -> Option<(<Self::SAE as SAECommon>::IdentifierType, IndentedLine)> {
        line.parse_header(regex!("^([a-z]|cs|dz|gy|ly|ny|sz|ty)\\) +(.*)$"))
    }

    fn try_extract_children(
        &self,
        _body: &[IndentedLine],
    ) -> Option<(<Self::SAE as SAECommon>::ChildrenType, Option<String>)> {
        /* Cannot have children :C */
        None
    }
}
