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

use anyhow::{anyhow, bail, Error, Result};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

use crate::util::{IsDefault, DIGITS, ROMAN_DIGITS};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "String")]
#[serde(try_from = "String")]
pub struct NumericIdentifier {
    num: u16,
    suffix: Option<HungarianIdentifierChar>,
}

impl NumericIdentifier {
    /// Can the parameter be considered the previous identifier. Handles suffix transitions.
    ///
    /// ```
    /// use hun_law::structure::NumericIdentifier;
    /// fn check_is_next_from(s1:&str, s2:&str) -> bool{
    ///     let i1 = s1.parse::<NumericIdentifier>().unwrap();
    ///     let i2 = s2.parse::<NumericIdentifier>().unwrap();
    ///     println!("{:?} {:?}", i1, i2);
    ///     i1.is_next_from(i2)
    /// };
    /// assert!(check_is_next_from("123", "122"));
    /// assert!(check_is_next_from("123/A", "123"));
    /// assert!(check_is_next_from("123zs", "123z"));
    /// assert!(check_is_next_from("13", "12c"));
    ///
    /// assert!(!check_is_next_from("12b", "12"));
    /// assert!(!check_is_next_from("12a", "11"));
    /// assert!(!check_is_next_from("13", "11"));
    /// assert!(!check_is_next_from("11", "11"));
    /// ```
    pub fn is_next_from(&self, other: Self) -> bool {
        match (self.suffix, other.suffix) {
            (None, _) => self.num - other.num == 1,
            (Some(ss), None) => self.num == other.num && ss.is_first(),
            (Some(ss), Some(so)) => self.num == other.num && ss.is_next_from(so),
        }
    }

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
                if suffix_str.is_empty() {
                    bail!("There must be an actual suffix_str after '/'")
                }
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
}

impl FromStr for NumericIdentifier {
    type Err = Error;

    /// Convert a possibly suffixed value to an identifier.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (num_str, suffix) = Self::split_suffix(value, &DIGITS)?;
        if num_str.is_empty() {
            bail!("{} does not start with a number", value);
        }
        Ok(Self {
            num: num_str.parse()?,
            suffix,
        })
    }
}

impl Display for NumericIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.suffix {
            Some(suffix) => write!(f, "{:?}{}", self.num, suffix),
            None => write!(f, "{:?}", self.num),
        }
    }
}

impl From<NumericIdentifier> for String {
    fn from(val: NumericIdentifier) -> Self {
        val.to_string()
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

impl IsDefault for NumericIdentifier {
    fn is_default(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HungarianIdentifierChar {
    /// A single lowercase latin character. Range: b'a'..=b'z'
    // I wish this could be niche optimized, but there's no pretty way to do it.
    // Also no reason to do it, because this is not exaclty performance sensitive.
    Latin(u8),
    // Hungarian digraphs in identifiers are actually very rare, and actually disallowed,
    // but they do occur in the amended version of 1997. évi XXXI. törvény [Gyvt.]
    // (Amendment in e.g. 2013. évi XXVII. törvény), so they must be handled correctly.
    //
    // Fortunately I haven't found cases with accented characters.
    Cs,
    Dz,
    Gy,
    Ly,
    Ny,
    Sz,
    Ty,
    Zs,
}

impl HungarianIdentifierChar {
    /// Is the parameter the previous character in the ABC. Handles digraph special cases.
    ///
    /// ```
    /// use hun_law::structure::HungarianIdentifierChar;
    /// let a: HungarianIdentifierChar = 'a'.try_into().unwrap();
    /// let b: HungarianIdentifierChar = 'B'.try_into().unwrap();
    /// let n: HungarianIdentifierChar = 'n'.try_into().unwrap();
    /// let ny: HungarianIdentifierChar = "Ny".parse().unwrap();
    /// let o: HungarianIdentifierChar = "o".parse().unwrap();
    /// assert!(b.is_next_from(a));
    /// assert!(!a.is_next_from(b));
    /// assert!(!a.is_next_from(a));
    /// assert!(ny.is_next_from(n));
    /// assert!(!ny.is_next_from(ny));
    /// assert!(!n.is_next_from(ny));
    /// assert!(o.is_next_from(ny));
    /// ```
    pub fn is_next_from(&self, other: Self) -> bool {
        match (*self, other) {
            (Self::Latin(s), Self::Latin(o)) => s > o && (s - o == 1),
            (Self::Cs, Self::Latin(b'c')) => true,
            (Self::Dz, Self::Latin(b'd')) => true,
            (Self::Gy, Self::Latin(b'g')) => true,
            (Self::Ly, Self::Latin(b'l')) => true,
            (Self::Ny, Self::Latin(b'n')) => true,
            (Self::Sz, Self::Latin(b's')) => true,
            (Self::Ty, Self::Latin(b't')) => true,
            (Self::Zs, Self::Latin(b'z')) => true,
            (Self::Latin(b'd'), Self::Cs) => true,
            (Self::Latin(b'e'), Self::Dz) => true,
            (Self::Latin(b'h'), Self::Gy) => true,
            (Self::Latin(b'm'), Self::Ly) => true,
            (Self::Latin(b'o'), Self::Ny) => true,
            (Self::Latin(b't'), Self::Sz) => true,
            (Self::Latin(b'u'), Self::Ty) => true,
            _ => false,
        }
    }

    pub fn is_first(&self) -> bool {
        *self == Self::Latin(b'a')
    }
}

impl TryFrom<char> for HungarianIdentifierChar {
    type Error = Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        let value = value.to_ascii_lowercase();
        if ('a'..='z').contains(&value) {
            Ok(Self::Latin(value as u8))
        } else {
            bail!("{} is not a valid latin or hungarian character.", value)
        }
    }
}

impl FromStr for HungarianIdentifierChar {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() == 1 {
            return Self::try_from(value.chars().next().unwrap());
        }
        match value {
            "cs" | "Cs" | "CS" => Ok(Self::Cs),
            "dz" | "Dz" | "DZ" => Ok(Self::Dz),
            "gy" | "Gy" | "GY" => Ok(Self::Gy),
            "ly" | "Ly" | "LY" => Ok(Self::Ly),
            "ny" | "Ny" | "NY" => Ok(Self::Ny),
            "sz" | "Sz" | "SZ" => Ok(Self::Sz),
            "ty" | "Ty" | "TY" => Ok(Self::Ty),
            "zs" | "Zs" | "ZS" => Ok(Self::Zs),
            _ => bail!(
                "{} is not a valid latin or hungarian character string.",
                value
            ),
        }
    }
}

impl Display for HungarianIdentifierChar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HungarianIdentifierChar::Latin(c) => write!(f, "{}", *c as char),
            HungarianIdentifierChar::Cs => write!(f, "cs"),
            HungarianIdentifierChar::Dz => write!(f, "dz"),
            HungarianIdentifierChar::Gy => write!(f, "gy"),
            HungarianIdentifierChar::Ly => write!(f, "ly"),
            HungarianIdentifierChar::Ny => write!(f, "ny"),
            HungarianIdentifierChar::Sz => write!(f, "sz"),
            HungarianIdentifierChar::Ty => write!(f, "ty"),
            HungarianIdentifierChar::Zs => write!(f, "zs"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

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
}
