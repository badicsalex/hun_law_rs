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

use anyhow::{bail, Result};

use crate::{
    structure::{ParagraphChildren, QuotedBlock},
    util::{indentedline::IndentedLine, QuoteCheck},
};

pub struct QuotedBlockParser;

#[derive(Debug, PartialEq, Eq)]
enum QuotedBlockParseState {
    Start,
    QuotedBlockIntro,
    WaitingForQuotedBlock,
    QuotedBlock,
    QuotedBlockWrapUp,
    WrapUp,
}

impl QuotedBlockParser {
    /// Extract multiple instances from the text. Fails if line does not start with quote
    pub fn extract_multiple(
        &self,
        lines: &[IndentedLine],
    ) -> Result<(ParagraphChildren, Option<String>)> {
        let mut state = QuotedBlockParseState::Start;
        let mut blocks = Vec::new();
        let mut wrap_up = String::new();

        let mut quoted_block_intro = String::new();
        let mut quoted_block_wrap_up = String::new();
        let mut quoted_lines = Vec::new();

        let mut quote_checker = QuoteCheck::default();
        for line in lines {
            quote_checker.update(line)?;
            match state {
                QuotedBlockParseState::Start => {
                    if line.content().starts_with(['„', '“']) {
                        if line.content().ends_with('”') {
                            blocks.push(QuotedBlock {
                                intro: None,
                                lines: vec![line.slice(1, Some(-1))],
                                wrap_up: None,
                            });
                            state = QuotedBlockParseState::WaitingForQuotedBlock;
                        } else {
                            quoted_lines = vec![line.slice(1, None)];
                            state = QuotedBlockParseState::QuotedBlock;
                        }
                    } else if line.content().starts_with(['(', '[']) {
                        if line.content().ends_with([')', ']']) {
                            quoted_block_intro = line.slice(1, Some(-1)).content().to_owned();
                            state = QuotedBlockParseState::WaitingForQuotedBlock;
                        } else {
                            quoted_block_intro = line.slice(1, None).content().to_owned();
                            state = QuotedBlockParseState::QuotedBlockIntro;
                        }
                    } else {
                        bail!("Quoted block starting char not found")
                    }
                }
                QuotedBlockParseState::WaitingForQuotedBlock => {
                    if !line.is_empty() {
                        if line.content().starts_with(['„', '“']) {
                            if line.content().ends_with('”') {
                                blocks.push(QuotedBlock {
                                    intro: None,
                                    lines: vec![line.slice(1, Some(-1))],
                                    wrap_up: None,
                                });
                                state = QuotedBlockParseState::WaitingForQuotedBlock;
                            } else {
                                quoted_lines = vec![line.slice(1, None)];
                                state = QuotedBlockParseState::QuotedBlock;
                            }
                        } else if line.content().starts_with(['(', '[']) {
                            if line.content().ends_with([')', ']']) {
                                quoted_block_wrap_up = line.slice(1, Some(-1)).content().to_owned();
                                state = QuotedBlockParseState::WrapUp;
                            } else {
                                quoted_block_wrap_up = line.slice(1, None).content().to_owned();
                                state = QuotedBlockParseState::QuotedBlockWrapUp;
                            }
                        } else {
                            wrap_up = line.content().to_owned();
                            state = QuotedBlockParseState::WrapUp;
                        }
                    }
                }

                QuotedBlockParseState::QuotedBlockIntro => {
                    if !line.is_empty()
                        && !quote_checker.end_is_quoted
                        && line.content().ends_with([')', ']'])
                    {
                        line.slice(0, Some(-1)).append_to(&mut quoted_block_intro);
                        state = QuotedBlockParseState::WaitingForQuotedBlock;
                    } else {
                        line.append_to(&mut quoted_block_intro);
                    }
                }

                QuotedBlockParseState::QuotedBlockWrapUp => {
                    if !line.is_empty()
                        && !quote_checker.end_is_quoted
                        && line.content().ends_with([')', ']'])
                    {
                        line.slice(0, Some(-1)).append_to(&mut quoted_block_wrap_up);
                        state = QuotedBlockParseState::WrapUp;
                    } else {
                        line.append_to(&mut quoted_block_wrap_up);
                    }
                }

                QuotedBlockParseState::QuotedBlock => {
                    if !line.is_empty()
                        && !quote_checker.end_is_quoted
                        && line.content().ends_with('”')
                    {
                        quoted_lines.push(line.slice(0, Some(-1)));
                        blocks.push(QuotedBlock {
                            intro: None,
                            lines: std::mem::take(&mut quoted_lines),
                            wrap_up: None,
                        });
                        state = QuotedBlockParseState::WaitingForQuotedBlock;
                    } else {
                        // Note that this else also applies to EMPTY_LINEs
                        quoted_lines.push(line.clone());
                    }
                }
                QuotedBlockParseState::WrapUp => {
                    line.append_to(&mut wrap_up);
                }
            }
        }
        if state != QuotedBlockParseState::WaitingForQuotedBlock
            && state != QuotedBlockParseState::WrapUp
        {
            bail!("Quoted block parser ended in invalid state");
        }

        if let Some(first_block) = blocks.first_mut() {
            first_block.intro = if quoted_block_intro.is_empty() {
                None
            } else {
                Some(quoted_block_intro)
            }
        } else {
            bail!("Quoted block parser didn't find any blocks");
        }

        if let Some(last_block) = blocks.last_mut() {
            last_block.wrap_up = if quoted_block_wrap_up.is_empty() {
                None
            } else {
                Some(quoted_block_wrap_up)
            }
        } else {
            bail!("Quoted block parser didn't find any blocks");
        }

        Ok((
            blocks.into(),
            if wrap_up.is_empty() {
                None
            } else {
                Some(wrap_up)
            },
        ))
    }
}
