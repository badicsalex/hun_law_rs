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

use std::str::FromStr;

use anyhow::{ensure, Result};
use clap::Parser;

pub mod cache;

#[derive(Parser, Debug)]
/// Hun-Law output generator
///
/// Downloads Magyar Közlöny issues as PDFs and converts the Acts in them to machine-parseable formats.
struct HunLawArgs {
    #[clap(required = true, name = "issue")]
    ///The  Magyar Közlöny issue to download in YEAR/ISSUE format. Example: '2013/31'
    issues: Vec<MkIssue>,
}

#[derive(Debug)]
struct MkIssue {
    year: i64,
    issue: i64,
}

impl FromStr for MkIssue {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('/').collect();
        ensure!(
            parts.len() == 2,
            "Magyar Közlöny issue descriptor format is YEAR/ISSUE"
        );
        Ok(MkIssue {
            year: parts[0].parse::<i64>()?,
            issue: parts[1].parse::<i64>()?,
        })
    }
}

pub fn cli_main() -> Result<()> {
    let args = HunLawArgs::parse();
    for issue in &args.issues {
        println!(
            "Would process Mk Issue year: {:?}, issue: {:?}",
            issue.year, issue.issue
        )
    }
    Ok(())
}
