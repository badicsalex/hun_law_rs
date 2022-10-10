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

use anyhow::{anyhow, Error};
use lazy_regex::regex_captures;
use serde::{Deserialize, Serialize};

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
}
