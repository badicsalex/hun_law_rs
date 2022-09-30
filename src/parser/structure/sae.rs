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

use std::fmt::Debug;

use anyhow::{anyhow, ensure, Result};
use lazy_regex::regex;

use super::{act::ParsingContext, quote::QuotedBlockParser};
use crate::{
    identifier::{
        AlphabeticIdentifier, HungarianIdentifierChar, IdentifierCommon, NumericIdentifier,
        PrefixedAlphabeticIdentifier,
    },
    structure::{
        AlphabeticPointChildren, AlphabeticSubpointChildren, ChildrenCommon, NumericPointChildren,
        NumericSubpointChildren, ParagraphChildren, SAEBody, SubArticleElement,
    },
    util::{indentedline::IndentedLine, QuoteCheck},
};

#[derive(Debug)]
pub struct SAEParseParams {
    pub parse_wrap_up: bool,
    pub check_children_count: bool,
    pub context: ParsingContext,
}

impl SAEParseParams {
    fn children_parsing_default(context: ParsingContext) -> Self {
        Self {
            parse_wrap_up: true,
            check_children_count: true,
            context,
        }
    }
}

pub trait SAEParser: Debug {
    type IdentifierType: IdentifierCommon;
    type ChildrenType: ChildrenCommon;

    /// Parse the header into and identifier, and return it, along with the rest of the first line
    fn parse_header(&self, line: &IndentedLine) -> Option<(Self::IdentifierType, IndentedLine)>;

    /// Try to extract the children of this type, assuming the first line is the header of at least
    /// one child type, and there are multiple children. If any of this is not true, fail.
    /// Expected to call [SAEParser::extract_multiple] for all possible children type
    fn try_extract_children(
        &self,
        identifier: &Self::IdentifierType,
        previous_nonempty_line: Option<&IndentedLine>,
        body: &[IndentedLine],
        params: &SAEParseParams,
    ) -> Result<(Self::ChildrenType, Option<String>)>;

    /// Parse a single instance.
    fn parse(
        &self,
        identifier: Self::IdentifierType,
        body: &[IndentedLine],
        context: ParsingContext,
    ) -> Result<SubArticleElement<Self::IdentifierType, Self::ChildrenType>> {
        let mut intro = String::new();
        let mut quote_checker = QuoteCheck::default();
        let mut previous_nonempty_line = None;
        for i in 0..body.len() {
            quote_checker.update(&body[i])?;
            if !quote_checker.beginning_is_quoted {
                if let Ok((children, wrap_up)) = self.try_extract_children(
                    &identifier,
                    previous_nonempty_line,
                    &body[i..],
                    &SAEParseParams::children_parsing_default(context),
                ) {
                    return Ok(
                        SubArticleElement::<Self::IdentifierType, Self::ChildrenType> {
                            identifier,
                            body: SAEBody::Children {
                                intro,
                                children,
                                wrap_up,
                            },
                            semantic_info: Default::default(),
                        },
                    );
                }
            }
            let line = &body[i];
            line.append_to(&mut intro);
            if !line.is_empty() {
                previous_nonempty_line = Some(line);
            }
        }
        quote_checker.check_end()?;
        Ok(
            SubArticleElement::<Self::IdentifierType, Self::ChildrenType> {
                identifier,
                body: intro.into(),
                semantic_info: Default::default(),
            },
        )
    }

