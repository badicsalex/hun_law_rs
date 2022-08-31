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

pub trait FromHungarianString: Sized {
    fn from_hungarian(s: &str) -> Result<Self>;
}

pub trait ToHungarianString {
    fn to_hungarian(self) -> Result<&'static str>;
}

impl FromHungarianString for NaiveDate {
    fn from_hungarian(s: &str) -> Result<Self> {
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
}

mod generated {
    include!(concat!(env!("OUT_DIR"), "/phf_generated.rs"));
}

macro_rules! hun_str_for_num {
    ($t:ident) => {
        impl FromHungarianString for $t {
            fn from_hungarian(s: &str) -> Result<Self> {
                match generated::STR_TO_INT_HUN.get(s) {
                    Some(v) => Ok(*v as $t),
                    None => Err(anyhow!("Invalid hungarian numeral string: {}", s)),
                }
            }
        }

        impl ToHungarianString for $t {
            fn to_hungarian(self) -> Result<&'static str> {
                match generated::INT_TO_STR_HUN.get(self as usize) {
                    Some(v) => Ok(v),
                    None => Err(anyhow!(
                        "Number out of range for int->hun conversion: {:?}",
                        self
                    )),
                }
            }
        }
    };
}

hun_str_for_num!(u8);
hun_str_for_num!(u16);

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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_to_from_hungarian() {
        for i in 0u16..100u16 {
            assert_eq!(u16::from_hungarian(i.to_hungarian().unwrap()).unwrap(), i);
            assert_eq!(
                u16::from_hungarian(&i.to_hungarian().unwrap().to_uppercase()).unwrap(),
                i
            );
        }
        assert_eq!(u16::from_hungarian("Huszonötödik").unwrap(), 25u16);
        assert_eq!(25u16.to_hungarian().unwrap(), "huszonötödik");
    }
}
