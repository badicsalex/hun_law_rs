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

use anyhow::{anyhow, Result};
use flate2::write::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use fn_error_context::context;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub struct Cache {
    cache_dir: PathBuf,
}

impl Cache {
    pub fn new<T>(path: &T) -> Self
    where
        T: std::convert::AsRef<std::ffi::OsStr> + Into<PathBuf>,
    {
        Cache {
            cache_dir: path.into(),
        }
    }

    #[context("Storing cache object '{}'", key)]
    pub fn store(&self, key: &str, data: &[u8]) -> Result<()> {
        let file_path = self.cache_dir.join(key);
        let file_dir = file_path
            .parent()
            .ok_or(anyhow!("Cache object must have a parent directory"))?;
        fs::create_dir_all(file_dir)?;
        fs::write(file_path, data)?;
        Ok(())
    }

    #[context("Loading cache object '{}'", key)]
    pub fn load(&self, key: &str) -> Result<Vec<u8>> {
        let file_path = self.cache_dir.join(key);
        Ok(fs::read(file_path)?)
    }

    #[context("Storing cache object as .json.gz '{}'", key)]
    pub fn store_json_gz<T: serde::Serialize>(&self, key: &str, data: &T) -> Result<()> {
        let key_with_postfix = format!("{}.json.gz", key);

        let the_json = serde_json::to_vec(data)?;

        let mut gz_encoder = GzEncoder::new(Vec::new(), Compression::default());
        gz_encoder.write_all(&the_json)?;
        let gz_encoded_data = gz_encoder.finish()?;

        self.store(&key_with_postfix, &gz_encoded_data)
    }

    #[context("Storing cache object as .json.gz '{}'", key)]
    pub fn load_json_gz<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<T> {
        let key_with_postfix = format!("{}.json.gz", key);

        let gz_encoded_data = self.load(&key_with_postfix)?;

        let mut gz_decoder = GzDecoder::new(Vec::new());
        gz_decoder.write_all(&gz_encoded_data)?;
        let the_json = gz_decoder.finish()?;

        Ok(serde_json::from_slice(&the_json)?)
    }

    pub fn run_cached<T, F>(&self, key: &str, mut f: F) -> Result<T>
    where
        T: serde::Serialize + serde::de::DeserializeOwned,
        F: FnMut() -> Result<T>,
    {
        if let Ok(loaded_result) = self.load_json_gz::<T>(key) {
            return Ok(loaded_result);
        };
        let result = f()?;
        self.store_json_gz(key, &result)?;
        Ok(result)
    }
}
