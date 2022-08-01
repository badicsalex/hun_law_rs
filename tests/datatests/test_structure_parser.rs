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

use std::path::Path;

use hun_law::structure::{Act, ActChild, ParagraphChildren, SAEBody};

use crate::test_utils::{ensure_eq, parse_txt_as_act, read_all};

use crate::declare_test;
declare_test!(dir = "data_structure_parser", pattern = r"\.txt");

pub fn run_test(path: &Path) -> datatest_stable::Result<()> {
    let mut act = parse_txt_as_act(path)?;
    clean_quoted_blocks(&mut act);
    let expected_act: Act = serde_yaml::from_slice(&read_all(path.with_extension("yml"))?)?;
    ensure_eq(&expected_act, &act, "Wrong act contents")?;
    Ok(())
}

// clean out the quoted blocks' contents, because we don't want to
// pollute the test yamls with serialized indented lines
fn clean_quoted_blocks(act: &mut Act) {
    for act_child in &mut act.children {
        if let ActChild::Article(article) = act_child {
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
}
