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

mod fixup_editor;

use std::{collections::BTreeSet, fs::File, io::Write, path::PathBuf, str::FromStr};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use fixup_editor::run_fixup_editor;
use hun_law::{
    fixups::Fixups,
    identifier::ActIdentifier,
    mk_downloader::{download_mk_issue, MkIssue, DEFAULT_MK_CROP},
    output::{CliOutput, OutputFormat},
    parser::pdf::parse_pdf,
    parser::{
        mk_act_section::{parse_mk_pages_into_acts, ActRawText},
        structure::parse_act_structure,
    },
};
use log::info;
use serde::{Deserialize, Deserializer};

/// Hun-Law output generator
///
/// Downloads Magyar Közlöny issues as PDFs and converts the Acts in them to machine-parseable formats.
#[derive(clap::Parser, Debug)]
struct HunLawArgs {
    #[clap(required = true, name = "id")]
    /// Acts or Magyar Közlöny issues (if --mk is specified) to convert in long, YEAR/Number or YEAR/ISSUE format.
    /// Examples: "2013/31", "2012. évi C. törvény"
    ids: Vec<String>,
    /// Ids are Magyar Közlöny issues if set
    #[clap(long)]
    mk: bool,
    /// Output format
    #[clap(value_enum, long, short = 't', default_value_t)]
    output_format: OutputFormat,
    /// Do parsing only until and including this step
    #[clap(value_enum, long, short, default_value_t)]
    parse_until: ParsingStep,
    /// Interactively fix errors with a fixup editor, should they occur during parsing
    #[clap(long, short)]
    interactive: bool,
    /// Force showing the fixup editor by emulating a failure. Use with -i
    #[clap(long)]
    force_fixup_editor: bool,
    /// Editor to use for interactive fixups
    #[clap(long, short, default_value = "nvim")]
    editor: String,
    /// Output directory. If not specified, output is printed to stdout
    #[clap(long, short)]
    output_dir: Option<PathBuf>,
    /// Cache directory used to store downloaded MK issue pdfs
    #[clap(long, short, default_value = "./cache")]
    cache_dir: PathBuf,
    /// Width of the word-wrapped text (applies to text output only)
    #[clap(long, short, default_value = "105")]
    width: usize,
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

fn main() -> Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    )
    .format(|buf, record| writeln!(buf, "{:>5}: {}", record.level(), record.args()))
    .init();

    let mut args = HunLawArgs::parse();
    if args.output_dir.is_none() && args.output_format == OutputFormat::Plain {
        args.output_format = OutputFormat::Colored
    }

    let mut everything_ok = true;
    let (issues, acts) = get_issues(&args)?;
    for issue in &issues {
        if let Err(e) = || -> Result<()> {
            let mk_name = format!("mk_{}_{}", issue.year, issue.issue);
            info!("Processing {mk_name}");
            let body = download_mk_issue(issue, &args.cache_dir)?;
            info!("{:?} bytes", body.len());
            let pages = parse_pdf(&body, DEFAULT_MK_CROP.clone())?;
            if args.parse_until == ParsingStep::PdfLines {
                let mut output = get_output(&mk_name, &args)?;
                pages.cli_output(args.width, args.output_format, &mut output)?;
                return Ok(());
            }

            for act in parse_mk_pages_into_acts(&pages)? {
                if !acts.is_empty() && !acts.contains(&act.identifier) {
                    log::info!("Skipping {}", act.identifier);
                    continue;
                }

                let mut output = get_output(&act.identifier.to_string(), &args)?;
                let process_result = if args.interactive {
                    process_single_act_interactive(act, &args, &mut output)
                } else {
                    process_single_act(act, &args, &mut output)
                };
                if let Err(error) = process_result {
                    log::error!("{error:?}");
                    everything_ok = false;
                }
            }
            Ok(())
        }() {
            println!("{e:#?}");
        }
    }
    if everything_ok {
        Ok(())
    } else {
        Err(anyhow!("Some acts were not processed"))
    }
}

