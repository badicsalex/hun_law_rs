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

use std::{fmt::Display, str::FromStr};

use anyhow::{anyhow, Error, Result};
use lazy_regex::regex_captures;
use serde::{Deserialize, Serialize};

use crate::util::compact_string::CompactString;

#[derive(
    Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct ActIdentifier {
    pub year: i16,
    pub number: i32,
}

impl Display for ActIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}. évi {}. törvény",
            self.year,
            roman::to(self.number).unwrap()
        )
    }
}

impl FromStr for ActIdentifier {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn try_classic(s: &str) -> Option<ActIdentifier> {
            let (_, year, number) = regex_captures!("([0-9]{4}). évi ([IVXLCDM]+). törvény", s)?;
            Some(ActIdentifier {
                year: year.parse().ok()?,
                number: roman::from(number)?,
            })
        }
        fn try_decimal(s: &str) -> Option<ActIdentifier> {
            let (_, year, number) = regex_captures!("([0-9]{4})[/_\\.-]([0-9]+)", s)?;
            Some(ActIdentifier {
                year: year.parse().ok()?,
                number: number.parse().ok()?,
            })
        }
        try_classic(s)
            .or_else(|| try_decimal(s))
            .ok_or_else(|| anyhow!("Unknown act identifier format: {}", s))
    }
}

impl CompactString for ActIdentifier {
    fn fmt_compact_string(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.year, self.number)
    }

    fn from_compact_string(s: impl AsRef<str>) -> Result<Self> {
        let s = s.as_ref();
        if let Some((year, number)) = s.split_once('.') {
            Ok(Self {
                year: year.parse()?,
                number: number.parse()?,
            })
        } else {
            Err(anyhow!("Invalid compact act string: {}", s))
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_act_identifier_parsing() {
        assert_eq!(
            "2012/123".parse::<ActIdentifier>().unwrap(),
            ActIdentifier {
                year: 2012,
                number: 123,
            }
        );
        assert_eq!(
            "2012.123".parse::<ActIdentifier>().unwrap(),
            ActIdentifier {
                year: 2012,
                number: 123,
            }
        );
        assert_eq!(
            "2012-123".parse::<ActIdentifier>().unwrap(),
            ActIdentifier {
                year: 2012,
                number: 123,
            }
        );
        assert_eq!(
            "2012_123".parse::<ActIdentifier>().unwrap(),
            ActIdentifier {
                year: 2012,
                number: 123,
            }
        );
        assert_eq!(
            "2012. évi CLIV. törvény".parse::<ActIdentifier>().unwrap(),
            ActIdentifier {
                year: 2012,
                number: 154,
            }
        );
        let roundtrip_test = ActIdentifier {
            year: 2022,
            number: 420,
        };
        assert_eq!(roundtrip_test, roundtrip_test.to_string().parse().unwrap());
    }

    #[test]
    fn test_compact_string() {
        let tst = ActIdentifier::from_str("2012/420").unwrap();
        assert_eq!(&tst.compact_string().to_string(), "2012.420");
        assert_eq!(tst, ActIdentifier::from_compact_string("2012.420").unwrap());
        assert!(ActIdentifier::from_compact_string("2012").is_err());
        assert!(ActIdentifier::from_compact_string("a").is_err());
        assert!(ActIdentifier::from_compact_string("2012.a").is_err());
        assert!(ActIdentifier::from_compact_string("a.420").is_err());
    }
}
