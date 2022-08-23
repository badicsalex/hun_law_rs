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

use hun_law::util::indentedline::IndentedLine;

pub fn quick_display_indented_line(l: &IndentedLine, testing_tags: bool) -> String {
    let mut s = String::new();
    let mut indent = (l.indent() * 0.2) as usize;
    if testing_tags {
        if l.is_bold() {
            s.push_str("<BOLD>");
            indent = indent.saturating_sub(6);
        }
        if !l.is_justified() {
            s.push_str("<NJ>");
            indent = indent.saturating_sub(4);
        }
    }
    s.push_str(&" ".repeat(indent));
    s.push_str(l.content());
    s
}
