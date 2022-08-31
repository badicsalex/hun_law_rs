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

use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use lazy_regex::regex_captures;

/// Convert from string to date, using a typical hungarian date pattern
pub fn date_from_hungarian_string(s: &str) -> Result<NaiveDate> {
    // This is not a performance critical part, so I won't bother with 'optimizing' this regex
    let (_, year, month, day) = regex_captures!(r"^(\d{4}). ([^ ]+) (\d{1,2}).", s)
        .ok_or_else(|| anyhow!("Could not parse date string {}", s))?;

    NaiveDate::from_ymd_opt(
        year.parse()?,
        text_to_month_hun(month)?.into(),
        day.parse()?,
    )
    .ok_or_else(|| anyhow!("Invalid date: {}", s))
}

pub fn text_to_month_hun(s: &str) -> Result<u8> {
    match s {
        "január" => Ok(1),
        "február" => Ok(2),
        "március" => Ok(3),
        "április" => Ok(4),
        "május" => Ok(5),
        "június" => Ok(6),
        "július" => Ok(7),
        "augusztus" => Ok(8),
        "szeptember" => Ok(9),
        "október" => Ok(10),
        "november" => Ok(11),
        "december" => Ok(12),
        _ => Err(anyhow!("Invalid month name {}", s)),
    }
}
