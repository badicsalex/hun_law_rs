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

use std::{
    fmt::{Debug, Display, Formatter},
    str::FromStr,
};

use anyhow::{anyhow, ensure, Error, Result};
use serde::{Deserialize, Serialize};

use super::{HungarianIdentifierChar, IdentifierCommon};
use crate::util::{
    compact_string::CompactString, hun_str::ToHungarianString, DIGITS, ROMAN_DIGITS,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(into = "String")]
#[serde(try_from = "String")]
pub struct NumericIdentifier {
    num: u16,
    suffix: Option<HungarianIdentifierChar>,
}

impl NumericIdentifier {
    pub fn from_roman(s: &str) -> Result<Self> {
        let (num_str, suffix) = Self::split_suffix(s, &ROMAN_DIGITS)?;
        let num = roman::from(num_str)
            .ok_or_else(|| anyhow!("{} is not a valid suffixed roman numeral", s))?
            as u16;
        Ok(Self { num, suffix })
    }

    fn split_suffix<'a>(
        s: &'a str,
        allowed_chars: &'static [char],
    ) -> Result<(&'a str, Option<HungarianIdentifierChar>)> {
        if let Some(suffix_start) = s.find(|c: char| !allowed_chars.contains(&c)) {
            let (prefix, mut suffix_str) = s.split_at(suffix_start);
            if suffix_str.as_bytes()[0] == b'/' {
                suffix_str = &suffix_str[1..];
                ensure!(
                    !suffix_str.is_empty(),
                    "There must be an actual suffix_str after '/'"
                );
            }
            let suffix = if suffix_str.is_empty() {
                None
            } else {
                Some(suffix_str.parse()?)
            };
            Ok((prefix, suffix))
        } else {
            Ok((s, None))
        }
    }

    pub fn with_slash(&self) -> NumericIdentifierWithSlash {
        NumericIdentifierWithSlash { id: *self }
    }

    pub fn to_hungarian(&self) -> Result<String> {
        let mut result = self.num.to_hungarian()?.to_string();
        if let Some(suffix) = self.suffix {
            result.push_str(&format!("/{}", suffix));
        }
        Ok(result)
    }
    pub fn to_roman(&self) -> Result<String> {
        let mut result = roman::to(self.num as i32)
            .ok_or_else(|| anyhow!("Problem converting to roman numeral"))?;
        if let Some(suffix) = self.suffix {
            result.push_str(&format!("/{}", suffix));
        }
        Ok(result)
    }
}

impl IdentifierCommon for NumericIdentifier {
    fn is_next_from(&self, other: Self) -> bool {
        match (self.suffix, other.suffix) {
            (None, _) => self.num.wrapping_sub(other.num) == 1,
            (Some(ss), None) => self.num == other.num && ss.is_first(),
            (Some(ss), Some(so)) => self.num == other.num && ss.is_next_from(so),
        }
    }

    fn is_first(&self) -> bool {
        *self
            == Self {
                num: 1,
                suffix: None,
            }
    }
}

impl FromStr for NumericIdentifier {
    type Err = Error;

    /// Convert a possibly suffixed value to an identifier.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (num_str, suffix) = Self::split_suffix(value, &DIGITS)?;
        ensure!(
            !num_str.is_empty(),
            "{} does not start with a number",
            value
        );
        Ok(Self {
            num: num_str.parse()?,
            suffix,
        })
    }
}

impl Display for NumericIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.suffix {
            Some(suffix) => write!(f, "{:?}{}", self.num, suffix),
            None => write!(f, "{:?}", self.num),
        }
    }
}

impl Debug for NumericIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl From<NumericIdentifier> for String {
    fn from(val: NumericIdentifier) -> Self {
        val.to_string()
    }
}

impl CompactString for NumericIdentifier {
    fn fmt_compact_string(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }

    fn from_compact_string(s: impl AsRef<str>) -> Result<Self> {
        s.as_ref().parse()
    }
}

