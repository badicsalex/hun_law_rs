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

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(from = "IdentifierRangeSerdeHelper<T>")]
#[serde(into = "IdentifierRangeSerdeHelper<T>")]
pub struct IdentifierRange<T: Copy + Eq> {
    pub(super) start: T,
    pub(super) end: T,
}

impl<T: Copy + Eq> IdentifierRange<T> {
    pub fn is_range(&self) -> bool {
        self.start != self.end
    }

    pub fn first_in_range(&self) -> T {
        self.start
    }

    pub fn last_in_range(&self) -> T {
        self.end
    }
}

impl<T: Ord + Copy + Eq> IdentifierRange<T> {
    pub fn contains(&self, id: T) -> bool {
        self.start >= id && self.end <= id
    }
}

pub trait IdentifierRangeFrom<T: Copy>: Sized {
    fn from_single(id: T) -> Self {
        Self::from_range(id, id)
    }

    fn from_range(start: T, end: T) -> Self;
}

impl<T: Copy + Eq> IdentifierRangeFrom<T> for IdentifierRange<T> {
    fn from_range(start: T, end: T) -> Self {
        Self { start, end }
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

impl<T: Copy + Eq> From<IdentifierRangeSerdeHelper<T>> for IdentifierRange<T> {
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
impl<T: Copy + Eq> From<IdentifierRange<T>> for IdentifierRangeSerdeHelper<T> {
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
