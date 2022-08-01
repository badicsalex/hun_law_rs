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

mod datatests;
pub mod test_utils;

#[allow(unused_macros)]
macro_rules! declare_test {
    (dir = $dir:expr, pattern = $pattern:expr) => {
        pub fn test_dir() -> String {
            std::path::Path::new(file!())
                .parent()
                .expect("No parent of calling module")
                .join($dir)
                .to_str()
                .expect("Path was not unicode somehow")
                .to_owned()
        }

        pub const FILE_PATTERN: &str = $pattern;
    };
}

pub(crate) use declare_test;

macro_rules! generate_harness{
    ($($id_first:ident$(::$id_rest:ident)*),* $(,)*) => {
        datatest_stable::harness!(
            $(
                datatests::$id_first$(::$id_rest)*::run_test,
                datatests::$id_first$(::$id_rest)*::test_dir(),
                datatests::$id_first$(::$id_rest)*::FILE_PATTERN,
            )*
        );
    }
}

generate_harness!(
    test_pdf_parser,
    test_structure_parser,
    test_semantic_parser,
    test_add_semantic_info,
);
