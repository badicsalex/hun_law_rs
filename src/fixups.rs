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
    fs::{create_dir_all, File},
    path::PathBuf,
};

use anyhow::{anyhow, ensure, Result};
use serde::{Deserialize, Serialize};

use crate::{
    identifier::ActIdentifier,
    util::{
        debug::{DebugContextString, WithElemContext},
        indentedline::{IndentedLine, IndentedLinePart, EMPTY_LINE},
        is_default,
    },
};

const REPLACEMENT_FAKE_WIDTH: f64 = 10.0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fixup {
    #[serde(default, skip_serializing_if = "is_default")]
    pub after: Vec<String>,
    pub old: String,
    pub new: String,
}

#[derive(Debug, Clone)]
pub struct Fixups {
    fixups: Vec<Fixup>,
    fixup_path: PathBuf,
}

impl Fixups {
    pub fn load(act_id: ActIdentifier) -> Result<Self> {
        Self::load_from(act_id, "./data/fixups/".into())
    }

    pub fn load_from(act_id: ActIdentifier, base_dir: PathBuf) -> Result<Self> {
        let fixup_path = base_dir
            .join(act_id.year.to_string())
            .join(format!("{}.yml", act_id));
        let fixups = if fixup_path.exists() {
            serde_yaml::from_reader(File::open(&fixup_path)?)?
        } else {
            Vec::new()
        };
        Ok(Self { fixups, fixup_path })
    }

    pub fn add(&mut self, f: Fixup) {
        self.fixups.push(f);
    }

    pub fn save(&self) -> Result<()> {
        create_dir_all(
            &self
                .fixup_path
                .parent()
                .ok_or_else(|| anyhow!("No parent for fixup_path"))?,
        )?;
        serde_yaml::to_writer(&mut File::create(&self.fixup_path)?, &self.fixups)?;
        Ok(())
    }

    pub fn apply(&self, lines: &mut [IndentedLine]) -> Result<()> {
        for fixup in &self.fixups {
            fixup.apply(lines)?
        }
        Ok(())
    }
}

impl From<Fixups> for Vec<Fixup> {
    fn from(f: Fixups) -> Self {
        f.fixups
    }
}

impl Fixup {
    pub fn apply(&self, lines: &mut [IndentedLine]) -> Result<()> {
        let mut needle = self.after.clone();
        needle.push(self.old.clone());
        let lines_as_strs: Vec<&str> = lines.iter().map(|l| l.content()).collect();
        let position = lines_as_strs
            .windows(needle.len())
            .position(|w| w == needle)
            .ok_or_else(|| anyhow!("Could not find '{}' in text", self.old))?
            + self.after.len();

        let found_places = lines_as_strs
            .windows(needle.len())
            .filter(|w| *w == needle)
            .count();
        ensure!(
            found_places == 1,
            "Replacement 'old' text ('{}') found too many ({:?}) times.",
            self.old,
            found_places
        );

        lines[position] = self
            .apply_to_line(&lines[position])
            .with_elem_context("Could not apply fixup", self)?;
        Ok(())
    }

