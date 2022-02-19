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

mod cache;
use std::path::PathBuf;
use anyhow::Result;

fn main() -> Result<()>{
    let cache = cache::Cache::new(PathBuf::from("./cache"));
    let result = cache.run_cached("lel", || {
        print!("I ran!");
        Ok(5)
    });
    print!("Result is {:?}", result);
    Ok(())
}
