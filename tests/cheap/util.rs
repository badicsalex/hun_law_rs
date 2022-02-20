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

#[fixture]
pub fn cache(tempdir: TempDir) -> Cache {
    Cache::new(&tempdir.path())
}
