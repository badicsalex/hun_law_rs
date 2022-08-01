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

use anyhow::{anyhow, bail, Result};
use lazy_regex::regex;

use crate::{
    identifier::{HungarianIdentifierChar, IsNextFrom, PrefixedAlphabeticIdentifier},
    structure::{
        AlphabeticPoint, AlphabeticSubpoint, NumericPoint, NumericSubpoint, Paragraph,
        ParagraphChildren, QuotedBlock, SAEBody, SAECommon,
    },
    util::{indentedline::IndentedLine, QuoteCheck},
};

#[derive(Debug, PartialEq)]
pub enum ParseWrapUp {
    Yes,
    No,
}

pub trait SAEParser {
    type SAE: Sized + SAECommon;

    /// Parse the header into and identifier, and return it, along with the rest of the first line
    fn parse_header(
        &self,
        line: &IndentedLine,
    ) -> Option<(<Self::SAE as SAECommon>::IdentifierType, IndentedLine)>;

    /// Try to extract the children of this type, assuming the first line is the header of at least
    /// one child type, and there are multiple children. If any of this is not true, fail.
    /// Expected to call [SAEParser::extract_multiple] for all possible children type
    fn try_extract_children(
        &self,
        identifier: &<Self::SAE as SAECommon>::IdentifierType,
        body: &[IndentedLine],
    ) -> Result<(<Self::SAE as SAECommon>::ChildrenType, Option<String>)>;

    /// Parse a single instance.
    fn parse(
        &self,
        identifier: <Self::SAE as SAECommon>::IdentifierType,
        body: &[IndentedLine],
    ) -> Result<Self::SAE> {
        let mut intro = String::new();
        let mut quote_checker = QuoteCheck::default();
        for i in 0..body.len() {
            quote_checker.update(&body[i])?;
            if !quote_checker.beginning_is_quoted {
                if let Ok((children, wrap_up)) = self.try_extract_children(&identifier, &body[i..])
                {
                    return Ok(<Self::SAE>::new(
                        identifier,
                        SAEBody::Children {
                            intro,
                            children,
                            wrap_up,
                        },
                    ));
                }
            }
            let line = &body[i];
            line.append_to(&mut intro);
        }
        quote_checker.check_end()?;
        Ok(<Self::SAE>::new(identifier, intro.into()))
    }

    /// Extract multiple instances from the text. Fails if the first line is not a header
    fn extract_multiple<T>(
        &self,
        lines: &[IndentedLine],
        parse_wrap_up: ParseWrapUp,
    ) -> Result<(T, Option<String>)>
    where
        T: From<Vec<Self::SAE>>,
    {
        let (mut identifier, first_line_rest) = self
            .parse_header(&lines[0])
            .ok_or(anyhow!("Invalid header"))?;
        if !identifier.is_first() {
            bail!("Parsed identifier was not first: {:?}", identifier);
        };
        let mut quote_checker = QuoteCheck::default();
        quote_checker.update(&first_line_rest)?;
        let mut result: Vec<Self::SAE> = Vec::new();
        let mut body: Vec<IndentedLine> = vec![first_line_rest];
        let header_indent = lines[0].indent();

        for line in &lines[1..] {
            quote_checker.update(line)?;
            let new_header = if !quote_checker.beginning_is_quoted {
                self.parse_and_check_header(&identifier, header_indent, line)
            } else {
                None
            };
            if let Some((new_identifier, rest)) = new_header {
                result.push(self.parse(identifier, &body)?);
                identifier = new_identifier;
                body = vec![rest];
            } else if !line.is_empty() {
                body.push(line.clone())
            }
        }
        quote_checker.check_end()?;
        let mut wrap_up = None;
        // This is a stupid heuristic: we hope line-broken points are indented, while
        // the wrapup will be at the same level as the headers.
        if parse_wrap_up == ParseWrapUp::Yes {
            if let Some(wrap_up_split) =
                body.iter().position(|l| l.indent_less_or_eq(header_indent))
            {
                let wrap_up_lines = body.split_off(wrap_up_split);
                wrap_up = Some(wrap_up_lines.into_iter().fold(String::new(), |mut s, l| {
                    l.append_to(&mut s);
                    s
                }))
            }
        }

        result.push(self.parse(identifier, &body)?);

        if result.len() < 2 {
            bail!("Not enough children could be parsed");
        }
        Ok((result.into(), wrap_up))
    }