    fn apply_to_line(&self, line: &IndentedLine) -> Result<IndentedLine> {
        // Should never happen
        ensure!(
            line.content() == self.old,
            "Erroneous call to apply_to_line"
        );
        ensure!(self.old != self.new, "Useless fixup (old == new)");
        if self.new.is_empty() {
            return Ok(EMPTY_LINE);
        }

        let prefix_len = self
            .old
            .chars()
            .zip(self.new.chars())
            .take_while(|(o, n)| o == n)
            .count() as i64;

        // This is needed in the case of e.g. 'aaa' -> 'aa', where both prefix and postifx would be 2.
        let rest_of_old: String = self.old.chars().skip(prefix_len as usize).collect();
        let postfix_len = rest_of_old
            .chars()
            .rev()
            .zip(self.new.chars().rev())
            .take_while(|(o, n)| o == n)
            .count() as i64;

        let old_len = self.old.chars().count() as i64;
        let replacement_indent_start = if prefix_len >= old_len {
            // Pure appending. Unfortunately indent_at will give us the indent of the last character,
            // so we have to offset to the right a bit
            line.indent_at(prefix_len) + REPLACEMENT_FAKE_WIDTH * 0.5
        } else if prefix_len > 0 {
            if postfix_len > 0 && prefix_len + postfix_len >= old_len {
                // Looks like a pure insertion in the middle.
                // replacement_indent_end will be set to basically
                // line.indent_at(prefix_len)
                // so we have to squeeze between a bit.
                (line.indent_at(prefix_len - 1) + line.indent_at(prefix_len)) * 0.5
            } else {
                // We can start at the character after the prefix, since it will
                // be replaced anyway
                line.indent_at(prefix_len)
            }
        } else if postfix_len > 0 {
            line.indent_at(-(postfix_len)) - REPLACEMENT_FAKE_WIDTH
        } else {
            // Complete replacement
            line.indent()
        };

        // The replacement string will never reach this exact indent, it will stop one 'step' short.
        let replacement_indent_end = if postfix_len > 0 {
            line.indent_at(-(postfix_len))
        } else if prefix_len > 0 {
            line.indent_at(10000) + REPLACEMENT_FAKE_WIDTH
        } else if line.is_empty() {
            // Complete replacement from emtpy line
            REPLACEMENT_FAKE_WIDTH
        } else {
            // Complete replacement
            line.indent_at(10000)
        };

        let replacement_str_len =
            (self.new.chars().count() as i64 - prefix_len - postfix_len) as usize;

        let replacement_str: String = self
            .new
            .chars()
            .skip(prefix_len as usize)
            .take(replacement_str_len)
            .collect();
        let parts = replacement_str
            .chars()
            .enumerate()
            .map(|(i, content)| IndentedLinePart {
                dx: if i == 0 {
                    replacement_indent_start
                } else {
                    // This is not divide by zero, because lenght is at least 1 at this point.
                    (replacement_indent_end - replacement_indent_start)
                        / (replacement_str_len as f64)
                },
                content,
                // XXX: this is not exactly correct. Maybe get the boldness of what we are replacing?
                bold: line.is_bold(),
            })
            .collect();
        let replacement = IndentedLine::from_parts(parts, false);

        let prefix = line.slice(0, Some(prefix_len));
        let postfix = if postfix_len == 0 {
            EMPTY_LINE
        } else {
            // Unfortunately line.slice(-0, None) != EMPTY_LINE
            line.slice(-postfix_len, None)
        };

        Ok(IndentedLine::from_multiple(&[
            &prefix,
            &replacement,
            &postfix,
        ]))
    }
}

