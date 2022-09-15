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

use std::{fs::File, io::Write, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use fixup_editor::run_fixup_editor;
use hun_law::{
    fixups::Fixups,
    mk_downloader::{download_mk_issue, MkIssue, DEFAULT_MK_CROP},
    output::{CliOutput, OutputFormat},
    parser::pdf::parse_pdf,
    parser::{
        mk_act_section::{parse_mk_pages_into_acts, ActRawText},
        structure::parse_act_structure,
    },
};
use log::info;

/// Hun-Law output generator
///
/// Downloads Magyar Közlöny issues as PDFs and converts the Acts in them to machine-parseable formats.
#[derive(clap::Parser, Debug)]
struct HunLawArgs {
    #[clap(value_parser, required = true, name = "issue")]
    /// The  Magyar Közlöny issues to download in YEAR/ISSUE format. Example: '2013/31'
    issues: Vec<MkIssue>,
    /// Output format
    #[clap(value_enum, long, short = 't', default_value_t)]
    output_format: OutputFormat,
    /// Do parsing only until and including this step
    #[clap(value_enum, long, short, default_value_t)]
    parse_until: ParsingStep,
    /// Interactively fix errors with a fixup editor, should they occur during parsing
    #[clap(long, short)]
    interactive: bool,
    /// Editor to use for interactive fixups
    #[clap(long, short, default_value = "nvim")]
    editor: String,
    /// Output directory. If not specified, output is printed to stdout
    #[clap(long, short)]
    output_dir: Option<PathBuf>,
    /// Cache directory used to store downloaded MK issue pdfs
    #[clap(long, short, default_value = "./cache")]
    cache_dir: PathBuf,
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
    for issue in &args.issues {
        let mk_name = format!("mk_{}_{}", issue.year, issue.issue);
        info!("Processing {}", mk_name);
        let body = download_mk_issue(issue, &args.cache_dir)?;
        info!("{:?} bytes", body.len());
        let pages = parse_pdf(&body, DEFAULT_MK_CROP.clone())?;
        if args.parse_until == ParsingStep::PdfLines {
            let mut output = get_output(&mk_name, &args)?;
            pages.cli_output(args.output_format, &mut output)?;
            continue;
        }

        for act in parse_mk_pages_into_acts(&pages)? {
            let mut output = get_output(&act.identifier.to_string(), &args)?;
            let process_result = if args.interactive {
                process_single_act_interactive(act, &args, &mut output)
            } else {
                process_single_act(act, &args, &mut output)
            };
            if let Err(error) = process_result {
                log::error!("{:?}", error);
                everything_ok = false;
            }
        }
    }
    if everything_ok {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Some acts were not processed"))
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
            let path = odir.join(format!("{}.{}", filename, extension));
            info!("Writing into {:?}", path);
            Ok(Box::new(File::create(path)?))
        }
        None => {
            println!("------ {} ------", filename);
            Ok(Box::new(std::io::stdout()))
        }
    }
}

fn process_single_act_interactive(
    act_raw: ActRawText,
    args: &HunLawArgs,
    output: &mut impl std::io::Write,
) -> Result<()> {
    while let Err(error) = process_single_act(act_raw.clone(), args, output) {
        log::error!("{:?}", error);
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
        return act_raw.cli_output(args.output_format, output);
    }

    let mut act = parse_act_structure(&act_raw)?;

    if args.parse_until == ParsingStep::Structure {
        return act.cli_output(args.output_format, output);
    }

    act.add_semantic_info()?;
    act.convert_block_amendments()?;
    act.cli_output(args.output_format, output)
}

fn confirm(s: &str) -> Result<bool> {
    eprint!("{} [Y/n]", s);
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    buf.make_ascii_lowercase();
    Ok(buf.trim().is_empty() || buf.starts_with('y'))
}