    /// Extract multiple instances from the text. Fails if the first line is not a header
    fn extract_multiple<T>(
        &self,
        lines: &[IndentedLine],
        params: &SAEParseParams,
        expected_identifier: Option<Self::IdentifierType>,
    ) -> Result<(T, Option<String>)>
    where
        T: From<Vec<SubArticleElement<Self::IdentifierType, Self::ChildrenType>>>,
    {
        let (mut identifier, first_line_rest) = self
            .parse_header(&lines[0])
            .ok_or_else(|| anyhow!("Invalid header for {:?}: '{}'", self, lines[0].content()))?;
        if let Some(ei) = expected_identifier {
            ensure!(
                identifier == ei,
                "Parsed identifier is different than expected"
            )
        } else {
            ensure!(identifier.is_first(), "Parsed identifier was not first")
        }
        let mut quote_checker = QuoteCheck::default();
        quote_checker.update(&first_line_rest)?;
        let mut result: Vec<_> = Vec::new();
        let mut body: Vec<IndentedLine> = vec![first_line_rest];
        let header_indent = lines[0].indent();

        for line in &lines[1..] {
            quote_checker.update(line)?;
            let new_header = if !quote_checker.beginning_is_quoted {
                self.parse_and_check_header(identifier, header_indent, line)
            } else {
                None
            };
            if let Some((new_identifier, rest)) = new_header {
                result.push(self.parse(identifier, &body, params.context)?);
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
        if params.parse_wrap_up {
            let wrap_up_split = match params.context {
                ParsingContext::FullAct => {
                    // Indentation-driven split
                    body.iter().position(|l| l.indent_less_or_eq(header_indent))
                }
                ParsingContext::BlockAmendment => {
                    // We have no proper indentations in block amendments, so we try to
                    // find the split, based on right-justification:
                    // We assume that the wrap-up starts after the first non-justified line
                    // It's important to go from back to front, because of the following scenario,
                    // where all lines are non-justified:
                    //
                    // a) ...
                    // b) ...
                    // ba) ...
                    // bc) ...
                    // ...
                    //
                    // In this case, we would split off the subpoints from b), which is not intended.

                    body.iter()
                        .enumerate()
                        .rev()
                        .skip(1) // Last line can be justified or not, it's not going to be a split point.
                        .find(|(_, l)| !l.is_justified())
                        .map(|(i, _)| i + 1) // Split point is _after_ the found non-justified line
                }
            };
            if let Some(wrap_up_split) = wrap_up_split {
                let wrap_up_lines = body.split_off(wrap_up_split);
                wrap_up = Some(wrap_up_lines.into_iter().fold(String::new(), |mut s, l| {
                    l.append_to(&mut s);
                    s
                }))
            }
        }

        result.push(self.parse(identifier, &body, params.context)?);

        if params.check_children_count {
            ensure!(result.len() > 1, "Not enough children could be parsed");
        }
        Ok((result.into(), wrap_up))
    }

    /// Parse the header line, and return it, along with the rest of the line.
    /// Checks indentation and identifier correctness.
    fn parse_and_check_header(
        &self,
        last_identifier: Self::IdentifierType,
        expected_indent: f64,
        line: &IndentedLine,
    ) -> Option<(Self::IdentifierType, IndentedLine)> {
        if !line.indent_less_or_eq(expected_indent) {
            return None;
        }
        let (id, rest) = self.parse_header(line)?;
        if !id.is_next_from(last_identifier) {
            return None;
        }

        Some((id, rest))
    }
}

#[derive(Debug)]
pub struct ParagraphParser;

impl SAEParser for ParagraphParser {
    type IdentifierType = Option<NumericIdentifier>;
    type ChildrenType = ParagraphChildren;

    fn parse_header(&self, line: &IndentedLine) -> Option<(Self::IdentifierType, IndentedLine)> {
        let (id, rest) = line.parse_header(regex!("^\\(([0-9]+[a-z]?)\\) +(.*)$"))?;
        Some((Some(id), rest))
    }

    fn try_extract_children(
        &self,
        _identifier: &Self::IdentifierType,
        previous_nonempty_line: Option<&IndentedLine>,
        body: &[IndentedLine],
        params: &SAEParseParams,
    ) -> Result<(Self::ChildrenType, Option<String>)> {
        QuotedBlockParser
            .extract_multiple(previous_nonempty_line, body)
            .or_else(|_| NumericPointParser.extract_multiple(body, params, None))
            .or_else(|_| AlphabeticPointParser.extract_multiple(body, params, None))
    }
}

#[derive(Debug)]
pub struct NumericPointParser;

impl SAEParser for NumericPointParser {
    type IdentifierType = NumericIdentifier;
    type ChildrenType = NumericPointChildren;

    fn parse_header(&self, line: &IndentedLine) -> Option<(Self::IdentifierType, IndentedLine)> {
        line.parse_header(regex!("^([0-9]+(/?[a-z])?)\\. +(.*)$"))
    }

    fn try_extract_children(
        &self,
        _identifier: &Self::IdentifierType,
        _previous_nonempty_line: Option<&IndentedLine>,
        body: &[IndentedLine],
        params: &SAEParseParams,
    ) -> Result<(Self::ChildrenType, Option<String>)> {
        AlphabeticSubpointParser { prefix: None }.extract_multiple(body, params, None)
    }
}

#[derive(Debug)]
pub struct AlphabeticPointParser;

impl SAEParser for AlphabeticPointParser {
    type IdentifierType = AlphabeticIdentifier;
    type ChildrenType = AlphabeticPointChildren;

    fn parse_header(&self, line: &IndentedLine) -> Option<(Self::IdentifierType, IndentedLine)> {
        line.parse_header(regex!("^([a-z]|cs|dz|gy|ly|ny|sz|ty)\\) +(.*)$"))
    }

    fn try_extract_children(
        &self,
        identifier: &Self::IdentifierType,
        _previous_nonempty_line: Option<&IndentedLine>,
        body: &[IndentedLine],
        params: &SAEParseParams,
    ) -> Result<(Self::ChildrenType, Option<String>)> {
        NumericSubpointParser
            .extract_multiple(body, params, None)
            .or_else(|_| {
                AlphabeticSubpointParser {
                    prefix: Some(*identifier),
                }
                .extract_multiple(body, params, None)
            })
    }
}

#[derive(Debug)]
pub struct NumericSubpointParser;

impl SAEParser for NumericSubpointParser {
    type IdentifierType = NumericIdentifier;
    type ChildrenType = NumericSubpointChildren;

    fn parse_header(&self, line: &IndentedLine) -> Option<(Self::IdentifierType, IndentedLine)> {
        line.parse_header(regex!("^([0-9]+(/?[a-z])?)\\. +(.*)$"))
    }

    fn try_extract_children(
        &self,
        _identifier: &Self::IdentifierType,
        _previous_nonempty_line: Option<&IndentedLine>,
        _body: &[IndentedLine],
        _params: &SAEParseParams,
    ) -> Result<(Self::ChildrenType, Option<String>)> {
        Err(anyhow!("Subpoints can't have children"))
    }
}

#[derive(Debug)]
pub struct AlphabeticSubpointParser {
    pub prefix: Option<HungarianIdentifierChar>,
}

impl SAEParser for AlphabeticSubpointParser {
    type IdentifierType = PrefixedAlphabeticIdentifier;
    type ChildrenType = AlphabeticSubpointChildren;

    fn parse_header(&self, line: &IndentedLine) -> Option<(Self::IdentifierType, IndentedLine)> {
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
        _identifier: &Self::IdentifierType,
        _previous_nonempty_line: Option<&IndentedLine>,
        _body: &[IndentedLine],
        _params: &SAEParseParams,
    ) -> Result<(Self::ChildrenType, Option<String>)> {
        Err(anyhow!("Subpoints can't have children"))
    }
}
