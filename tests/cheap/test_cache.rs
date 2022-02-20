use super::util::{cache_in_tempdir, CacheInTempDir};
use anyhow::Result;
use rstest::rstest;
use serde::{Deserialize, Serialize};

#[rstest]
fn test_cache_simple_store_load(cache_in_tempdir: CacheInTempDir) {
    let cache = cache_in_tempdir.cache;
    assert!(cache.store("abcd", b"testdata").is_ok());
    assert_eq!(cache.load("abcd").unwrap(), b"testdata");

    assert!(cache.store("abcd", b"other data").is_ok());
    assert_eq!(cache.load("abcd").unwrap(), b"other data");
}

#[rstest]
fn test_cache_subdir_store_load(cache_in_tempdir: CacheInTempDir) {
    let cache = cache_in_tempdir.cache;
    assert!(cache.store("abc/abcd", b"testdata").is_ok());
    assert!(cache.store("123/abcd", b"123testdata").is_ok());
    assert_eq!(cache.load("abc/abcd").unwrap(), b"testdata");
    assert_eq!(cache.load("123/abcd").unwrap(), b"123testdata");
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct CacheTestStruct {
    x: i32,
    s: String,
    v: Vec<CacheTestEnum>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum CacheTestEnum {
    CacheTestA,
    CacheTestB { val: i64 },
}

#[rstest]
fn test_cache_structured(cache_in_tempdir: CacheInTempDir) {
    let cache = cache_in_tempdir.cache;
    let test_struct = CacheTestStruct {
        x: 1234,
        s: "lel".to_string(),
        v: vec![
            CacheTestEnum::CacheTestA,
            CacheTestEnum::CacheTestB { val: 5 },
        ],
    };
    assert!(cache.store_json_gz("abcd", &"simple_string").is_ok());
    assert_eq!(
        cache.load_json_gz::<String>("abcd").unwrap(),
        "simple_string"
    );

    assert!(cache.store_json_gz("struct", &test_struct).is_ok());
    assert_eq!(
        cache.load_json_gz::<CacheTestStruct>("struct").unwrap(),
        test_struct
    );
}

#[rstest]
fn test_failing_load(cache_in_tempdir: CacheInTempDir) {
    let cache = cache_in_tempdir.cache;
    assert!(cache.load("abcd").is_err());
    assert!(cache.load_json_gz::<CacheTestStruct>("strct").is_err());
}

#[rstest]
fn test_cached_run_simple(cache_in_tempdir: CacheInTempDir) {
    let cache = cache_in_tempdir.cache;
    let mut fn_run_count = 0;

    assert_eq!(
        cache
            .run_cached("key1", || -> Result<i32> {
                fn_run_count += 1;
                Ok(5)
            })
            .unwrap(),
        5,
        "Simple successful run"
    );
    assert_eq!(fn_run_count, 1, "Function ran once");

    assert_eq!(
        cache
            .run_cached("key1", || -> Result<i32> {
                fn_run_count += 1;
                Ok(5)
            })
            .unwrap(),
        5,
        "Second run"
    );
    assert_eq!(fn_run_count, 1, "Function still only ran once");

    assert_eq!(
        cache
            .run_cached("key1", || -> Result<i32> {
                fn_run_count += 1;
                Ok(1337)
            })
            .unwrap(),
        5,
        "If cache exists, cached value is returned"
    );
    assert_eq!(fn_run_count, 1, "Function still only ran once");

    assert_eq!(
        cache
            .run_cached("key2", || -> Result<i32> {
                fn_run_count += 1;
                Ok(1337)
            })
            .unwrap(),
        1337,
        "Different keys store different values"
    );
    assert_eq!(fn_run_count, 2, "Function ran twice for two keys");

    assert_eq!(
        cache
            .run_cached("key2", || -> Result<i32> {
                fn_run_count += 1;
                Ok(420420)
            })
            .unwrap(),
        1337,
        "Caching works for both keys separately"
    );
    assert_eq!(
        fn_run_count, 2,
        "Function still only ran twice for two keys"
    );
}

#[rstest]
fn test_cached_run_errors(cache_in_tempdir: CacheInTempDir) {
    let cache = cache_in_tempdir.cache;
    let mut fn_run_count = 0;

    assert!(
        cache
            .run_cached("key1", || -> Result<i32> {
                fn_run_count += 1;
                anyhow::bail!("oops!")
            })
            .is_err(),
        "Errors are propagated"
    );
    assert_eq!(fn_run_count, 1, "Function ran once");

    assert!(
        cache
            .run_cached("key1", || -> Result<i32> {
                fn_run_count += 1;
                anyhow::bail!("oops!")
            })
            .is_err(),
        "Errors are still propagated"
    );
    assert_eq!(
        fn_run_count, 2,
        "Function is retried if previous run failed"
    );

    assert_eq!(
        cache
            .run_cached("key1", || -> Result<i32> {
                fn_run_count += 1;
                Ok(5)
            })
            .unwrap(),
        5,
        "Successful run after the errors"
    );
    assert_eq!(
        fn_run_count, 3,
        "Function is still retried if previous run failed"
    );

    assert_eq!(
        cache
            .run_cached("key1", || -> Result<i32> {
                fn_run_count += 1;
                anyhow::bail!("oops!")
            })
            .unwrap(),
        5,
        "Function is not even ran after caching, cahced result is returned"
    );
    assert_eq!(
        fn_run_count, 3,
        "Function does not run if cache is available"
    );
}
