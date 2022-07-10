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

use datatests::*;

#[allow(unused_macros)]
macro_rules! declare_test {
    (dir = $dir:expr, pattern = $pattern:expr) => {
        pub fn test_dir() -> &'static str {
            std::path::Path::new(file!())
                .parent()
                .expect("No parent of calling module")
                .to_str()
                .expect("Path was not unicode somehow")
        }

        pub const FILE_PATTERN: &str = $pattern;
    };
}

pub(crate) use declare_test;

macro_rules! generate_harness{
    ($($test:ident),*) => {
        datatest_stable::harness!(
            $(
                $test::run_test,
                $test::test_dir(),
                $test::FILE_PATTERN,
            )*
        );
    }
}

generate_harness!(
    test_pdf_parser,
    test_structure_parser,
    test_reference_parsing
);
