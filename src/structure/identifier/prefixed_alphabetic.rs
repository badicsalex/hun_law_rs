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

use anyhow::{anyhow, Error};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

use crate::util::IsDefault;

use super::{HungarianIdentifierChar, IsNextFrom};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(into = "String")]
#[serde(try_from = "String")]
pub struct PrefixedAlphabeticIdentifier {
    prefix: Option<HungarianIdentifierChar>,
    chr: HungarianIdentifierChar,
}

impl PrefixedAlphabeticIdentifier {
    pub fn prefix_is(&self, prefix: Option<HungarianIdentifierChar>) -> bool {
        self.prefix == prefix
    }
}

impl IsNextFrom for PrefixedAlphabeticIdentifier {
    fn is_first(&self) -> bool {
        self.chr.is_first()
    }

    fn is_next_from(&self, other: Self) -> bool {
        match (self.prefix, other.prefix) {
            (None, None) => self.chr.is_next_from(other.chr),
            (Some(ps), Some(po)) if ps == po => self.chr.is_next_from(other.chr),
            _ => false,
        }
    }
}

impl FromStr for PrefixedAlphabeticIdentifier {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.chars().count() {
            1 => Ok(Self {
                prefix: None,
                chr: s.chars().next().unwrap().try_into()?,
            }),
            2 => {
                let mut c = s.chars();
                Ok(Self {
                    prefix: Some(c.next().unwrap().try_into()?),
                    chr: c.next().unwrap().try_into()?,
                })
            }
            _ => Err(anyhow!(
                "{} is not a valid prefixed alphabetic identifier",
                s
            )),
        }
    }
}

impl Display for PrefixedAlphabeticIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.prefix {
            Some(p) => write!(f, "{}{}", p, self.chr),
            None => self.chr.fmt(f),
        }
    }
}

impl From<PrefixedAlphabeticIdentifier> for String {
    fn from(val: PrefixedAlphabeticIdentifier) -> Self {
        val.to_string()
    }
}

impl TryFrom<String> for PrefixedAlphabeticIdentifier {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl IsDefault for PrefixedAlphabeticIdentifier {
    fn is_default(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefixed_alphabetic_parsing() {
        assert_eq!(
            "a".parse::<PrefixedAlphabeticIdentifier>().unwrap(),
            PrefixedAlphabeticIdentifier {
                prefix: None,
                chr: HungarianIdentifierChar::Latin(b'a'),
            }
        );
        assert_eq!(
            "aa".parse::<PrefixedAlphabeticIdentifier>().unwrap(),
            PrefixedAlphabeticIdentifier {
                prefix: Some(HungarianIdentifierChar::Latin(b'a')),
                chr: HungarianIdentifierChar::Latin(b'a'),
            }
        );
        // Known limitation
        assert_eq!(
            "sz".parse::<PrefixedAlphabeticIdentifier>().unwrap(),
            PrefixedAlphabeticIdentifier {
                prefix: Some(HungarianIdentifierChar::Latin(b's')),
                chr: HungarianIdentifierChar::Latin(b'z'),
            }
        );
    }

    #[test]
    fn test_ordering() {
        assert!(
            PrefixedAlphabeticIdentifier::from_str("x").unwrap()
                > PrefixedAlphabeticIdentifier::from_str("a").unwrap()
        );
        assert!(
            PrefixedAlphabeticIdentifier::from_str("xa").unwrap()
                < PrefixedAlphabeticIdentifier::from_str("xb").unwrap()
        );
        assert!(
            PrefixedAlphabeticIdentifier::from_str("cc").unwrap()
                < PrefixedAlphabeticIdentifier::from_str("dc").unwrap()
        );
    }
}
