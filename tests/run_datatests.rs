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

datatest_stable::harness!(
    datatests::test_pdf_parser::test_pdf_parser,
    "tests/datatests/data_pdf_parser",
    r"\.pdf",
    datatests::test_structure_parser::test_structure_parser,
    "tests/datatests/data_structure_parser",
    r"\.txt",
);
