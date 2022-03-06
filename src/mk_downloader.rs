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

use std::io::Read;
use std::str::FromStr;

use crate::cache::Cache;
use anyhow::{ensure, Result};
use log::info;

#[derive(Debug)]
pub struct MkIssue {
    pub year: i64,
    pub issue: i64,
}

impl MkIssue {
    fn cache_key(&self) -> String {
        format!("MK/{}/{}.pdf", self.year, self.issue)
    }

    fn url(&self) -> String {
        format!(
            "http://www.kozlonyok.hu/nkonline/MKPDF/hiteles/MK{:02}{:03}.pdf",
            self.year % 100,
            self.issue,
        )
    }
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

pub fn download_mk_issue(issue: &MkIssue, cache: &Cache) -> Result<Vec<u8>> {
    if let Ok(cached_result) = cache.load(&issue.cache_key()) {
        return Ok(cached_result);
    }
    info!("Downloading {} into {}", issue.url(), issue.cache_key());
    let http_response = ureq::get(&issue.url()).call()?;
    let mut http_body: Vec<u8> = vec![];
    http_response.into_reader().read_to_end(&mut http_body)?;
    cache.store(&issue.cache_key(), &http_body)?;
    Ok(http_body)
}
