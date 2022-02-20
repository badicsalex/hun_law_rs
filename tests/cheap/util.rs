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
