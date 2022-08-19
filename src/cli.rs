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

use anyhow::Result;
use clap::Parser;
use log::info;
use serde::Serialize;
use std::io::Write;

use crate::{
    cache::Cache,
    mk_downloader::{download_mk_issue, MkIssue},
    parser::pdf::{parse_pdf, CropBox, PageOfLines},
    parser::{
        mk_act_section::{parse_mk_pages_into_acts, ActRawText},
        structure::parse_act_structure,
    },
    structure::Act,
    util::indentedline::IndentedLine,
};

/// Hun-Law output generator
///
/// Downloads Magyar Közlöny issues as PDFs and converts the Acts in them to machine-parseable formats.
#[derive(clap::Parser, Debug)]
struct HunLawArgs {
    #[clap(value_parser, required = true, name = "issue")]
    /// The  Magyar Közlöny issues to download in YEAR/ISSUE format. Example: '2013/31'
    issues: Vec<MkIssue>,
    /// Output type
    #[clap(value_enum, long, short, default_value_t)]
    output: OutputType,
    /// Do parsing only until and including this step
    #[clap(value_enum, long, short, default_value_t)]
    parse_until: ParsingStep,
}

/// What
#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
enum OutputType {
    /// Plain text output
    Plain,
    /// Plain text output with special markers for bold and not right-justified lines
    TestPlain,
    /// JSON output. Compact. Use YAML format if you need a human readable version
    Json,
    /// YAML output
    Yaml,
}

impl Default for OutputType {
    fn default() -> Self {
        Self::Yaml
    }
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
enum ParsingStep {
    /// Only parse the PDFs into a list of lines
    PdfLines,
    /// Parse into Acts, but only as a list of lines
    ActLines,
    /// Parse Act sructure
    Structure,
    /// Parse the internal texts, try to find semantic phrases, and convert Block Amendments
    Semantic,
}

impl Default for ParsingStep {
    fn default() -> Self {
        Self::Semantic
    }
}

trait CliOutput: Sized + Serialize {
    fn cli_output(self, output_type: OutputType, target: &mut impl std::io::Write) -> Result<()> {
        match output_type {
            OutputType::Plain => self.cli_output_plain(false, target)?,
            OutputType::TestPlain => self.cli_output_plain(true, target)?,
            OutputType::Json => serde_json::to_writer(target, &self)?,
            OutputType::Yaml => serde_yaml::to_writer(target, &self)?,
        };
        Ok(())
    }
    fn cli_output_plain(self, testing_tags: bool, target: &mut impl std::io::Write) -> Result<()>;
}

impl<T: CliOutput> CliOutput for Vec<T> {
    fn cli_output_plain(self, testing_tags: bool, target: &mut impl std::io::Write) -> Result<()> {
        let mut first = true;
        for item in self {
            if !first {
                writeln!(target, "------->8------")?;
            }
            first = false;
            item.cli_output_plain(testing_tags, target)?;
        }
        Ok(())
    }
}

impl CliOutput for PageOfLines {
    fn cli_output_plain(self, testing_tags: bool, target: &mut impl std::io::Write) -> Result<()> {
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
    fn cli_output_plain(self, testing_tags: bool, target: &mut impl std::io::Write) -> Result<()> {
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
    fn cli_output_plain(self, _testing_tags: bool, target: &mut impl std::io::Write) -> Result<()> {
        writeln!(target, "Sorry, no.")?;
        Ok(())
    }
}

fn quick_display_indented_line(l: &IndentedLine, testing_tags: bool) -> String {
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

pub fn cli_main() -> Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    )
    .format(|buf, record| writeln!(buf, "{:>5}: {}", record.level(), record.args()))
    .init();

    let args = HunLawArgs::parse();
    let cache = Cache::new(&"./cache");
    let mut output = std::io::stdout();
    for issue in &args.issues {
        info!("Processing MK {:?}/{:?}", issue.year, issue.issue);
        let body = download_mk_issue(issue, &cache)?;
        info!("{:?} bytes", body.len());
        let crop = CropBox {
            top: 842.0 - 1.25 * 72.0,
            ..Default::default()
        };
        let pages = parse_pdf(&body, crop)?;
        if args.parse_until == ParsingStep::PdfLines {
            pages.cli_output(args.output, &mut output)?;
            continue;
        }

        let acts = parse_mk_pages_into_acts(&pages)?;
        if args.parse_until == ParsingStep::ActLines {
            acts.cli_output(args.output, &mut output)?;
            continue;
        }

        let acts = acts
            .into_iter()
            .map(parse_act_structure)
            .collect::<Result<Vec<_>>>()?;
        if args.parse_until == ParsingStep::Structure {
            acts.cli_output(args.output, &mut output)?;
            continue;
        }

        let acts = acts
            .into_iter()
            .map(|mut act| {
                act = act.add_semantic_info()?;
                act.convert_block_amendments()?;
                Ok(act)
            })
            .collect::<Result<Vec<_>>>()?;
        acts.cli_output(args.output, &mut output)?;
    }
    Ok(())
}
