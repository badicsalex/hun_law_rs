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

use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

use super::{IdentifierCommon, NumericIdentifier};
use crate::util::compact_string::CompactString;

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
// Transparent is needed for the missing "identifier" field functionality
// to work automatically. (See comment at SubArticleElement.identifier)
#[serde(transparent)]
pub struct ParagraphIdentifier(Option<NumericIdentifier>);

impl ParagraphIdentifier {
    pub fn map<T>(&self, f: impl Fn(NumericIdentifier) -> T) -> Option<T> {
        self.0.map(f)
    }
}

impl IdentifierCommon for ParagraphIdentifier {
    fn is_first(&self) -> bool {
        match self.0 {
            Some(x) => x.is_first(),
            None => false,
        }
    }

    fn is_next_from(&self, other: Self) -> bool {
        match (self.0, other.0) {
            (Some(x), Some(o)) => x.is_next_from(o),
            _ => false,
        }
    }

    fn is_empty(&self) -> bool {
        self.0.is_none()
    }
}

impl FromStr for ParagraphIdentifier {
    type Err = Error;

    /// Convert a possibly suffixed value to an identifier.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty() {
            Ok(Self(None))
        } else {
            Ok(Self(Some(value.parse()?)))
        }
    }
}

impl Display for ParagraphIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(identifier) = &self.0 {
            Display::fmt(identifier, f)
        } else {
            Ok(())
        }
    }
}

impl Debug for ParagraphIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl From<ParagraphIdentifier> for String {
    fn from(val: ParagraphIdentifier) -> Self {
        val.to_string()
    }
}

impl TryFrom<String> for ParagraphIdentifier {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl CompactString for ParagraphIdentifier {
    fn fmt_compact_string(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }

    fn from_compact_string(s: impl AsRef<str>) -> Result<Self> {
        s.as_ref().parse()
    }
}

impl From<u16> for ParagraphIdentifier {
    fn from(v: u16) -> Self {
        Self(Some(v.into()))
    }
}

impl From<NumericIdentifier> for ParagraphIdentifier {
    fn from(v: NumericIdentifier) -> Self {
        Self(Some(v))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_paragraph_identifier_parsing() {
        assert_eq!(
            "51".parse::<ParagraphIdentifier>().unwrap(),
            ParagraphIdentifier(Some("51".parse().unwrap()))
        );
        assert_eq!(
            "".parse::<ParagraphIdentifier>().unwrap(),
            ParagraphIdentifier(None)
        );
    }

    #[test]
    fn test_ordering() {
        assert!(
            ParagraphIdentifier(Some("51".parse().unwrap()))
                > ParagraphIdentifier(Some("50".parse().unwrap()))
        );
        assert!(ParagraphIdentifier(None) < ParagraphIdentifier(Some("50".parse().unwrap())));
    }

    #[test]
    fn test_compact_string() {
        let tst_full = ParagraphIdentifier(Some("12a".parse().unwrap()));
        assert_eq!(&tst_full.compact_string().to_string(), "12a");
        assert_eq!(
            tst_full,
            ParagraphIdentifier::from_compact_string("12a").unwrap()
        );

        let tst_empty = ParagraphIdentifier(None);
        assert_eq!(&tst_empty.compact_string().to_string(), "");
        assert_eq!(
            tst_empty,
            ParagraphIdentifier::from_compact_string("").unwrap()
        );

        assert!(ParagraphIdentifier::from_compact_string("a").is_err());
    }
}
