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

use anyhow::{Context, Error};

pub trait DebugContextString {
    /// Override this with something that returns something like "Article 5"
    fn debug_ctx(&self) -> String;
}

pub trait WithElemContext<T, E> {
    fn with_elem_context(self, msg: &str, elem: &impl DebugContextString) -> Result<T, Error>;
}

impl<T, E> WithElemContext<T, E> for Result<T, E>
where
    Self: Context<T, E>,
    E: Send + Sync + 'static,
{
    fn with_elem_context(self, msg: &str, elem: &impl DebugContextString) -> Result<T, Error> {
        self.with_context(|| format!("{} in {}", msg, elem.debug_ctx()))
    }
}
