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

use serde::Serialize;
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

pub fn ensure_eq<T, U>(expected: &T, actual: &U, message: &str) -> datatest_stable::Result<()>
where
    T: Serialize + ?Sized + PartialEq<U>,
    U: Serialize + ?Sized,
{
    // This is duplicated from ensure_eq, but that's because the structures may be 'equal' even
    // when ther YML form is not.
    if expected != actual {
        Err(format!(
            "{}\n{}",
            message,
            colored_diff::PrettyDifference {
                expected: &serde_yaml::to_string(expected).unwrap(),
                actual: &serde_yaml::to_string(actual).unwrap()
            }
        )
        .into())
    } else {
        Ok(())
    }
}
