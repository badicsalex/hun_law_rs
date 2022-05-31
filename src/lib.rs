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

pub mod cache;
pub mod mk_act_section_parser;
pub mod mk_downloader;
pub mod pdf_parser;
pub mod util;
pub mod structure;
pub mod reference;

use cache::Cache;
use mk_act_section_parser::parse_mk_pages_into_acts;
use mk_downloader::{download_mk_issue, MkIssue};
use pdf_parser::{parse_pdf, CropBox, PageOfLines};
use util::indentedline::IndentedLine;

#[derive(Parser, Debug)]
/// Hun-Law output generator
///
/// Downloads Magyar Közlöny issues as PDFs and converts the Acts in them to machine-parseable formats.
struct HunLawArgs {
    #[clap(required = true, name = "issue")]
    ///The  Magyar Közlöny issue to download in YEAR/ISSUE format. Example: '2013/31'
    issues: Vec<MkIssue>,
}

pub fn quick_display_indented_line(l: &IndentedLine) -> String {
    let mut s = String::new();
    let mut indent = (l.indent() * 0.2) as usize;
    if l.is_bold() {
        s.push_str("<B>");
        indent -= 4;
    }
    s.push_str(&" ".repeat(indent));
    s.push_str(l.content());
    s
}

fn output_indented_lines(lines: &[IndentedLine]) {
    print!(
        "{}",
        lines
            .iter()
            .map(quick_display_indented_line)
            .collect::<Vec<String>>()
            .join("\n")
    );
    println!();
}

#[allow(dead_code)]
fn output_raw_text(pages: &[PageOfLines]) {
    for page in pages {
        println!();
        println!("------------");
        output_indented_lines(&page.lines);
    }
}

pub fn cli_main() -> Result<()> {
    let args = HunLawArgs::parse();
    let cache = Cache::new(&"./cache");
    for issue in &args.issues {
        info!("Processing MK {:?}/{:?}", issue.year, issue.issue);
        let body = download_mk_issue(issue, &cache)?;
        info!("{:?} bytes", body.len());
        let crop = CropBox {
            top: 842.0 - 1.25 * 72.0,
            ..Default::default()
        };
        let pages = parse_pdf(&body, crop)?;
        let acts = parse_mk_pages_into_acts(&pages)?;
        println!();
        for act in acts {
            println!("------------");
            println!("Act ID: {} - {}", act.identifier.to_string(), act.subject);
            println!("Pub date: {:?}", act.publication_date);
            println!();
            output_indented_lines(&act.body);
            println!();
        }
    }
    Ok(())
}
