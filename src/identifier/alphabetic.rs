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
    fmt::{Debug, Display},
    str::FromStr,
};

use anyhow::{bail, Error, Result};
use serde::{Deserialize, Serialize};

use super::IdentifierCommon;
use crate::util::compact_string::CompactString;

pub type AlphabeticIdentifier = HungarianIdentifierChar;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "String")]
#[serde(try_from = "String")]
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
    pub fn to_uppercase(&self) -> UppercaseHungarianIdentifierChar {
        UppercaseHungarianIdentifierChar(*self)
    }

    fn ord_helper(&self) -> u16 {
        match self {
            HungarianIdentifierChar::Latin(x) => (*x as u16) * 2,
            HungarianIdentifierChar::Cs => (b'c' as u16) * 2 + 1,
            HungarianIdentifierChar::Dz => (b'd' as u16) * 2 + 1,
            HungarianIdentifierChar::Gy => (b'g' as u16) * 2 + 1,
            HungarianIdentifierChar::Ly => (b'l' as u16) * 2 + 1,
            HungarianIdentifierChar::Ny => (b'n' as u16) * 2 + 1,
            HungarianIdentifierChar::Sz => (b's' as u16) * 2 + 1,
            HungarianIdentifierChar::Ty => (b't' as u16) * 2 + 1,
            HungarianIdentifierChar::Zs => (b'z' as u16) * 2 + 1,
        }
    }
}

impl Ord for HungarianIdentifierChar {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ord_helper().cmp(&other.ord_helper())
    }
}

impl PartialOrd for HungarianIdentifierChar {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl IdentifierCommon for HungarianIdentifierChar {
    fn is_next_from(&self, other: Self) -> bool {
        match (*self, other) {
            (Self::Latin(s), Self::Latin(o)) if s > o && (s - o == 1) => true,
            // The guy who wrote 2018. évi CXVI. törvény thinks 'q' is not part of the latin alphabet.
            (Self::Latin(b'r'), Self::Latin(b'p')) => true,
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

    fn is_first(&self) -> bool {
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
            bail!("{value} is not a valid latin or hungarian character.")
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
            _ => bail!("{value} is not a valid latin or hungarian character string."),
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

impl Debug for HungarianIdentifierChar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl From<HungarianIdentifierChar> for String {
    fn from(val: HungarianIdentifierChar) -> Self {
        val.to_string()
    }
}

impl TryFrom<String> for HungarianIdentifierChar {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

pub struct UppercaseHungarianIdentifierChar(HungarianIdentifierChar);

impl Display for UppercaseHungarianIdentifierChar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            HungarianIdentifierChar::Latin(c) => write!(f, "{}", (c as char).to_uppercase()),
            HungarianIdentifierChar::Cs => write!(f, "CS"),
            HungarianIdentifierChar::Dz => write!(f, "DZ"),
            HungarianIdentifierChar::Gy => write!(f, "GY"),
            HungarianIdentifierChar::Ly => write!(f, "LY"),
            HungarianIdentifierChar::Ny => write!(f, "NY"),
            HungarianIdentifierChar::Sz => write!(f, "SZ"),
            HungarianIdentifierChar::Ty => write!(f, "TY"),
            HungarianIdentifierChar::Zs => write!(f, "ZS"),
        }
    }
}

impl CompactString for HungarianIdentifierChar {
    fn fmt_compact_string(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }

    fn from_compact_string(s: impl AsRef<str>) -> Result<Self> {
        s.as_ref().parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ordering() {
        assert!(
            HungarianIdentifierChar::from_str("ny").unwrap()
                > HungarianIdentifierChar::from_str("n").unwrap()
        );
        assert!(
            HungarianIdentifierChar::from_str("sz").unwrap()
                < HungarianIdentifierChar::from_str("zs").unwrap()
        );
        assert!(
            HungarianIdentifierChar::from_str("a").unwrap()
                < HungarianIdentifierChar::from_str("f").unwrap()
        );
    }
}
