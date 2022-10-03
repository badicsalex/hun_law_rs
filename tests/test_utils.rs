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
    fmt::Display,
    fs::File,
    io::{self, Read},
    path::Path,
};

use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use colored::*;
use hun_law::{
    identifier::ActIdentifier,
    parser::{mk_act_section::ActRawText, structure::parse_act_structure},
    structure::{Act, ParagraphChildren, SAEBody},
    util::{indentedline::IndentedLine, singleton_yaml},
};
use serde::Serialize;
use similar::{ChangeTag, TextDiff};
pub use tempfile::TempDir;

pub fn read_all(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let mut result = Vec::new();
    File::open(path)?.read_to_end(&mut result)?;
    Ok(result)
}

struct PrettyDiff {
    pub left: String,
    pub right: String,
}

impl Display for PrettyDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let diff = TextDiff::from_lines(&self.left, &self.right);

        for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
            if idx > 0 {
                writeln!(f, "{:-^1$}", "-", 80)?;
            }
            for op in group {
                for change in diff.iter_inline_changes(op) {
                    let tag = match change.tag() {
                        ChangeTag::Delete => "-".red(),
                        ChangeTag::Insert => "+".green(),
                        ChangeTag::Equal => " ".white(),
                    };
                    tag.fmt(f)?;

                    for (emphasized, value) in change.iter_strings_lossy() {
                        if emphasized {
                            write!(
                                f,
                                "{}",
                                value.color(tag.fgcolor().unwrap()).underline().bold()
                            )?;
                        } else {
                            write!(f, "{}", value.color(tag.fgcolor().unwrap()))?;
                        }
                    }
                    if change.missing_newline() {
                        writeln!(f)?;
                    }
                }
            }
        }
        Ok(())
    }
}

pub fn ensure_eq<T, U>(expected: &T, actual: &U, message: &str) -> Result<()>
where
    T: Serialize + ?Sized + PartialEq<U>,
    U: Serialize + ?Sized,
{
    // This is duplicated from ensure_eq, but that's because the structures may be 'equal' even
    // when ther YML form is not.
    if expected != actual {
        Err(anyhow!(
            "{}\n{}",
            message,
            PrettyDiff {
                left: singleton_yaml::to_string(expected).unwrap(),
                right: singleton_yaml::to_string(actual).unwrap(),
            }
        ))
    } else {
        Ok(())
    }
}

pub fn to_indented_lines(data: &[u8]) -> Vec<IndentedLine> {
    std::str::from_utf8(data)
        .unwrap()
        .split('\n')
        .map(IndentedLine::from_test_str)
        .collect()
}

pub fn parse_txt_as_act(path: &Path) -> Result<Act> {
    let data_as_lines = to_indented_lines(&read_all(path)?);
    parse_act_structure(&ActRawText {
        identifier: ActIdentifier {
            year: 2345,
            number: 0xd,
        },
        subject: "A tesztelésről".to_string(),
        publication_date: NaiveDate::from_ymd(2345, 6, 7),
        body: data_as_lines,
    })
}

// clean out the quoted blocks' contents, because we don't want to
// pollute the test yamls with serialized indented lines
pub fn clean_quoted_blocks(act: &mut Act) {
    for article in act.articles_mut() {
        for paragraph in &mut article.children {
            if let SAEBody::Children {
                children: ParagraphChildren::QuotedBlock(qbs),
                ..
            } = &mut paragraph.body
            {
                for qb in qbs {
                    qb.lines = vec![]
                }
            }
        }
    }
}
