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
mod mk_downloader;
mod pdf_parser;

use cache::Cache;
use mk_downloader::{download_mk_issue, MkIssue};
use pdf_parser::parse_pdf;

#[derive(Parser, Debug)]
/// Hun-Law output generator
///
/// Downloads Magyar Közlöny issues as PDFs and converts the Acts in them to machine-parseable formats.
struct HunLawArgs {
    #[clap(required = true, name = "issue")]
    ///The  Magyar Közlöny issue to download in YEAR/ISSUE format. Example: '2013/31'
    issues: Vec<MkIssue>,
}

pub fn cli_main() -> Result<()> {
    let args = HunLawArgs::parse();
    let cache = Cache::new(&"./cache");
    for issue in &args.issues {
        info!(
            "Processing Mk Issue {:?}. issue: {:?}",
            issue.year, issue.issue
        );
        let body = download_mk_issue(issue, &cache)?;
        info!("{:?} bytes", body.len());
        let parsed = parse_pdf(&body)?;
        for page in parsed {
            println!("");
            println!("------------");
            print!("{}", page.lines.join("\n"));
        }
    }
    Ok(())
}
