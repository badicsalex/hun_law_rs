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

use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

use hun_law::cache::Cache;
use rstest::fixture;

pub use tempfile::TempDir;

#[fixture]
pub fn tempdir() -> TempDir {
    tempfile::Builder::new()
        .prefix("hun_law_test_run")
        .tempdir()
        .unwrap()
}

pub struct CacheInTempDir {
    pub cache: Cache,

    // This field is here so that drop is called when the whole fixture goes out of scope
    #[allow(dead_code)]
    tempdir: TempDir,
}

#[fixture]
pub fn cache_in_tempdir(tempdir: TempDir) -> CacheInTempDir {
    CacheInTempDir {
        cache: Cache::new(&tempdir.path()),
        tempdir,
    }
}

pub fn read_all(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let mut result = Vec::new();
    File::open(path)?.read_to_end(&mut result)?;
    Ok(result)
}
