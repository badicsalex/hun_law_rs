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

use anyhow::{bail, ensure, Result};

use super::sae::ExtractMultipleResult;
use crate::{
    structure::{ParagraphChildren, QuotedBlock},
    util::{indentedline::IndentedLine, QuoteCheck},
};

pub struct QuotedBlockParser {
    state: fn(&mut QuotedBlockParser, line: &IndentedLine),
    blocks: Vec<QuotedBlock>,
    wrap_up: String,
    quoted_block_intro: String,
    quoted_block_wrap_up: String,
    quoted_lines: Vec<IndentedLine>,
    quote_checker: QuoteCheck,
}

impl QuotedBlockParser {
    fn new(expect_intro: bool) -> Self {
        Self {
            state: if expect_intro {
                Self::state_start_expect_intro
            } else {
                Self::state_start
            },
            blocks: Vec::new(),
            wrap_up: String::new(),
            quoted_block_intro: String::new(),
            quoted_block_wrap_up: String::new(),
            quoted_lines: Vec::new(),
            quote_checker: QuoteCheck::default(),
        }
    }

    fn parse_line(&mut self, line: &IndentedLine) -> Result<()> {
        self.quote_checker.update(line)?;
        (self.state)(self, line);
        Ok(())
    }

    fn state_start(&mut self, line: &IndentedLine) {
        if line.is_empty() {
            return;
        }
        let line = if line.content().starts_with(['„', '“']) {
            self.state = Self::state_inside_quoted_block;
            line.slice(1, None)
        } else {
            self.state = Self::state_wrap_up;
            line.clone()
        };
        (self.state)(self, &line);
    }

    fn state_start_expect_intro(&mut self, line: &IndentedLine) {
        if line.is_empty() {
            return;
        }
        let line = if line.content().starts_with(['(', '[']) {
            self.state = Self::state_quoted_block_intro;
            line.slice(1, None)
        } else {
            self.state = Self::state_start;
            line.clone()
        };
        (self.state)(self, &line);
    }

    fn state_quoted_block_intro(&mut self, line: &IndentedLine) {
        if !line.is_empty()
            && !self.quote_checker.end_is_quoted
            && line.content().ends_with([')', ']'])
        {
            line.slice(0, Some(-1))
                .append_to(&mut self.quoted_block_intro);
            self.state = Self::state_start_expect_intro;
        } else {
            line.append_to(&mut self.quoted_block_intro);
        }
    }

    fn state_quoted_block_wrap_up(&mut self, line: &IndentedLine) {
        if !line.is_empty()
            && !self.quote_checker.end_is_quoted
            && line.content().ends_with([')', ']'])
        {
            line.slice(0, Some(-1))
                .append_to(&mut self.quoted_block_wrap_up);
            self.state = Self::state_waiting_for_quoted_block_wrap_up;
        } else {
            line.append_to(&mut self.quoted_block_wrap_up);
        }
    }

    fn state_waiting_for_quoted_block(&mut self, line: &IndentedLine) {
        if line.is_empty() {
            return;
        }
        let line = if line.content().starts_with(['„', '“']) {
            self.state = Self::state_inside_quoted_block;
            line.slice(1, None)
        } else {
            self.state = Self::state_waiting_for_quoted_block_wrap_up;
            line.clone()
        };
        (self.state)(self, &line);
    }

    fn state_waiting_for_quoted_block_wrap_up(&mut self, line: &IndentedLine) {
        if line.is_empty() {
            return;
        }
        let line = if line.content().starts_with(['(', '[']) {
            self.state = Self::state_quoted_block_wrap_up;
            line.slice(1, None)
        } else {
            self.state = Self::state_wrap_up;
            line.clone()
        };
        (self.state)(self, &line);
    }

    fn state_inside_quoted_block(&mut self, line: &IndentedLine) {
        if !line.is_empty() && !self.quote_checker.end_is_quoted && line.content().ends_with('”')
        {
            self.quoted_lines.push(line.slice(0, Some(-1)));
            self.blocks.push(QuotedBlock {
                intro: None,
                lines: std::mem::take(&mut self.quoted_lines),
                wrap_up: None,
            });
            self.state = Self::state_waiting_for_quoted_block;
        } else {
            // Note that this else also applies to EMPTY_LINEs
            self.quoted_lines.push(line.clone());
        }
    }
    fn state_wrap_up(&mut self, line: &IndentedLine) {
        line.append_to(&mut self.wrap_up);
    }

    /// Extract multiple instances from the text. Fails if line does not start with quote
    pub fn extract_multiple(
        previous_nonempty_line: Option<&IndentedLine>,
        lines: &[IndentedLine],
    ) -> Result<ExtractMultipleResult<ParagraphChildren>> {
        let expect_intro = previous_nonempty_line.map_or(false, |l| l.content().ends_with(':'));

        // Fast fail path
        if let Some(first_line) = lines.get(0) {
            let search_for: &[char] = if expect_intro {
                &['(', '[', '„', '“']
            } else {
                &['„', '“']
            };
            if !first_line.content().starts_with(search_for) {
                bail!("Could not find quoted block starting token");
            }
        } else {
            bail!("Empty line list for quoted block");
        }

        let mut state = Self::new(expect_intro);
        for line in lines {
            state.parse_line(line)?;
        }

        ensure!(
            state.state as usize == Self::state_waiting_for_quoted_block as usize
                || state.state as usize == Self::state_wrap_up as usize
                || state.state as usize == Self::state_waiting_for_quoted_block_wrap_up as usize,
            "Quoted block parser ended in invalid state"
        );

        if let Some(first_block) = state.blocks.first_mut() {
            first_block.intro = if state.quoted_block_intro.is_empty() {
                None
            } else {
                Some(state.quoted_block_intro)
            }
        } else {
            bail!("Quoted block parser didn't find any blocks");
        }

        if let Some(last_block) = state.blocks.last_mut() {
            last_block.wrap_up = if state.quoted_block_wrap_up.is_empty() {
                None
            } else {
                Some(state.quoted_block_wrap_up)
            }
        } else {
            bail!("Quoted block parser didn't find any blocks");
        }

        Ok(ExtractMultipleResult {
            elements: state.blocks.into(),
            parent_wrap_up: if state.wrap_up.is_empty() {
                None
            } else {
                Some(state.wrap_up)
            },
            rest_of_wrap_up: Vec::new(),
        })
    }
}
