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

use std::fmt::{Display, Formatter};

use anyhow::Result;

pub struct CompactStringFormatter<'a, T: CompactString>(&'a T);

pub trait CompactString: Sized {
    fn compact_string(&self) -> CompactStringFormatter<Self> {
        CompactStringFormatter(self)
    }

    fn fmt_compact_string(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
    fn from_compact_string(s: impl AsRef<str>) -> Result<Self>;
}

impl<'a, T: CompactString> Display for CompactStringFormatter<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt_compact_string(f)
    }
}

impl<T: CompactString> CompactString for Option<T> {
    fn fmt_compact_string(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Some(x) => x.fmt_compact_string(f),
            None => Ok(()),
        }
    }

    fn from_compact_string(s: impl AsRef<str>) -> Result<Self> {
        let s = s.as_ref();
        if s.is_empty() {
            Ok(None)
        } else {
            Ok(Some(T::from_compact_string(s)?))
        }
    }
}
