use sayiir_openflow::*;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

#[test]
fn test_cleanup_stale_cache() {
    let cache_root = dirs::home_dir()
        .unwrap()
        .join(".sayiir")
        .join("cache")
        .join("test_cleanup");

    // Create test cache directories
    fs::create_dir_all(&cache_root).unwrap();

    // Create old cache (8 days old)
    let old_cache = cache_root.join("old_workflow_task_rust");
    fs::create_dir_all(&old_cache).unwrap();
    let old_time = SystemTime::now() - Duration::from_secs(8 * 24 * 3600);
    set_dir_mtime(&old_cache, old_time);

    // Create recent cache (2 days old)
    let recent_cache = cache_root.join("recent_workflow_task_python");
    fs::create_dir_all(&recent_cache).unwrap();

    // Run cleanup
    cleanup_stale_cache(Some(cache_root.clone()), 7).unwrap();

    // Old cache should be removed
    assert!(!old_cache.exists());

    // Recent cache should still exist
    assert!(recent_cache.exists());

    // Cleanup test directory
    fs::remove_dir_all(&cache_root).ok();
}

#[test]
fn test_cleanup_respects_age_threshold() {
    let cache_root = dirs::home_dir()
        .unwrap()
        .join(".sayiir")
        .join("cache")
        .join("test_threshold");

    fs::create_dir_all(&cache_root).unwrap();

    // Create cache that's slightly less than 7 days old (6 days 23 hours)
    let exactly_seven = cache_root.join("seven_days_old");
    fs::create_dir_all(&exactly_seven).unwrap();
    let seven_days_ago = SystemTime::now() - Duration::from_secs(7 * 24 * 3600 - 3600);
    set_dir_mtime(&exactly_seven, seven_days_ago);

    // Run cleanup with 7-day threshold
    cleanup_stale_cache(Some(cache_root.clone()), 7).unwrap();

    // Should still exist (not strictly greater than 7 days)
    assert!(exactly_seven.exists());

    // Cleanup
    fs::remove_dir_all(&cache_root).ok();
}

// Helper to set directory modification time
fn set_dir_mtime(path: &PathBuf, time: SystemTime) {
    filetime::set_file_mtime(path, filetime::FileTime::from_system_time(time)).unwrap();
}