    /// Parse the header line, and return it, along with the rest of the line.
    /// Checks indentation and identifier correctness.
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
        _identifier: &<Self::SAE as SAECommon>::IdentifierType,
        body: &[IndentedLine],
    ) -> Result<(<Self::SAE as SAECommon>::ChildrenType, Option<String>)> {
        QuotedBlockParser
            .extract_multiple(body)
            .or_else(|_| NumericPointParser.extract_multiple(body, ParseWrapUp::Yes))
            .or_else(|_| AlphabeticPointParser.extract_multiple(body, ParseWrapUp::Yes))
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
        _identifier: &<Self::SAE as SAECommon>::IdentifierType,
        body: &[IndentedLine],
    ) -> Result<(<Self::SAE as SAECommon>::ChildrenType, Option<String>)> {
        AlphabeticSubpointParser { prefix: None }.extract_multiple(body, ParseWrapUp::Yes)
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
        identifier: &<Self::SAE as SAECommon>::IdentifierType,
        body: &[IndentedLine],
    ) -> Result<(<Self::SAE as SAECommon>::ChildrenType, Option<String>)> {
        NumericSubpointParser
            .extract_multiple(body, ParseWrapUp::Yes)
            .or_else(|_| {
                AlphabeticSubpointParser {
                    prefix: Some(*identifier),
                }
                .extract_multiple(body, ParseWrapUp::Yes)
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
        _identifier: &<Self::SAE as SAECommon>::IdentifierType,
        _body: &[IndentedLine],
    ) -> Result<(<Self::SAE as SAECommon>::ChildrenType, Option<String>)> {
        Err(anyhow!("Subpoints can't have children"))
    }
}

pub struct AlphabeticSubpointParser {
    prefix: Option<HungarianIdentifierChar>,
}

impl SAEParser for AlphabeticSubpointParser {
    type SAE = AlphabeticSubpoint;
    fn parse_header(
        &self,
        line: &IndentedLine,
    ) -> Option<(<Self::SAE as SAECommon>::IdentifierType, IndentedLine)> {
        let (result, rest) =
            line.parse_header::<PrefixedAlphabeticIdentifier>(regex!("^([a-z]?[a-z])\\) +(.*)$"))?;
        if result.prefix_is(self.prefix) {
            Some((result, rest))
        } else {
            None
        }
    }

    fn try_extract_children(
        &self,
        _identifier: &<Self::SAE as SAECommon>::IdentifierType,
        _body: &[IndentedLine],
    ) -> Result<(<Self::SAE as SAECommon>::ChildrenType, Option<String>)> {
        Err(anyhow!("Subpoints can't have children"))
    }
}

pub struct QuotedBlockParser;

#[derive(Debug, PartialEq, Eq)]
enum QuotedBlockParseState {
    Start,
    QuotedBlock,
    WrapUp,
}

impl QuotedBlockParser {
    /// Extract multiple instances from the text. Fails if line does not start with quote
    pub fn extract_multiple(
        &self,
        lines: &[IndentedLine],
    ) -> Result<(ParagraphChildren, Option<String>)> {
        if lines.is_empty() || !lines[0].content().starts_with(['„', '“']) {
            bail!("Quoted block starting char not found")
        }

        let mut state = QuotedBlockParseState::Start;
        let mut blocks = Vec::new();
        let mut wrap_up = None;
        let mut quoted_lines = Vec::new();

        let mut quote_checker = QuoteCheck::default();
        for line in lines {
            quote_checker.update(line)?;
            match state {
                QuotedBlockParseState::Start => {
                    if !line.is_empty() {
                        if line.content().starts_with(['„', '“']) {
                            if line.content().ends_with('”') {
                                blocks.push(QuotedBlock {
                                    lines: vec![line.slice(1, Some(-1))],
                                })
                            } else {
                                quoted_lines = vec![line.slice(1, None)];
                                state = QuotedBlockParseState::QuotedBlock;
                            }
                        } else {
                            wrap_up = Some(line.content().to_string());
                            state = QuotedBlockParseState::WrapUp;
                        }
                    }
                }
                QuotedBlockParseState::QuotedBlock => {
                    if !line.is_empty()
                        && !quote_checker.end_is_quoted
                        && line.content().ends_with('”')
                    {
                        quoted_lines.push(line.slice(0, Some(-1)));
                        blocks.push(QuotedBlock {
                            lines: std::mem::take(&mut quoted_lines),
                        });
                        state = QuotedBlockParseState::Start;
                    } else {
                        // Note that this else also applies to EMPTY_LINEs
                        quoted_lines.push(line.clone());
                    }
                }
                QuotedBlockParseState::WrapUp => {
                    if let Some(wuc) = &mut wrap_up {
                        line.append_to(wuc);
                    } else {
                        // Should never happen, actually.
                        wrap_up = Some(line.content().to_string())
                    }
                }
            }
        }
        if state == QuotedBlockParseState::QuotedBlock {
            Err(anyhow!("Quoted block parser ended in invalid state"))
        } else if blocks.is_empty() {
            // This should be impossible, actually, because we start with a
            // starting quote the state is already checked, which means at
            // least one push should've been done.
            Err(anyhow!("Quoted block parser didn't find any blocks"))
        } else {
            Ok((blocks.into(), wrap_up))
        }
    }
}
