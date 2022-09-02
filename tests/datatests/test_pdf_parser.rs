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

use hun_law::parser::pdf::{parse_pdf, CropBox};
use hun_law::util::singleton_yaml;
use hun_law::util::{indentedline::IndentedLine, is_default};
use serde::{Deserialize, Serialize};

use crate::declare_test;
use crate::test_utils::{ensure_eq, read_all};

declare_test!(dir = "data_pdf_parser", pattern = r"\.pdf");

#[derive(Serialize, Deserialize, Debug)]
struct SimplifiedLine {
    #[serde(default, skip_serializing_if = "is_default")]
    indent: f64,
    #[serde(default, skip_serializing_if = "is_default")]
    is_bold: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    is_justified: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    content: String,
}

impl PartialEq for SimplifiedLine {
    fn eq(&self, other: &Self) -> bool {
        (self.indent - other.indent) < 0.01
            && self.is_bold == other.is_bold
            && self.content == other.content
            && self.is_justified == other.is_justified
    }
}

impl From<&IndentedLine> for SimplifiedLine {
    fn from(l: &IndentedLine) -> Self {
        SimplifiedLine {
            indent: l.indent(),
            is_bold: l.is_bold(),
            content: l.content().to_string(),
            is_justified: l.is_justified(),
        }
    }
}

pub fn run_test(path: &Path) -> datatest_stable::Result<()> {
    let crop = CropBox {
        top: 842.0 - 1.25 * 72.0,
        ..Default::default()
    };

    let parsed = parse_pdf(&read_all(path)?, crop)?;
    ensure_eq(&parsed.len(), &1, "Wrong number of pages parsed")?;
    let lines: Vec<SimplifiedLine> = parsed[0].lines.iter().map(SimplifiedLine::from).collect();
    let expected_lines: Vec<SimplifiedLine> =
        singleton_yaml::from_slice(&read_all(path.with_extension("yml"))?)?;
    ensure_eq(&lines, &expected_lines, "Wrong content")?;
    Ok(())
}
