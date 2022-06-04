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

use hun_law::pdf_parser::{parse_pdf, CropBox};
use hun_law::util::{indentedline::IndentedLine, is_default};

use crate::test_utils::test_data_from_file;

use rstest::rstest;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct SimplifiedLine {
    #[serde(default, skip_serializing_if = "is_default")]
    indent: f64,
    #[serde(default, skip_serializing_if = "is_default")]
    is_bold: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    content: String,
}

impl PartialEq for SimplifiedLine {
    fn eq(&self, other: &Self) -> bool {
        (self.indent - other.indent) < 0.01
            && self.is_bold == other.is_bold
            && self.content == other.content
    }
}

impl From<&IndentedLine> for SimplifiedLine {
    fn from(l: &IndentedLine) -> Self {
        SimplifiedLine {
            indent: l.indent(),
            is_bold: l.is_bold(),
            content: l.content().to_string(),
        }
    }
}

#[rstest]
#[case("2010_181_part")]
#[case("2015_124_part")]
#[case("ptk_part")]
#[case("korona_part")]
fn test_parsing_mk(#[case] name: &str) {
    let data = test_data_from_file!(format!("data/{}.pdf", name));
    let crop = CropBox {
        top: 842.0 - 1.25 * 72.0,
        ..Default::default()
    };

    let parsed = parse_pdf(&data, crop).unwrap();
    assert_eq!(parsed.len(), 1);
    let lines: Vec<SimplifiedLine> = parsed[0].lines.iter().map(SimplifiedLine::from).collect();
    let expected_lines: Vec<SimplifiedLine> =
        serde_json::from_slice(&test_data_from_file!(format!("data/{}.json", name))).unwrap();
    print!("{}", serde_json::to_string_pretty(&lines).unwrap());
    for (line, expected_line) in std::iter::zip(lines, expected_lines) {
        assert_eq!(line, expected_line);
    }
}
