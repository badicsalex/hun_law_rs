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

#[allow(unused_macros)]
macro_rules! test_data_from_file {
    ($file_path:expr) => {{
        use std::fs::File;
        use std::io::Read;
        use std::path::{Path, PathBuf};
        let mut fname = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        fname.push(
            Path::new(file!())
                .parent()
                .expect("No parent of calling module"),
        );
        fname.push($file_path);
        let mut f =
            File::open(&fname).unwrap_or_else(|_| panic!("Opening {:?} unsuccessful", fname));
        let mut buffer = Vec::<u8>::new();
        f.read_to_end(&mut buffer).expect("Cannot read file");
        buffer
    }};
}
// Hack to export the macro
#[allow(unused_imports)]
pub(crate) use test_data_from_file;
