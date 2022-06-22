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

mod act;
mod alphabetic;
mod article;
mod numeric;
mod prefixed_alphabetic;

pub use act::ActIdentifier;
pub use alphabetic::{AlphabeticIdentifier, HungarianIdentifierChar};
pub use article::ArticleIdentifier;
pub use numeric::NumericIdentifier;
pub use prefixed_alphabetic::PrefixedAlphabeticIdentifier;

pub trait IsNextFrom {
    fn is_first(&self) -> bool;

    /// Can the parameter be considered the previous identifier. Handles suffix transitions.
    ///
    /// ```
    /// use hun_law::structure::{NumericIdentifier, IsNextFrom};
    /// let id121:NumericIdentifier = 121.into();
    /// let id122:NumericIdentifier = 122.into();
    /// let id123:NumericIdentifier = 123.into();
    /// assert!(id122.is_next_from(id121));
    /// assert!(!id123.is_next_from(id121));
    /// assert!(!id121.is_next_from(id122));
    /// ```
    fn is_next_from(&self, other: Self) -> bool;
}

// TODO: Hacks until we have better IDs or something.
impl IsNextFrom for Option<NumericIdentifier> {
    fn is_first(&self) -> bool {
        match self {
            Some(x) => x.is_first(),
            None => false,
        }
    }

    fn is_next_from(&self, other: Self) -> bool {
        match (self, other) {
            (Some(x), Some(o)) => x.is_next_from(o),
            _ => false,
        }
    }
}

impl IsNextFrom for String {
    fn is_first(&self) -> bool {
        todo!()
    }

    fn is_next_from(&self, _other: Self) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, str::FromStr};

    use super::*;

    fn check_is_next_from<T>(s1: &str, s2: &str) -> bool
    where
        T: FromStr + IsNextFrom,
        T::Err: Debug,
    {
        let i1 = s1.parse::<T>().unwrap();
        let i2 = s2.parse::<T>().unwrap();
        i1.is_next_from(i2)
    }

    #[test]
    fn test_is_next_from_numeric() {
        assert!(check_is_next_from::<NumericIdentifier>("123", "122"));
        assert!(check_is_next_from::<NumericIdentifier>("123/A", "123"));
        assert!(check_is_next_from::<NumericIdentifier>("123zs", "123z"));
        assert!(check_is_next_from::<NumericIdentifier>("13", "12c"));

        assert!(!check_is_next_from::<NumericIdentifier>("12b", "12"));
        assert!(!check_is_next_from::<NumericIdentifier>("12a", "11"));
        assert!(!check_is_next_from::<NumericIdentifier>("13", "11"));
        assert!(!check_is_next_from::<NumericIdentifier>("11", "11"));
        assert!(!check_is_next_from::<NumericIdentifier>("11", "12"));
    }

    #[test]
    fn test_is_next_from_article() {
        assert!(check_is_next_from::<ArticleIdentifier>("1:123", "1:122"));
        assert!(check_is_next_from::<ArticleIdentifier>("2:1", "1:123"));
        assert!(check_is_next_from::<ArticleIdentifier>("13", "12c"));

        assert!(!check_is_next_from::<ArticleIdentifier>("1:1", "2:2"));
        assert!(!check_is_next_from::<ArticleIdentifier>("1:1", "3:1"));
        assert!(!check_is_next_from::<ArticleIdentifier>("2:1a", "1:123"));

        // Book <-> no book transitions not allowed
        assert!(!check_is_next_from::<ArticleIdentifier>("1:1", "1"));
        assert!(!check_is_next_from::<ArticleIdentifier>("1", "1:1"));
        assert!(!check_is_next_from::<ArticleIdentifier>("1:1", "2"));
        assert!(!check_is_next_from::<ArticleIdentifier>("1", "1:2"));
    }

    #[test]
    fn test_is_next_from_alphabetic() {
        let a: HungarianIdentifierChar = 'a'.try_into().unwrap();
        let b: HungarianIdentifierChar = 'B'.try_into().unwrap();
        let n: HungarianIdentifierChar = 'n'.try_into().unwrap();
        let ny: HungarianIdentifierChar = "Ny".parse().unwrap();
        let o: HungarianIdentifierChar = "o".parse().unwrap();
        assert!(b.is_next_from(a));
        assert!(!a.is_next_from(b));
        assert!(!a.is_next_from(a));
        assert!(ny.is_next_from(n));
        assert!(!ny.is_next_from(ny));
        assert!(!n.is_next_from(ny));
        assert!(o.is_next_from(ny));
    }

    #[test]
    fn test_is_next_from_prefixed_alphabetic() {
        assert!(check_is_next_from::<PrefixedAlphabeticIdentifier>("b", "a"));
        assert!(check_is_next_from::<PrefixedAlphabeticIdentifier>(
            "ab", "aa"
        ));
        assert!(check_is_next_from::<PrefixedAlphabeticIdentifier>("l", "k"));
        assert!(check_is_next_from::<PrefixedAlphabeticIdentifier>(
            "cl", "ck"
        ));

        assert!(!check_is_next_from::<PrefixedAlphabeticIdentifier>(
            "c", "c"
        ));
        assert!(!check_is_next_from::<PrefixedAlphabeticIdentifier>(
            "ca", "db"
        ));
        assert!(!check_is_next_from::<PrefixedAlphabeticIdentifier>(
            "c", "bb"
        ));
        assert!(!check_is_next_from::<PrefixedAlphabeticIdentifier>(
            "cc", "b"
        ));
    }
}
