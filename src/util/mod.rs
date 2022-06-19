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

pub mod date;
pub mod indentedline;

pub trait IsDefault {
    fn is_default(&self) -> bool;
}

impl<T> IsDefault for T
where
    T: Default + PartialEq,
{
    fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

pub fn is_default<T: IsDefault>(t: &T) -> bool {
    t.is_default()
}

mod generated {
    include!(concat!(env!("OUT_DIR"), "/phf_generated.rs"));
}

pub fn int_to_str_hun(i: u16) -> Option<&'static str> {
    generated::INT_TO_STR_HUN.get(i as usize).copied()
}

pub fn str_to_int_hun(s: &str) -> Option<u16> {
    generated::STR_TO_INT_HUN.get(s).copied()
}

pub const DIGITS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
pub const ROMAN_DIGITS: [char; 7] = ['I', 'V', 'X', 'L', 'C', 'D', 'M'];

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_int_to_str() {
        for i in 0..100 {
            assert_eq!(
                str_to_int_hun(int_to_str_hun(i as u16).unwrap()).unwrap(),
                i
            );
            assert_eq!(
                str_to_int_hun(&int_to_str_hun(i as u16).unwrap().to_uppercase()).unwrap(),
                i
            );
        }
        assert_eq!(str_to_int_hun("Huszonötödik").unwrap(), 25);
        assert_eq!(int_to_str_hun(25).unwrap(), "huszonötödik");
    }
}
