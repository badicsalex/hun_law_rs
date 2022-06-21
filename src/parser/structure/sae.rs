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
    structure::{AlphabeticPoint, IsNextFrom, NumericPoint, Paragraph, SAEBody, SAECommon},
    util::indentedline::IndentedLine,
};

pub trait SAEParser: SAECommon {
    fn parse_header(line: &IndentedLine) -> Option<(Self::IdentifierType, IndentedLine)>;
    fn try_extract_children(body: &[IndentedLine]) -> Option<(Self::ChildrenType, Option<String>)>;

    fn parse(identifier: Self::IdentifierType, body: &[IndentedLine]) -> Option<Self> {
        let mut intro = String::new();
        for i in 0..body.len() {
            if let Some((children, wrap_up)) = Self::try_extract_children(&body[i..]) {
                return Some(Self::new(
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
        Some(Self::new(identifier, SAEBody::Text(intro)))
    }

    /// Extract multiple instances from the text. Fails if the first line is not a header
    fn extract_multiple<T, F>(lines: &[IndentedLine], postprocess: F) -> Option<(T, Option<String>)>
    where
        F: FnOnce(Vec<Self>) -> T,
    {
        let (mut identifier, first_line_rest) = Self::parse_header(&lines[0])?;
        if !identifier.is_first() {
            return None;
        };
        let mut result: Vec<Self> = Vec::new();
        let mut body: Vec<IndentedLine> = vec![first_line_rest];
        let expected_indent = lines[0].indent();

        for line in &lines[1..] {
            if let Some((new_identifier, rest)) =
                Self::parse_and_check_header(&identifier, expected_indent, line)
            {
                result.push(Self::parse(identifier, &body)?);
                identifier = new_identifier;
                body = vec![rest];
            } else if !line.is_empty() {
                body.push(line.clone())
            }
        }
        result.push(Self::parse(identifier, &body)?);

        // TODO: Wrap-up
        // TODO: Check if multiple were extracted
        Some((postprocess(result), None))
    }

    fn parse_and_check_header(
        last_identifier: &Self::IdentifierType,
        expected_indent: f64,
        line: &IndentedLine,
    ) -> Option<(Self::IdentifierType, IndentedLine)> {
        if !line.indent_less_or_eq(expected_indent) {
            return None;
        }
        let (id, rest) = Self::parse_header(line)?;
        if !id.is_next_from(last_identifier.clone()) {
            return None;
        }

        Some((id, rest))
    }
}

impl SAEParser for Paragraph {
    fn parse_header(line: &IndentedLine) -> Option<(Self::IdentifierType, IndentedLine)> {
        let (id, rest) = line.parse_header(regex!("^\\(([0-9]+[a-z]?)\\) +(.*)$"))?;
        Some((Some(id), rest))
    }

    fn try_extract_children(
        _body: &[IndentedLine],
    ) -> Option<(Self::ChildrenType, Option<String>)> {
        None
        /* TODO:
        None.or_else(|| NumericPoint::extract_multiple(body, ParagraphChildren::NumericPoint))
            .or_else(|| AlphabeticPoint::extract_multiple(body, ParagraphChildren::AlphabeticPoint)) */
    }
}

impl SAEParser for NumericPoint {
    fn parse_header(line: &IndentedLine) -> Option<(Self::IdentifierType, IndentedLine)> {
        line.parse_header(regex!("^([0-9]+(/?[a-z])?)\\.  +(.*)$"))
    }

    fn try_extract_children(
        _body: &[IndentedLine],
    ) -> Option<(Self::ChildrenType, Option<String>)> {
        None
    }
}

impl SAEParser for AlphabeticPoint {
    fn parse_header(line: &IndentedLine) -> Option<(Self::IdentifierType, IndentedLine)> {
        line.parse_header(regex!("^([a-z]|cs|dz|gy|ly|ny|sz|ty)\\) +(.*)$"))
    }

    fn try_extract_children(
        _body: &[IndentedLine],
    ) -> Option<(Self::ChildrenType, Option<String>)> {
        None
    }
}