impl DebugContextString for Fixup {
    fn debug_ctx(&self) -> String {
        format!("'{}' -> '{}'", self.old, self.new)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn ilp(dx: f64, content: char) -> IndentedLinePart {
        IndentedLinePart {
            dx,
            content,
            bold: false,
        }
    }

    fn test_line() -> IndentedLine {
        IndentedLine::from_parts(
            vec![
                ilp(25.4, 'á'),
                ilp(5.6, 'ű'),
                ilp(5.7, 'é'),
                ilp(1.8, 'ú'),
                ilp(2.0, 'ó'),
            ],
            true,
        )
    }

    fn fixup_line(line: &IndentedLine, new: &str) -> IndentedLine {
        let result = Fixup {
            after: vec![],
            old: line.content().to_owned(),
            new: new.to_owned(),
        }
        .apply_to_line(line)
        .unwrap();
        check_monotonicity(&result);
        result
    }

    fn fixup(new: &str) -> IndentedLine {
        let line = test_line();
        fixup_line(&line, new)
    }

    fn check_monotonicity(line: &IndentedLine) {
        let mut previous_indent = -1.0;
        for i in 0..line.len() {
            let indent = line.indent_at(i as i64);
            assert!(
                previous_indent < indent,
                "Indentation was not strictly monotonic at char {:?} ({}) of {}. Previous: {:?}, Current: {:?}",
                i,
                line.content().chars().nth(i).unwrap(),
                line.content(),
                previous_indent,
                indent,

            );
            previous_indent = indent;
        }
    }

    #[test]
    fn test_middle() {
        assert_eq!(fixup("áűÍúó").content(), "áűÍúó", "Replaced same width");
        assert_eq!(fixup("áűÍÍÍúó").content(), "áűÍÍÍúó", "Replaced bigger");
        assert_eq!(fixup("áűÍó").content(), "áűÍó", "Replaced smoller (right)");
        assert_eq!(fixup("áÍúó").content(), "áÍúó", "Replaced smoller (left)");
        assert_eq!(fixup("áÍó").content(), "áÍó", "Replaced smoller (both)");
        assert_eq!(
            fixup("áűÍÍÍó").content(),
            "áűÍÍÍó",
            "Replaced bigger (right)"
        );
        assert_eq!(
            fixup("áÍÍÍúó").content(),
            "áÍÍÍúó",
            "Replaced bigger (left)"
        );
        assert_eq!(fixup("áÍÍÍó").content(), "áÍÍÍó", "Replaced bigger (both)");
        assert_eq!(fixup("áűúó").content(), "áűúó", "Complete remove");
        assert_eq!(fixup("áó").content(), "áó", "Complete remove");

        assert_eq!(fixup("áűéÍÍúó").content(), "áűéÍÍúó", "Pure insertion");
    }

    #[test]
    fn test_left() {
        assert_eq!(fixup("űéúó").content(), "űéúó", "Remove left");
        assert_eq!(fixup("úó").content(), "úó", "Remove left 2");

        assert_eq!(fixup("Íűéúó").content(), "Íűéúó", "Replace left");
        assert_eq!(
            fixup("ÍÍÍűéúó").content(),
            "ÍÍÍűéúó",
            "Replace left (bigger)"
        );
        assert_eq!(fixup("Íúó").content(), "Íúó", "Replace left (smoller)");
        assert_eq!(fixup("Íáűéúó").content(), "Íáűéúó", "Append left");
    }

    #[test]
    fn test_right() {
        assert_eq!(fixup("áűéú").content(), "áűéú", "Remove right");
        assert_eq!(fixup("áű").content(), "áű", "Remove right 2");

        assert_eq!(fixup("áűéúÍ").content(), "áűéúÍ", "Replace right");
        assert_eq!(
            fixup("áűéúÍÍÍ").content(),
            "áűéúÍÍÍ",
            "Replace right (bigger)"
        );
        assert_eq!(fixup("áűéÍ").content(), "áűéÍ", "Replace right (smoller)");
        assert_eq!(fixup("áűéúóÍ").content(), "áűéúóÍ", "Append right");
    }

    #[test]
    fn test_full_replace() {
        assert_eq!(fixup("ÍÍÍÍÍ").content(), "ÍÍÍÍÍ", "Same length");
        assert_eq!(fixup("ÍÍ").content(), "ÍÍ", "Smoller");
        assert_eq!(fixup("ÍÍÍÍÍÍÍÍÍÍ").content(), "ÍÍÍÍÍÍÍÍÍÍ", "Bigger");

        assert_eq!(fixup(""), EMPTY_LINE, "Emptify");
    }

    #[test]
    fn test_indents() {
        // Cases:
        // prefix >  0, postfix > 0
        let line = fixup("áűÍÍÍúó");
        assert_eq!(line.indent_at(0), test_line().indent_at(0));
        assert_eq!(line.indent_at(1), test_line().indent_at(1));
        assert_eq!(line.indent_at(2), test_line().indent_at(2));
        assert_eq!(line.indent_at(5), test_line().indent_at(3));
        assert_eq!(line.indent_at(6), test_line().indent_at(4));
        // prefix >  0, postfix > 0, pure insertion
        let line = fixup("áűéÍÍÍúó");
        assert_eq!(line.indent_at(0), test_line().indent_at(0));
        assert_eq!(line.indent_at(1), test_line().indent_at(1));
        assert_eq!(line.indent_at(2), test_line().indent_at(2));
        assert_eq!(line.indent_at(6), test_line().indent_at(3));
        assert_eq!(line.indent_at(7), test_line().indent_at(4));

        // prefix == 0, postfix > 0
        let line = fixup("ÍÍÍúó");
        assert!(line.indent() > 0.0);
        assert_eq!(line.indent_at(3), test_line().indent_at(3));
        assert_eq!(line.indent_at(4), test_line().indent_at(4));
        // prefix >  0, postfix == 0
        let line = fixup("áűÍÍÍ");
        assert_eq!(line.indent_at(0), test_line().indent_at(0));
        assert_eq!(line.indent_at(1), test_line().indent_at(1));
        assert_eq!(line.indent_at(2), test_line().indent_at(2));
        // prefix >  0, postfix == 0, pure appending
        let line = fixup("áűéúóÍÍÍ");
        assert_eq!(line.indent_at(0), test_line().indent_at(0));
        assert_eq!(line.indent_at(1), test_line().indent_at(1));
        assert_eq!(line.indent_at(2), test_line().indent_at(2));
        assert_eq!(line.indent_at(3), test_line().indent_at(3));
        assert_eq!(line.indent_at(4), test_line().indent_at(4));
        // prefix == 0, postfix == 0, nonempty
        let line = fixup("ÍÍÍ");
        assert_eq!(line.indent(), test_line().indent());
    }

    #[test]
    fn test_justified() {
        assert!(fixup("áűÍúó").is_justified(), "Replaced same width");
        assert!(fixup("űéúó").is_justified(), "Remove left");
        assert!(!fixup("áűéú").is_justified(), "Remove right");

        // Maybe this should be justified though?
        assert!(!fixup("ÍÍÍÍÍ").is_justified(), "Same length");
    }

    #[test]
    fn test_bold() {
        assert!(!fixup("áűÍúó").is_bold(), "Replaced same width");
        assert!(!fixup("űéúó").is_bold(), "Remove left");
        assert!(!fixup("áűéú").is_bold(), "Remove right");
        assert!(!fixup("ÍÍÍÍÍ").is_bold(), "Same length");

        let bold_line = IndentedLine::from_parts(
            vec![
                IndentedLinePart {
                    dx: 25.5,
                    content: 'a',
                    bold: true,
                },
                IndentedLinePart {
                    dx: 15.4,
                    content: 'b',
                    bold: true,
                },
            ],
            false,
        );
        assert!(fixup_line(&bold_line, "aÍÍÍÍb").is_bold());
        assert!(fixup_line(&bold_line, "abÍÍÍÍb").is_bold());
        assert!(fixup_line(&bold_line, "aÍÍÍÍ").is_bold());
        assert!(fixup_line(&bold_line, "ÍÍÍÍab").is_bold());
        assert!(fixup_line(&bold_line, "ÍÍÍÍb").is_bold());
        assert!(fixup_line(&bold_line, "ÍÍÍÍ").is_bold());
        assert!(fixup_line(&bold_line, "a").is_bold());
    }

    #[test]
    fn test_from_empty() {
        let result = Fixup {
            after: vec![],
            old: String::new(),
            new: "ÍÍÍÍÍ".to_owned(),
        }
        .apply_to_line(&EMPTY_LINE)
        .unwrap();
        check_monotonicity(&result);
        assert_eq!(result.content(), "ÍÍÍÍÍ");
        assert!(result.indent() > 0.0);
        assert!(!result.is_bold());
        assert!(!result.is_justified());
    }
}
