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
use anyhow::{bail, Result};

pub mod date;
pub mod indentedline;
use indentedline::IndentedLine;

pub trait IsDefault {
    fn is_default(&self) -> bool;
}

impl<T> IsDefault for Vec<T> {
    fn is_default(&self) -> bool {
        self.is_empty()
    }
}

impl<T> IsDefault for Option<T> {
    fn is_default(&self) -> bool {
        self.is_none()
    }
}

impl IsDefault for bool {
    fn is_default(&self) -> bool {
        self == &false
    }
}

impl IsDefault for String {
    fn is_default(&self) -> bool {
        self.is_empty()
    }
}

impl IsDefault for f64 {
    fn is_default(&self) -> bool {
        self == &0f64
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

#[derive(Debug, Default)]
#[must_use]
pub struct QuoteCheck {
    quote_level: i64,
    pub beginning_is_quoted: bool,
    pub end_is_quoted: bool,
}

impl QuoteCheck {
    pub fn update(&mut self, line: &IndentedLine) -> Result<()> {
        self.beginning_is_quoted = self.quote_level > 0;
        self.quote_level += line.content().matches(['???', '???']).count() as i64;
        self.quote_level -= line.content().matches('???').count() as i64;

        if self.quote_level < 0 {
            bail!(
                "Malformed quoting. (Quote_level = {}, line='{}')",
                self.quote_level,
                line.content()
            )
        }
        self.end_is_quoted = self.quote_level > 0;
        Ok(())
    }

    pub fn check_end(self) -> Result<()> {
        if self.quote_level != 0 {
            bail!("Unclosed quoting. (Quote_level = {})", self.quote_level)
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{indentedline::IndentedLinePart, *};
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
        assert_eq!(str_to_int_hun("Huszon??t??dik").unwrap(), 25);
        assert_eq!(int_to_str_hun(25).unwrap(), "huszon??t??dik");
    }

    fn line_from_str(s: &str) -> IndentedLine {
        IndentedLine::from_parts(
            s.chars()
                .map(|c| IndentedLinePart {
                    dx: 5.0,
                    content: c,
                    bold: false,
                })
                .collect(),
        )
    }

    #[test]
    fn test_quote_check() {
        let mut quote_check_1 = QuoteCheck::default();
        quote_check_1.update(&line_from_str("A b c")).unwrap();
        assert!(!quote_check_1.beginning_is_quoted);
        assert!(!quote_check_1.end_is_quoted);

        quote_check_1.update(&line_from_str("???Abc")).unwrap();
        assert!(!quote_check_1.beginning_is_quoted);
        assert!(quote_check_1.end_is_quoted);

        quote_check_1.update(&line_from_str("Abcd???")).unwrap();
        assert!(quote_check_1.beginning_is_quoted);
        assert!(!quote_check_1.end_is_quoted);

        quote_check_1.update(&line_from_str("Ab???c???d")).unwrap();
        assert!(!quote_check_1.beginning_is_quoted);
        assert!(!quote_check_1.end_is_quoted);

        quote_check_1.update(&line_from_str("Ab ??? ??? cd")).unwrap();
        assert!(!quote_check_1.beginning_is_quoted);
        assert!(quote_check_1.end_is_quoted);

        quote_check_1.update(&line_from_str("Ab???c???d")).unwrap();
        assert!(quote_check_1.beginning_is_quoted);
        assert!(quote_check_1.end_is_quoted);

        quote_check_1.update(&line_from_str("Ab ??? ???")).unwrap();
        assert!(quote_check_1.beginning_is_quoted);
        assert!(!quote_check_1.end_is_quoted);

        quote_check_1.check_end().unwrap();

        // Error case: negative quote level

        let mut quote_check_2 = QuoteCheck::default();
        quote_check_2.update(&line_from_str("A b c")).unwrap();
        assert!(quote_check_2.update(&line_from_str("Abcd???")).is_err());
        assert!(quote_check_2.check_end().is_err());

        // Error case: unclosed quote

        let mut quote_check_3 = QuoteCheck::default();
        quote_check_3.update(&line_from_str("A b c")).unwrap();
        quote_check_3.update(&line_from_str("Abcd???")).unwrap();
        assert!(quote_check_3.check_end().is_err());
    }
}
