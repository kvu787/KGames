// The persistent store: round trips, fresh starts, wrong files, and the
// dirty flag that makes one cascade save once.

use std::path::PathBuf;

use platformer::progress::Progress;

fn folder(test: &str) -> PathBuf {
    let folder = std::env::temp_dir().join("PlatformerProgressTests").join(test);
    let _ = std::fs::remove_dir_all(&folder);
    folder
}

#[test]
fn progress_round_trips() {
    let path = folder("RoundTrip").join("Save.json");
    let mut progress = Progress::new(3);
    assert!(progress.record(0, 12.345).is_some());
    assert!(progress.unlock(1));
    progress.save(&path).unwrap();
    let loaded = Progress::load(&path, 3).unwrap();
    assert_eq!(loaded.unlocked, 2);
    assert_eq!(loaded.best, vec![Some(12.345), None, None]);
    assert!(!loaded.dirty);
}

#[test]
fn a_missing_file_is_a_fresh_start() {
    let progress = Progress::load(&folder("Missing").join("Save.json"), 2).unwrap();
    assert_eq!(progress.unlocked, 1);
    assert_eq!(progress.best, vec![None, None]);
}

#[test]
fn a_wrong_file_says_what_is_wrong() {
    let path = folder("Wrong").join("Save.json");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "{ \"Unlocked\": \"many\" }").unwrap();
    let problem = Progress::load(&path, 2).unwrap_err();
    assert!(problem.contains("Unlocked"), "{problem}");
    std::fs::write(&path, "{ \"Best\": [true] }").unwrap();
    let problem = Progress::load(&path, 2).unwrap_err();
    assert!(problem.contains("Best[0]"), "{problem}");
    std::fs::write(&path, "not json").unwrap();
    assert!(Progress::load(&path, 2).is_err());
}

#[test]
fn only_a_better_time_records() {
    let mut progress = Progress::new(1);
    assert_eq!(progress.record(0, 10.0), Some(None));
    assert_eq!(progress.record(0, 12.0), None, "slower doesn't record");
    assert_eq!(progress.record(0, 8.0), Some(Some(10.0)), "faster does, and says what it beat");
    assert_eq!(progress.best[0], Some(8.0));
}

#[test]
fn saving_clears_the_dirty_flag_and_unlock_is_idempotent() {
    let path = folder("Dirty").join("Save.json");
    let mut progress = Progress::new(2);
    assert!(progress.unlock(1));
    assert!(progress.dirty);
    progress.save(&path).unwrap();
    assert!(!progress.dirty);
    assert!(!progress.unlock(1), "already unlocked");
    assert!(!progress.dirty);
    assert!(!progress.unlock(5), "past the levels");
}