impl TryFrom<String> for NumericIdentifier {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<u16> for NumericIdentifier {
    fn from(val: u16) -> Self {
        Self {
            num: val,
            suffix: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NumericIdentifierWithSlash {
    id: NumericIdentifier,
}

impl Display for NumericIdentifierWithSlash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.id.suffix {
            Some(suffix) => write!(f, "{:?}/{}", self.id.num, suffix.to_uppercase()),
            None => write!(f, "{:?}", self.id.num),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_numeric_identifier_parsing() {
        assert_eq!(
            "123".parse::<NumericIdentifier>().unwrap(),
            NumericIdentifier {
                num: 123,
                suffix: None
            }
        );
        assert_eq!(
            "123/A".parse::<NumericIdentifier>().unwrap(),
            NumericIdentifier {
                num: 123,
                suffix: Some(HungarianIdentifierChar::Latin(b'a'))
            }
        );
        assert_eq!(
            "123/Z".parse::<NumericIdentifier>().unwrap(),
            NumericIdentifier {
                num: 123,
                suffix: Some(HungarianIdentifierChar::Latin(b'z'))
            }
        );
        assert_eq!(
            "123/C".parse::<NumericIdentifier>().unwrap(),
            NumericIdentifier {
                num: 123,
                suffix: Some(HungarianIdentifierChar::Latin(b'c'))
            }
        );
        assert_eq!(
            "51x".parse::<NumericIdentifier>().unwrap(),
            NumericIdentifier {
                num: 51,
                suffix: Some(HungarianIdentifierChar::Latin(b'x'))
            }
        );
        assert_eq!(
            "5zs".parse::<NumericIdentifier>().unwrap(),
            NumericIdentifier {
                num: 5,
                suffix: Some(HungarianIdentifierChar::Zs)
            }
        );
        assert_eq!(
            "11/cs".parse::<NumericIdentifier>().unwrap(),
            NumericIdentifier {
                num: 11,
                suffix: Some(HungarianIdentifierChar::Cs)
            }
        );
        assert!("".parse::<NumericIdentifier>().is_err());
        assert!("a".parse::<NumericIdentifier>().is_err());
        assert!("1aa".parse::<NumericIdentifier>().is_err());
        assert!("1/".parse::<NumericIdentifier>().is_err());
        assert!("12/".parse::<NumericIdentifier>().is_err());
        assert!("12/aa".parse::<NumericIdentifier>().is_err());
        assert!("12//a".parse::<NumericIdentifier>().is_err());
        assert!("1:123".parse::<NumericIdentifier>().is_err());
    }

    #[test]
    fn test_numeric_identifier_roman_parsing() {
        assert_eq!(
            NumericIdentifier::from_roman("XIV").unwrap(),
            NumericIdentifier {
                num: 14,
                suffix: None
            }
        );
        assert_eq!(
            NumericIdentifier::from_roman("III/i").unwrap(),
            NumericIdentifier {
                num: 3,
                suffix: Some(HungarianIdentifierChar::Latin(b'i'))
            }
        );
        assert_eq!(
            NumericIdentifier::from_roman("XCV/Z").unwrap(),
            NumericIdentifier {
                num: 95,
                suffix: Some(HungarianIdentifierChar::Latin(b'z'))
            }
        );
        assert_eq!(
            NumericIdentifier::from_roman("C/C").unwrap(),
            NumericIdentifier {
                num: 100,
                suffix: Some(HungarianIdentifierChar::Latin(b'c'))
            }
        );
        assert_eq!(
            NumericIdentifier::from_roman("XI/cs").unwrap(),
            NumericIdentifier {
                num: 11,
                suffix: Some(HungarianIdentifierChar::Cs)
            }
        );
        assert!(NumericIdentifier::from_roman("").is_err());
        assert!(NumericIdentifier::from_roman("a").is_err());
        assert!(NumericIdentifier::from_roman("I/").is_err());
        assert!(NumericIdentifier::from_roman("II/").is_err());
        assert!(NumericIdentifier::from_roman("II/aa").is_err());
        assert!(NumericIdentifier::from_roman("II//a").is_err());
        assert!(NumericIdentifier::from_roman("I:II").is_err());
    }

    #[test]
    fn test_ordering() {
        assert!(
            NumericIdentifier::from_str("5").unwrap() > NumericIdentifier::from_str("2").unwrap()
        );
        assert!(
            NumericIdentifier::from_str("2").unwrap() < NumericIdentifier::from_str("5").unwrap()
        );
        assert!(
            NumericIdentifier::from_str("5b").unwrap() > NumericIdentifier::from_str("2").unwrap()
        );
        assert!(
            NumericIdentifier::from_str("2b").unwrap() < NumericIdentifier::from_str("5").unwrap()
        );
        assert!(
            NumericIdentifier::from_str("2b").unwrap() < NumericIdentifier::from_str("2c").unwrap()
        );
        assert!(
            NumericIdentifier::from_str("3").unwrap() > NumericIdentifier::from_str("2c").unwrap()
        );
    }
}
