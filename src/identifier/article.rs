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

use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

use super::{IsNextFrom, NumericIdentifier};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(into = "String")]
#[serde(try_from = "String")]
pub struct ArticleIdentifier {
    book: Option<u8>,
    identifier: NumericIdentifier,
}

impl IsNextFrom for ArticleIdentifier {
    fn is_next_from(&self, other: Self) -> bool {
        match (self.book, other.book) {
            (None, None) => self.identifier.is_next_from(other.identifier),
            (Some(bs), Some(bo)) if bs == bo => self.identifier.is_next_from(other.identifier),
            (Some(bs), Some(bo)) if bs.wrapping_sub(bo) == 1 => self.identifier.is_first(),
            _ => false,
        }
    }

    fn is_first(&self) -> bool {
        if let Some(book) = self.book {
            book == 1 && self.identifier.is_first()
        } else {
            self.identifier.is_first()
        }
    }
}

impl FromStr for ArticleIdentifier {
    type Err = Error;

    /// Convert a possibly suffixed value to an identifier.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Some((book_str, id_str)) = value.split_once(':') {
            Ok(Self {
                book: Some(book_str.parse()?),
                identifier: id_str.parse()?,
            })
        } else {
            Ok(Self {
                book: None,
                identifier: value.parse()?,
            })
        }
    }
}

impl Display for ArticleIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.book {
            Some(book) => {
                write!(f, "{:?}:", book)?;
                self.identifier.fmt_with_slash(f)
            }
            None => self.identifier.fmt_with_slash(f),
        }
    }
}

impl From<ArticleIdentifier> for String {
    fn from(val: ArticleIdentifier) -> Self {
        val.to_string()
    }
}

impl TryFrom<String> for ArticleIdentifier {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<u16> for ArticleIdentifier {
    fn from(val: u16) -> Self {
        Self {
            book: None,
            identifier: val.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_article_identifier_parsing() {
        assert_eq!(
            "123".parse::<ArticleIdentifier>().unwrap(),
            ArticleIdentifier {
                book: None,
                identifier: "123".parse().unwrap()
            }
        );
        assert_eq!(
            "88:1/SZ".parse::<ArticleIdentifier>().unwrap(),
            ArticleIdentifier {
                book: Some(88),
                identifier: "1/SZ".parse().unwrap()
            }
        );
    }

    #[test]
    fn test_ordering() {
        assert!(
            ArticleIdentifier::from_str("9").unwrap() > ArticleIdentifier::from_str("6/C").unwrap()
        );
        assert!(
            ArticleIdentifier::from_str("2:12").unwrap()
                < ArticleIdentifier::from_str("3:1").unwrap()
        );
        assert!(
            ArticleIdentifier::from_str("3:12/X").unwrap()
                < ArticleIdentifier::from_str("3:15/B").unwrap()
        );
    }
}