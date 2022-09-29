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

pub fn fix_character_coding_quirks(c: char) -> char {
    match c {
        'Õ' => 'Ő', // Note the ~ on top of the first ő
        'õ' => 'ő', // Note the ~ on top of the first ő
        'Û' => 'Ű', // Note the ^ on top of the first ű
        'û' => 'ű', // Note the ^ on top of the first ű
        _ => c,
    }
}

/// Compare floats but only for somewhat correct sorting.
///
/// Does not care about equal values or NaNs
pub fn compare_float_for_sorting(f1: f32, f2: f32) -> std::cmp::Ordering {
    if f1 < f2 {
        std::cmp::Ordering::Less
    } else {
        std::cmp::Ordering::Greater
    }
}
