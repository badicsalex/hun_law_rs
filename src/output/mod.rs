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

pub mod text;

use std::io::Write;

use anyhow::Result;
use serde::Serialize;

use crate::{
    parser::{mk_act_section::ActRawText, pdf::PageOfLines},
    structure::Act,
    util::{indentedline::IndentedLine, singleton_yaml},
};

use self::text::{TextOutput, TextOutputParams};

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Plain text output
    #[clap(alias("txt"))]
    Plain,
    /// Plain text output with special markers for bold and not right-justified lines
    TestPlain,
    /// Colored text output
    #[clap(alias("color"))]
    Colored,
    /// JSON output. Compact. Use YAML format if you need a human readable version
    Json,
    /// YAML output
    #[clap(alias("yml"))]
    Yaml,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Yaml
    }
}

pub trait CliOutput: Sized + Serialize {
    fn cli_output(
        self,
        width: usize,
        output_type: OutputFormat,
        target: &mut impl Write,
    ) -> Result<()> {
        match output_type {
            OutputFormat::Plain => self.cli_output_plain(width, false, false, target)?,
            OutputFormat::Colored => self.cli_output_plain(width, false, true, target)?,
            OutputFormat::TestPlain => self.cli_output_plain(width, true, false, target)?,
            OutputFormat::Json => serde_json::to_writer(target, &self)?,
            OutputFormat::Yaml => singleton_yaml::to_writer(target, &self)?,
        };
        Ok(())
    }
    fn cli_output_plain(
        self,
        width: usize,
        testing_tags: bool,
        color: bool,
        target: &mut impl Write,
    ) -> Result<()>;
}

impl CliOutput for Vec<PageOfLines> {
    fn cli_output_plain(
        self,
        width: usize,
        testing_tags: bool,
        color: bool,
        target: &mut impl Write,
    ) -> Result<()> {
        let num_pages = self.len();
        for (page_no, page) in self.into_iter().enumerate() {
            writeln!(
                target,
                "\n------- page {:?}/{:?} -------\n",
                page_no + 1,
                num_pages,
            )?;
            page.cli_output_plain(width, testing_tags, color, target)?;
        }
        Ok(())
    }
}

impl CliOutput for PageOfLines {
    fn cli_output_plain(
        self,
        _width: usize,
        testing_tags: bool,
        _color: bool,
        target: &mut impl Write,
    ) -> Result<()> {
        for line in self.lines {
            writeln!(
                target,
                "{}",
                quick_display_indented_line(&line, testing_tags)
            )?
        }
        Ok(())
    }
}

impl CliOutput for ActRawText {
    fn cli_output_plain(
        self,
        _width: usize,
        testing_tags: bool,
        _color: bool,
        target: &mut impl Write,
    ) -> Result<()> {
        writeln!(target, "Act ID: {} - {}", self.identifier, self.subject)?;
        writeln!(target, "Pub date: {:?}", self.publication_date)?;
        writeln!(target)?;
        for line in self.body {
            writeln!(
                target,
                "{}",
                quick_display_indented_line(&line, testing_tags)
            )?
        }
        Ok(())
    }
}

impl CliOutput for Act {
    fn cli_output_plain(
        self,
        width: usize,
        _testing_tags: bool,
        color: bool,
        target: &mut impl Write,
    ) -> Result<()> {
        let params = TextOutputParams::new(width, color).indented();
        self.write_as_text(target, params)
    }
}

pub fn quick_display_indented_line(l: &IndentedLine, testing_tags: bool) -> String {
    let mut s = String::new();
    let mut indent = (l.indent() * 0.2) as usize;
    if testing_tags {
        if l.is_bold() {
            s.push_str("<BOLD>");
            indent = indent.saturating_sub(6);
        }
        if !l.is_justified() {
            s.push_str("<NJ>");
            indent = indent.saturating_sub(4);
        }
    }
    s.push_str(&" ".repeat(indent));
    s.push_str(l.content());
    s
}
