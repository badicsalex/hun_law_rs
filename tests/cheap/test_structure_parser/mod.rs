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

use crate::test_utils::test_data_from_file;
use hun_law::{
    parser::mk_act_section::ActRawText,
    parser::structure::parse_act_structure,
    structure::ActIdentifier,
    util::{date::Date, indentedline::IndentedLine},
};
use pretty_assertions::assert_eq;
use rstest::rstest;

fn to_indented_lines(data: &[u8]) -> Vec<IndentedLine> {
    std::str::from_utf8(data)
        .unwrap()
        .split('\n')
        .map(IndentedLine::from_test_str)
        .collect()
}

#[rstest]
#[case("structural_elements_1")]
#[case("structural_elements_2")]
fn test_structure_parser(#[case] name: &str) {
    let data = test_data_from_file!(format!("data/{}.txt", name));
    let data_as_lines = to_indented_lines(&data);
    let act = parse_act_structure(ActRawText {
        identifier: ActIdentifier {
            year: 2345,
            number: 0xd,
        },
        subject: "A tesztelésről".to_string(),
        publication_date: Date {
            year: 2345,
            month: 6,
            day: 7,
        },
        body: data_as_lines,
    })
    .unwrap();
    println!("{}", serde_yaml::to_string(&act).unwrap());

    let expected_data = test_data_from_file!(format!("data/{}.yml", name));
    let expected_act = serde_yaml::from_slice(&expected_data).unwrap();
    assert_eq!(act, expected_act);
}
