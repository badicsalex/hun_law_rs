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

use std::fmt::{Debug, Display, Formatter, Write};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::IdentifierCommon;
use crate::util::compact_string::CompactString;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(from = "IdentifierRangeSerdeHelper<T>")]
#[serde(into = "IdentifierRangeSerdeHelper<T>")]
pub struct IdentifierRange<T: IdentifierCommon> {
    pub(super) start: T,
    pub(super) end: T,
}

impl<T: IdentifierCommon> IdentifierRange<T> {
    pub fn is_range(&self) -> bool {
        self.start != self.end
    }

    pub fn first_in_range(&self) -> T {
        self.start
    }

    pub fn last_in_range(&self) -> T {
        self.end
    }

    pub fn contains(&self, id: T) -> bool {
        self.start <= id && self.end >= id
    }
}

pub trait IdentifierRangeFrom<T: IdentifierCommon>: Sized {
    fn from_single(id: T) -> Self {
        Self::from_range(id, id)
    }

    fn from_range(start: T, end: T) -> Self;
}

impl<T: IdentifierCommon> IdentifierRangeFrom<T> for IdentifierRange<T> {
    fn from_range(start: T, end: T) -> Self {
        Self { start, end }
    }
}

impl<T: IdentifierCommon + Display> Debug for IdentifierRange<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_range() {
            f.debug_struct("Range")
                .field("start", &self.start)
                .field("end", &self.end)
                .finish()
        } else {
            Display::fmt(&self.start, f)
        }
    }
}

impl<T: IdentifierCommon> CompactString for IdentifierRange<T> {
    fn fmt_compact_string(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.start.fmt_compact_string(f)?;
        if self.is_range() {
            f.write_char('-')?;
            self.end.fmt_compact_string(f)?;
        }
        Ok(())
    }

    fn from_compact_string(s: impl AsRef<str>) -> Result<Self> {
        let s = s.as_ref();
        if let Some((left, right)) = s.split_once('-') {
            Ok(Self::from_range(
                CompactString::from_compact_string(left)?,
                CompactString::from_compact_string(right)?,
            ))
        } else {
            Ok(Self::from_single(CompactString::from_compact_string(s)?))
        }
    }
}

// I tried manually implementing Serialize and Deserialize for IdentifierRange,
// But it was some 200 lines of very error-prone code. This little trick is
// too cute for my taste, but it had to be done.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum IdentifierRangeSerdeHelper<T> {
    Single(T),
    Range { start: T, end: T },
}

impl<T: IdentifierCommon> From<IdentifierRangeSerdeHelper<T>> for IdentifierRange<T> {
    fn from(helper: IdentifierRangeSerdeHelper<T>) -> Self {
        match helper {
            IdentifierRangeSerdeHelper::Single(val) => Self {
                start: val,
                end: val,
            },
            IdentifierRangeSerdeHelper::Range { start, end } => Self { start, end },
        }
    }
}
impl<T: IdentifierCommon> From<IdentifierRange<T>> for IdentifierRangeSerdeHelper<T> {
    fn from(val: IdentifierRange<T>) -> Self {
        if val.start == val.end {
            Self::Single(val.start)
        } else {
            Self::Range {
                start: val.start,
                end: val.end,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::identifier::NumericIdentifier;

    #[test]
    fn test_compact_string() {
        let single = IdentifierRange::from_single(NumericIdentifier::from(5));
        assert_eq!(&single.compact_string().to_string(), "5");
        assert_eq!(single, IdentifierRange::from_compact_string("5").unwrap());
        let range =
            IdentifierRange::from_range(NumericIdentifier::from(5), NumericIdentifier::from(10));
        assert_eq!(&range.compact_string().to_string(), "5-10");
        assert_eq!(range, IdentifierRange::from_compact_string("5-10").unwrap());

        assert!(IdentifierRange::<NumericIdentifier>::from_compact_string("a").is_err());
        assert!(IdentifierRange::<NumericIdentifier>::from_compact_string("a-10").is_err());
        assert!(IdentifierRange::<NumericIdentifier>::from_compact_string("10-a").is_err());
        assert!(IdentifierRange::<NumericIdentifier>::from_compact_string("1-10-5").is_err());
    }
}
