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

use anyhow::{anyhow, ensure, Result};

use hun_law::{
    fixups::{Fixup, Fixups},
    parser::mk_act_section::ActRawText,
};
use std::io::{Read, Seek, SeekFrom, Write};

use crate::util::quick_display_indented_line;

pub fn run_fixup_editor(act: &ActRawText, editor: &str) -> Result<()> {
    let mut temp_file = tempfile::Builder::new()
        .prefix(&act.identifier.to_string())
        .suffix(".txt")
        .tempfile()?;
    for line in &act.body {
        ensure!(
            !line.content().ends_with(' '),
            "All lines must be rtrimmed, or else modification detection does not work"
        );
        ensure!(
            !line.content().starts_with(' '),
            "All lines must be rtrimmed, or else modification detection does not work"
        );
        writeln!(temp_file, "L: {}", quick_display_indented_line(line, false))?;
    }
    temp_file.flush()?;
    std::process::Command::new(editor)
        .arg(temp_file.path())
        .status()?;
    let mut contents = String::new();
    temp_file.seek(SeekFrom::Start(0))?;
    temp_file.read_to_string(&mut contents)?;
    let old_lines: Vec<&str> = act.body.iter().map(|l| l.content().trim()).collect();
    let new_lines: Vec<&str> = contents.lines().map(|l| l[3..].trim()).collect();
    let mut fixups = Fixups::load(act.identifier)?;
    update_fixups(&mut fixups, old_lines, new_lines)?;
    fixups.save()?;
    Ok(())
}

fn update_fixups<'a>(
    fixups: &mut Fixups,
    mut original_lines: Vec<&'a str>,
    new_lines: Vec<&'a str>,
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
                anyhow!(
                    "Could not find big enough context for {}",
                    original_lines[change_pos]
                )
            })?;
        let after = original_lines[change_pos - context_len..change_pos]
            .iter()
            .map(|l| l.to_owned().to_owned())
            .collect();
        fixups.add(Fixup {
            after,
            old: original_lines[change_pos].to_owned(),
            new: new_lines[change_pos].to_owned(),
        });
        original_lines[change_pos] = new_lines[change_pos];
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

        update_fixups(&mut fixups, original_lines.into(), new_lines.into())?;
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
                },
                Fixup {
                    after: vec![],
                    old: "line 3".into(),
                    new: "modified 3".into(),
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
                },
                Fixup {
                    after: vec![],
                    old: "rme".into(),
                    new: "modified".into(),
                },
                Fixup {
                    after: vec!["modified".into(), "modified".into()],
                    old: "modified".into(),
                    new: "m2".into(),
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
                },
                Fixup {
                    after: vec![],
                    new: String::new(),
                    old: "rme".into(),
                },
                Fixup {
                    after: vec!["".into()],
                    old: "modified".into(),
                    new: "m2".into(),
                }
            ]
        )
    }
}
