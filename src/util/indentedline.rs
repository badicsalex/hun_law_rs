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

use std::{fmt::Debug, str::FromStr};

use lazy_regex::Regex;
use serde::{Deserialize, Serialize};

// Extremely scientific.
// Value worked well for the python version for hundreds of documents
const INDENT_SIMILARITY_THRESHOLD: f64 = 1.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndentedLinePart {
    pub dx: f64,
    pub content: char,
    pub bold: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndentedLine {
    parts: Vec<IndentedLinePart>,
    cached_content: String,
    cached_bold: bool,
}

impl IndentedLine {
    pub fn from_parts(parts: Vec<IndentedLinePart>) -> Self {
        let cached_content: String = parts.iter().map(|p| p.content).collect();
        let bold_character_count = parts.iter().filter(|p| p.bold).count();
        let cached_bold = bold_character_count * 2 > parts.len();
        IndentedLine {
            parts,
            cached_content,
            cached_bold,
        }
    }

    pub fn from_multiple(others: &[&Self]) -> Self {
        let mut result_parts = Vec::<IndentedLinePart>::new();
        let mut x = 0.0;
        let mut first;
        for other in others {
            first = true;
            for part in &other.parts {
                if first {
                    result_parts.push(IndentedLinePart {
                        dx: part.dx - x,
                        content: part.content,
                        bold: part.bold,
                    });
                    x = part.dx;
                    first = false;
                } else {
                    result_parts.push(part.clone());
                    x += part.dx;
                }
            }
        }
        Self::from_parts(result_parts)
    }

    pub fn indent(&self) -> f64 {
        match self.parts.first() {
            Some(part) => part.dx,
            None => 0.0,
        }
    }

    pub fn content(&self) -> &str {
        &self.cached_content
    }

    pub fn is_bold(&self) -> bool {
        self.cached_bold
    }

    pub fn len(&self) -> usize {
        self.parts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    pub fn slice(&self, from: i64, to: Option<i64>) -> IndentedLine {
        let len = self.len() as i64;
        let from = if from >= 0 { from } else { len + from };
        let from = from.clamp(0, len) as usize;
        let to = match to {
            Some(num) if num < 0 => len + num,
            Some(num) => num,
            None => len,
        };
        let to = to.clamp(from as i64, len) as usize;
        if to == from {
            return EMPTY_LINE;
        }

        let mut new_parts = self.parts[from..to].to_owned();

        let additional_indent: i64 = self
            .parts
            .iter()
            .take(from as usize)
            .map(|e| e.dx as i64)
            .sum();
        new_parts[0].dx += additional_indent as f64;
        Self::from_parts(new_parts.to_owned())
    }

    pub fn slice_bytes(&self, from: usize, to: Option<usize>) -> IndentedLine {
        let chars_from = self
            .content()
            .char_indices()
            .position(|(cp, _)| cp == from)
            .unwrap() as i64;
        let chars_to = to.map(|to_inner| {
            if to_inner >= self.content().as_bytes().len() {
                self.content().chars().count() as i64
            } else {
                self.content()
                    .char_indices()
                    .position(|(cp, _)| cp == to_inner)
                    .unwrap() as i64
            }
        });
        self.slice(chars_from, chars_to)
    }

    pub fn from_test_str(s: &str) -> Self {
        let mut parts = Vec::<IndentedLinePart>::new();
        let mut spaces_num = 1;
        let bold = s.contains("<BOLD>");
        let s = s.replace("<BOLD>", "      ");
        for c in s.chars() {
            if c == ' ' {
                if spaces_num == 0 {
                    parts.push(IndentedLinePart {
                        dx: 5.0,
                        content: c,
                        bold,
                    });
                }
                spaces_num += 1;
            } else {
                parts.push(IndentedLinePart {
                    dx: 5.0 + spaces_num as f64 * 5.0,
                    content: c,
                    bold,
                });
                spaces_num = 0
            }
        }
        Self::from_parts(parts)
    }

    pub fn indent_less_or_eq(&self, other: f64) -> bool {
        self.indent() < other + INDENT_SIMILARITY_THRESHOLD
    }

    pub fn parse_header<T: FromStr>(&self, regex: &Regex) -> Option<(T, IndentedLine)> {
        let content = self.content();
        let mut capture_locations = regex.capture_locations();
        // This is called for its side-effects, and the '?' is important.
        regex.captures_read(&mut capture_locations, content)?;

        let (identifier_from, identifier_to) = capture_locations.get(1).unwrap();
        let identifier: T = content[identifier_from..identifier_to]
            .to_string()
            .parse()
            .ok()?;
        let (content_from, content_to) =
            capture_locations.get(capture_locations.len() - 1).unwrap();
        let rest = self.slice_bytes(content_from, Some(content_to));
        Some((identifier, rest))
    }

    /// Appends this line to the string, using a space if necessary
    pub fn append_to(&self, s: &mut String) {
        if !self.is_empty() && !s.is_empty() && !s.ends_with('-') {
            s.push(' ');
        }
        s.push_str(self.content());
    }
}

impl PartialEq for IndentedLine {
    fn eq(&self, other: &Self) -> bool {
        self.indent().eq(&other.indent())
            && self.content() == other.content()
            && self.is_bold() == other.is_bold()
    }
}
impl Eq for IndentedLine {}

pub const EMPTY_LINE: IndentedLine = IndentedLine {
    parts: Vec::new(),
    cached_content: String::new(),
    cached_bold: false,
};

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_empty() {
        assert_eq!(EMPTY_LINE, IndentedLine::from_parts(vec![]));
        assert_eq!(IndentedLine::from_parts(vec![]), EMPTY_LINE);
        assert_eq!(EMPTY_LINE, IndentedLine::from_multiple(&[]));
        assert!(IndentedLine::from_parts(vec![]).is_empty());
        assert!(IndentedLine::from_multiple(&[]).is_empty());
        assert!(IndentedLine::from_multiple(&[&EMPTY_LINE, &EMPTY_LINE]).is_empty());
    }

    fn ilp(dx: f64, content: char) -> IndentedLinePart {
        IndentedLinePart {
            dx,
            content,
            bold: false,
        }
    }

    fn ilpb(dx: f64, content: char) -> IndentedLinePart {
        IndentedLinePart {
            dx,
            content,
            bold: true,
        }
    }

    #[test]
    fn test_indented_line_slice() {
        let line = IndentedLine::from_parts(vec![
            ilp(5.0, 'a'),
            ilp(5.0, 'b'),
            ilp(5.0, 'c'),
            ilp(1.0, 'd'),
            ilp(2.0, 'e'),
            ilp(2.0, ' '),
            ilp(5.0, 'f'),
        ]);
        assert_eq!(line.content(), "abcde f");
        assert_eq!(line.indent(), 5.0);

        assert_eq!(line.slice(0, None), line);

        assert_eq!(line.slice(1, None).content(), "bcde f");
        assert_eq!(line.slice(1, None).indent(), 10.0);

        assert_eq!(line.slice(2, None).content(), "cde f");
        assert_eq!(line.slice(2, None).indent(), 15.0);
        assert_eq!(line.slice(5, None).content(), " f");
        assert_eq!(line.slice(5, None).indent(), 20.0);

        assert_eq!(line.slice(7, None), EMPTY_LINE);
        assert_eq!(line.slice(100, None), EMPTY_LINE);

        assert_eq!(line.slice(-2, None).content(), " f");
        assert_eq!(line.slice(-2, None).indent(), 20.0);

        assert_eq!(line.slice(0, Some(-1)).content(), "abcde ");
        assert_eq!(line.slice(0, Some(-2)).content(), "abcde");
        assert_eq!(line.slice(0, Some(5)).content(), "abcde");

        assert_eq!(line.slice(1, Some(-1)).content(), "bcde ");
        assert_eq!(line.slice(2, Some(-2)).content(), "cde");
        assert_eq!(line.slice(2, Some(5)).content(), "cde");
        assert_eq!(line.slice(2, Some(5)).indent(), 15.0);
        assert_eq!(line.slice(-2, Some(-1)).content(), " ");

        assert_eq!(line.slice(1, Some(1)), EMPTY_LINE);
        assert_eq!(line.slice(5, Some(3)), EMPTY_LINE);
    }

    #[test]
    fn test_indented_line_from_multiple() {
        let line1 = IndentedLine::from_parts(vec![ilp(5.0, 'a'), ilp(5.0, 'b'), ilp(5.0, 'c')]);
        let line2 = IndentedLine::from_parts(vec![ilp(25.0, 'a'), ilp(5.0, 'b'), ilp(5.0, 'c')]);
        let concatenated = IndentedLine::from_multiple(&[&line1, &line2]);
        assert_eq!(concatenated.content(), "abcabc");
        assert_eq!(concatenated.indent(), 5.0);
        assert_eq!(concatenated.slice(3, None).content(), "abc");

        assert_eq!(concatenated.slice(2, None).indent(), 15.0);
        assert_eq!(concatenated.slice(3, None).indent(), 25.0);
        assert_eq!(concatenated.slice(4, None).indent(), 30.0);

        let big_conc = IndentedLine::from_multiple(&[
            &IndentedLine::from_parts(vec![ilp(5.0, 'a'), ilp(5.0, 'b'), ilp(5.0, 'c')]),
            &IndentedLine::from_parts(vec![ilp(25.0, 'a'), ilp(5.0, 'b'), ilp(5.0, 'c')]),
            &IndentedLine::from_parts(vec![ilp(45.0, 'a'), ilp(5.0, 'b'), ilp(5.0, 'c')]),
            &IndentedLine::from_parts(vec![ilp(65.0, 'a'), ilp(5.0, 'b'), ilp(5.0, 'c')]),
        ]);

        println!("{:?}", big_conc);
        assert_eq!(big_conc.slice(8, None).indent(), 55.0);
        assert_eq!(big_conc.slice(9, None).indent(), 65.0);
        assert_eq!(big_conc.slice(10, None).indent(), 70.0);
        assert_eq!(big_conc.len(), 12);

        for i in 0..11 {
            let slice1 = big_conc.slice(0, Some(i));
            let slice2 = big_conc.slice(i, None);
            let concatenated_2 = IndentedLine::from_multiple(&[&slice1, &slice2]);
            let reslice2 = concatenated_2.slice(i, None);

            assert_eq!(concatenated_2, big_conc);
            assert_eq!(slice2, reslice2);
        }
    }

    #[test]
    fn test_boldness() {
        assert!(!IndentedLine::from_parts(vec![ilp(25.0, 'a')]).is_bold());
        assert!(IndentedLine::from_parts(vec![ilpb(25.0, 'a')]).is_bold());

        let half_bold = IndentedLine::from_parts(vec![
            ilp(5.0, 'a'),
            ilp(5.0, 'b'),
            ilpb(5.0, 'c'),
            ilpb(1.0, 'd'),
        ]);
        assert!(!half_bold.is_bold());

        let more_than_half_bold = IndentedLine::from_parts(vec![
            ilp(25.0, 'a'),
            ilp(5.0, 'b'),
            ilpb(5.0, 'c'),
            ilpb(1.0, 'd'),
            ilpb(1.0, '2'),
        ]);
        assert!(more_than_half_bold.is_bold());

        let spliced = IndentedLine::from_multiple(&[&half_bold, &more_than_half_bold]);
        assert!(spliced.is_bold());
        assert!(!spliced.slice(0, Some(-1)).is_bold());
        assert!(spliced.slice(1, Some(-1)).is_bold());
        assert!(spliced.slice(2, Some(5)).is_bold());
    }

    #[test]
    fn test_from_test_str() {
        assert_eq!(
            IndentedLine::from_test_str("    Lol ez   mi?"),
            IndentedLine::from_parts(vec![
                ilp(30.0, 'L'),
                ilp(5.0, 'o'),
                ilp(5.0, 'l'),
                ilp(5.0, ' '),
                ilp(5.0, 'e'),
                ilp(5.0, 'z'),
                ilp(5.0, ' '),
                ilp(10.0, 'm'),
                ilp(5.0, 'i'),
                ilp(5.0, '?'),
            ])
        );
        assert_eq!(
            IndentedLine::from_test_str(" <BOLD> bld"),
            IndentedLine::from_parts(vec![ilpb(50.0, 'b'), ilpb(5.0, 'l'), ilpb(5.0, 'd'),])
        )
    }

    #[test]
    fn test_slice_bytes() {
        let line = IndentedLine::from_parts(vec![
            IndentedLinePart {
                dx: 75.0,
                content: '2',
                bold: false,
            },
            IndentedLinePart {
                dx: 5.0,
                content: ':',
                bold: false,
            },
            IndentedLinePart {
                dx: 5.0,
                content: '2',
                bold: false,
            },
            IndentedLinePart {
                dx: 5.0,
                content: '.',
                bold: false,
            },
            IndentedLinePart {
                dx: 5.0,
                content: ' ',
                bold: false,
            },
            IndentedLinePart {
                dx: 10.0,
                content: '??',
                bold: false,
            },
            IndentedLinePart {
                dx: 5.0,
                content: ' ',
                bold: false,
            },
            IndentedLinePart {
                dx: 10.0,
                content: '[',
                bold: false,
            },
            IndentedLinePart {
                dx: 5.0,
                content: 'D',
                bold: false,
            },
            IndentedLinePart {
                dx: 5.0,
                content: 'u',
                bold: false,
            },
            IndentedLinePart {
                dx: 5.0,
                content: 'm',
                bold: false,
            },
            IndentedLinePart {
                dx: 5.0,
                content: 'm',
                bold: false,
            },
            IndentedLinePart {
                dx: 5.0,
                content: 'y',
                bold: false,
            },
            IndentedLinePart {
                dx: 5.0,
                content: ' ',
                bold: false,
            },
            IndentedLinePart {
                dx: 10.0,
                content: 't',
                bold: false,
            },
            IndentedLinePart {
                dx: 5.0,
                content: 'i',
                bold: false,
            },
            IndentedLinePart {
                dx: 5.0,
                content: 't',
                bold: false,
            },
            IndentedLinePart {
                dx: 5.0,
                content: 'l',
                bold: false,
            },
            IndentedLinePart {
                dx: 5.0,
                content: 'e',
                bold: false,
            },
            IndentedLinePart {
                dx: 5.0,
                content: ']',
                bold: false,
            },
        ]);
        assert_eq!(&"2:2. ?? [Dummy title]"[8..21], "[Dummy title]");
        assert_eq!(line.content(), "2:2. ?? [Dummy title]");
        assert_eq!(line.slice_bytes(8, Some(21)).content(), "[Dummy title]");
        assert_eq!(line.slice_bytes(8, None).content(), "[Dummy title]");
        assert_eq!(line.slice_bytes(8, Some(15)).content(), "[Dummy ");
        assert_eq!(line.slice_bytes(2, Some(15)).content(), "2. ?? [Dummy ");
    }
}
