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

use std::io::{Read, Seek, SeekFrom, Write};

use anyhow::{anyhow, ensure, Result};
use hun_law::{
    fixups::{Fixup, Fixups},
    output::quick_display_indented_line,
    parser::mk_act_section::ActRawText,
    util::QuoteCheck,
};
use log::info;

pub fn run_fixup_editor(act: &ActRawText, editor: &str) -> Result<()> {
    let mut temp_file = tempfile::Builder::new()
        .prefix(&act.identifier.to_string())
        .suffix(".txt")
        .tempfile()?;
    let mut quote_check = QuoteCheck::default();
    let mut first_error = None;
    let mut line_after_last_unquoted = None;
    for (i, line) in act.body.iter().enumerate() {
        ensure!(
            !line.content().ends_with(' '),
            "All lines must be rtrimmed, or else modification detection does not work"
        );
        ensure!(
            !line.content().starts_with(' '),
            "All lines must be ltrimmed, or else modification detection does not work"
        );
        // We don't care about the check error here, only the actual count
        if quote_check.update(line).is_err() && first_error.is_none() {
            first_error = Some(i);
        }
        if !quote_check.end_is_quoted {
            line_after_last_unquoted = Some(i + 1);
        }

        writeln!(
            temp_file,
            "{:>4} {}",
            quote_check.quote_level,
            quick_display_indented_line(line, false)
        )?;
    }
    temp_file.flush()?;

    let open_at_line = first_error.or(line_after_last_unquoted).unwrap_or(0);
    std::process::Command::new(editor)
        .arg(temp_file.path())
        .arg(format!("+{}", open_at_line + 1))
        .status()?;

    let mut contents = String::new();
    temp_file.seek(SeekFrom::Start(0))?;
    temp_file.read_to_string(&mut contents)?;
    let old_lines = act
        .body
        .iter()
        .map(|l| l.content().trim().to_owned())
        .collect();
    let new_lines = contents.lines().map(|l| l[5..].trim().to_owned()).collect();
    let mut fixups = Fixups::load(act.identifier)?;
    update_fixups(&mut fixups, old_lines, new_lines)?;
    fixups.save()?;
    Ok(())
}