fn get_output(filename: &str, args: &HunLawArgs) -> Result<Box<dyn std::io::Write>> {
    match &args.output_dir {
        Some(odir) => {
            let extension = match args.output_format {
                OutputFormat::Plain => "txt",
                OutputFormat::TestPlain => "txt",
                OutputFormat::Colored => "txt",
                OutputFormat::Json => "json",
                OutputFormat::Yaml => "yml",
            };
            let path = odir.join(format!("{filename}.{extension}"));
            info!("Writing into {:?}", path);
            Ok(Box::new(File::create(path)?))
        }
        None => {
            println!("------ {filename} ------");
            Ok(Box::new(std::io::stdout()))
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ActToMkIssueRecord {
    #[serde(deserialize_with = "deserialize_from_str")]
    mk_issue: MkIssue,
    #[serde(deserialize_with = "deserialize_from_str")]
    act: ActIdentifier,
}

fn deserialize_from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    let buf = String::deserialize(deserializer)?;
    T::from_str(&buf).map_err(serde::de::Error::custom)
}

fn get_issues(args: &HunLawArgs) -> Result<(Vec<MkIssue>, Vec<ActIdentifier>)> {
    if args.mk {
        Ok((
            args.ids
                .iter()
                .map(|s| MkIssue::from_str(s))
                .collect::<Result<Vec<_>>>()?,
            Vec::new(),
        ))
    } else {
        let path = "./data/act_to_mk_issue.csv";
        let acts = args
            .ids
            .iter()
            .map(|s| ActIdentifier::from_str(s))
            .collect::<Result<Vec<_>>>()?;
        let records = csv::Reader::from_path(path)
            .with_context(|| anyhow!("Error opening {path}"))?
            .deserialize()
            .collect::<csv::Result<Vec<ActToMkIssueRecord>>>()
            .with_context(|| anyhow!("Error parsing {path}"))?;
        let mut issues = BTreeSet::new();
        for act in &acts {
            let issue = records
                .iter()
                .find_map(|r| {
                    if r.act == *act {
                        Some(r.mk_issue.clone())
                    } else {
                        None
                    }
                })
                .ok_or_else(|| anyhow!("Could not find {act} in {path}"))?;
            issues.insert(issue);
        }

        Ok((issues.into_iter().collect(), acts))
    }
}

fn process_single_act_interactive(
    act_raw: ActRawText,
    args: &HunLawArgs,
    output: &mut impl std::io::Write,
) -> Result<()> {
    while let Err(error) = process_single_act(act_raw.clone(), args, output) {
        log::error!("{error:?}");
        if confirm("Try to fix issue in editor?")? {
            // TODO: Remove this duplicate code somehow
            let mut act_fixed_up = act_raw.clone();
            Fixups::load(act_fixed_up.identifier)?.apply(&mut act_fixed_up.body)?;
            act_fixed_up.remove_double_empty_lines();

            run_fixup_editor(&act_fixed_up, &args.editor)?;
            continue;
        }
        return Err(error);
    }
    Ok(())
}

fn process_single_act(
    mut act_raw: ActRawText,
    args: &HunLawArgs,
    output: &mut impl std::io::Write,
) -> Result<()> {
    info!("Parsing {}", act_raw.identifier);
    Fixups::load(act_raw.identifier)?.apply(&mut act_raw.body)?;
    act_raw.remove_double_empty_lines();

    if args.parse_until == ParsingStep::ActLines {
        return act_raw.cli_output(args.width, args.output_format, output);
    }

    let mut act = parse_act_structure(&act_raw)?;

    if args.parse_until == ParsingStep::Structure {
        return act.cli_output(args.width, args.output_format, output);
    }

    act.add_semantic_info()?;
    act.convert_block_amendments()?;
    act.cli_output(args.width, args.output_format, output)?;
    if args.force_fixup_editor {
        Err(anyhow!("Forcing fixup editor because of parameters"))
    } else {
        Ok(())
    }
}

fn confirm(s: &str) -> Result<bool> {
    eprint!("{s} [Y/n]");
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    buf.make_ascii_lowercase();
    Ok(buf.trim().is_empty() || buf.starts_with('y'))
}