fn update_fixups(
    fixups: &mut Fixups,
    mut original_lines: Vec<String>,
    mut new_lines: Vec<String>,
) -> Result<()> {
    ensure!(
        original_lines.len() == new_lines.len(),
        "Line insertions and deletions are not supported. Please leave an empty line if you want to delete it"
    );
    while let Some(change_pos) = original_lines
        .iter()
        .zip(&new_lines)
        .position(|(ol, nl)| ol != nl)
    {
        let context_len = (0..=change_pos.min(10))
            .find(|context_len| {
                let needle = &original_lines[change_pos - context_len..=change_pos];
                let count = original_lines
                    .windows(needle.len())
                    .filter(|w| *w == needle)
                    .count();
                count == 1
            })
            .ok_or_else(|| {
                let l = &original_lines[change_pos];
                anyhow!("Could not find big enough context for '{l}' on line {change_pos:?}",)
            })?;
        let after = original_lines[change_pos - context_len..change_pos]
            .iter()
            .map(|l| l.to_owned())
            .collect();
        let (set_bold, new) = if new_lines[change_pos].contains("<BOLD>") {
            (
                true,
                new_lines[change_pos]
                    .replace("<BOLD>", "")
                    .trim()
                    .to_owned(),
            )
        } else {
            (false, new_lines[change_pos].to_owned())
        };
        info!("Added fixup, old: {:?}", original_lines[change_pos]);
        info!("             new: {new:?}");
        info!("     context len: {context_len:?}");
        info!("            bold: {set_bold:?}");
        fixups.add(Fixup {
            after,
            old: original_lines[change_pos].to_owned(),
            new: new.clone(),
            set_bold,
        });
        new_lines[change_pos] = new.clone();
        original_lines[change_pos] = new;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use hun_law::identifier::ActIdentifier;
    use pretty_assertions::assert_eq;

    use super::*;

    fn run_update_fixups<'a>(
        original_lines: &'a [&'a str],
        new_lines: &'a [&'a str],
    ) -> Result<Vec<Fixup>> {
        let temp_dir = tempfile::Builder::new()
            .prefix("hun_law_test_run")
            .tempdir()
            .unwrap();
        let mut fixups = Fixups::load_from(
            ActIdentifier {
                year: 2042,
                number: 420,
            },
            temp_dir.path().to_owned(),
        )?;

        update_fixups(
            &mut fixups,
            original_lines.iter().map(|l| l.to_string()).collect(),
            new_lines.iter().map(|l| l.to_string()).collect(),
        )?;
        Ok(fixups.into())
    }

    #[test]
    fn test_simple_replacement() {
        let fixups = run_update_fixups(
            &["line 1", "line 2", "line 3"],
            &["line 1", "modified", "line 3"],
        )
        .unwrap();
        assert_eq!(
            fixups,
            vec![Fixup {
                after: vec![],
                old: "line 2".into(),
                new: "modified".into(),
                set_bold: false,
            }]
        )
    }

    #[test]
    fn test_replace_with_context() {
        let fixups = run_update_fixups(
            &["line 1", "r1", "line 3", "r1"],
            &["line 1", "modified", "line 3", "r1"],
        )
        .unwrap();
        assert_eq!(
            fixups,
            vec![Fixup {
                after: vec!["line 1".into()],
                old: "r1".into(),
                new: "modified".into(),
                set_bold: false,
            }]
        );
        let fixups = run_update_fixups(
            &["line 1", "r1", "line 3", "r1"],
            &["line 1", "r1", "line 3", "modified"],
        )
        .unwrap();
        assert_eq!(
            fixups,
            vec![Fixup {
                after: vec!["line 3".into()],
                old: "r1".into(),
                new: "modified".into(),
                set_bold: false,
            }]
        )
    }

    #[test]
    fn test_replace_with_empty_context() {
        let fixups = run_update_fixups(
            &["line 1", "r1", "", "r1"],
            &["line 1", "modified", "", "r1"],
        )
        .unwrap();
        assert_eq!(
            fixups,
            vec![Fixup {
                after: vec!["line 1".into()],
                old: "r1".into(),
                new: "modified".into(),
                set_bold: false,
            }]
        );
        let fixups = run_update_fixups(
            &["line 1", "r1", "", "r1"],
            &["line 1", "r1", "", "modified"],
        )
        .unwrap();
        assert_eq!(
            fixups,
            vec![Fixup {
                after: vec!["".into()],
                old: "r1".into(),
                new: "modified".into(),
                set_bold: false,
            }]
        )
    }

    #[test]
    fn test_replace_multi() {
        let fixups = run_update_fixups(
            &["line 1", "line 2", "line 3"],
            &["line 1", "modified", "modified 3"],
        )
        .unwrap();
        assert_eq!(
            fixups,
            vec![
                Fixup {
                    after: vec![],
                    old: "line 2".into(),
                    new: "modified".into(),
                    set_bold: false,
                },
                Fixup {
                    after: vec![],
                    old: "line 3".into(),
                    new: "modified 3".into(),
                    set_bold: false,
                }
            ]
        )
    }

    #[test]
    fn test_replace_multi_with_context() {
        let fixups = run_update_fixups(
            &["line 1", "rme", "rme", "modified"],
            &["line 1", "modified", "modified", "m2"],
        )
        .unwrap();
        assert_eq!(
            fixups,
            vec![
                Fixup {
                    after: vec!["line 1".into()],
                    old: "rme".into(),
                    new: "modified".into(),
                    set_bold: false,
                },
                Fixup {
                    after: vec![],
                    old: "rme".into(),
                    new: "modified".into(),
                    set_bold: false,
                },
                Fixup {
                    after: vec!["modified".into(), "modified".into()],
                    old: "modified".into(),
                    new: "m2".into(),
                    set_bold: false,
                }
            ]
        )
    }

    #[test]
    fn test_simple_delete() {
        let fixups =
            run_update_fixups(&["line 1", "line 2", "line 3"], &["line 1", "", "line 3"]).unwrap();
        assert_eq!(
            fixups,
            vec![Fixup {
                after: vec![],
                new: String::new(),
                old: "line 2".into(),
                set_bold: false,
            }]
        )
    }

    #[test]
    fn test_delete_with_context() {
        let fixups = run_update_fixups(
            &["line 1", "r1", "line 3", "r1"],
            &["line 1", "", "line 3", "r1"],
        )
        .unwrap();
        assert_eq!(
            fixups,
            vec![Fixup {
                after: vec!["line 1".into()],
                new: String::new(),
                old: "r1".into(),
                set_bold: false,
            }]
        );
        let fixups = run_update_fixups(
            &["line 1", "r1", "line 3", "r1"],
            &["line 1", "r1", "line 3", ""],
        )
        .unwrap();
        assert_eq!(
            fixups,
            vec![Fixup {
                after: vec!["line 3".into()],
                new: String::new(),
                old: "r1".into(),
                set_bold: false,
            }]
        )
    }

    #[test]
    fn test_delete_with_empty_context() {
        let fixups =
            run_update_fixups(&["line 1", "r1", "", "r1"], &["line 1", "", "", "r1"]).unwrap();
        assert_eq!(
            fixups,
            vec![Fixup {
                after: vec!["line 1".into()],
                new: String::new(),
                old: "r1".into(),
                set_bold: false,
            }]
        );
        let fixups =
            run_update_fixups(&["line 1", "r1", "", "r1"], &["line 1", "r1", "", ""]).unwrap();
        assert_eq!(
            fixups,
            vec![Fixup {
                after: vec!["".into()],
                new: String::new(),
                old: "r1".into(),
                set_bold: false,
            }]
        )
    }

    #[test]
    fn test_delete_multi_with_context() {
        let fixups = run_update_fixups(
            &["line 1", "rme", "rme", "modified", "modified"],
            &["line 1", "", "", "m2", "modified"],
        )
        .unwrap();
        assert_eq!(
            fixups,
            vec![
                Fixup {
                    after: vec!["line 1".into()],
                    new: String::new(),
                    old: "rme".into(),
                    set_bold: false,
                },
                Fixup {
                    after: vec![],
                    new: String::new(),
                    old: "rme".into(),
                    set_bold: false,
                },
                Fixup {
                    after: vec!["".into()],
                    old: "modified".into(),
                    new: "m2".into(),
                    set_bold: false,
                }
            ]
        )
    }

    #[test]
    fn test_simple_bold() {
        let fixups = run_update_fixups(
            &["line 1", "line 2", "line 3"],
            &["line 1", "<BOLD> line 2", "line 3"],
        )
        .unwrap();
        assert_eq!(
            fixups,
            vec![Fixup {
                after: vec![],
                old: "line 2".into(),
                new: "line 2".into(),
                set_bold: true,
            }]
        )
    }
}
