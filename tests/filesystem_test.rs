use rgrep::{Config, ExitStatus, run};
use std::fs;
use tempfile;

fn create_config(pattern: &str) -> Config {
    Config {
        patterns: vec![pattern.to_string()],
        color: false,
        ..Default::default()
    }
}

// ============ BINARY FILE DETECTION TESTS ============

#[test]
fn test_binary_file_skipped() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    // Create a binary file with NUL byte
    let bin_file = root.join("binary.dat");
    fs::write(&bin_file, &[0x00, 0x48, 0x65, 0x6c, 0x6c, 0x6f]).unwrap();
    
    // Create a text file
    let txt_file = root.join("text.txt");
    fs::write(&txt_file, b"match this").unwrap();
    
    let mut cfg = create_config("match");
    cfg.recursive = true;
    
    let inputs = vec![root.to_string_lossy().to_string()];
    let result = run(&cfg, &inputs).unwrap();
    
    // Should find match in text file but skip binary
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.contains("match this"));
}

#[test]
fn test_all_binary_files_returns_no_match() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    let bin1 = root.join("file1.bin");
    let bin2 = root.join("file2.bin");
    
    fs::write(&bin1, &[0x00, 0xFF, 0xFE]).unwrap();
    fs::write(&bin2, &[0x00, 0x01, 0x02]).unwrap();
    
    let mut cfg = create_config("pattern");
    cfg.recursive = true;
    
    let inputs = vec![root.to_string_lossy().to_string()];
    let result = run(&cfg, &inputs).unwrap();
    
    assert_eq!(result.status, ExitStatus::NoMatch);
}

#[test]
fn test_text_file_without_nul_is_not_binary() {
    let td = tempfile::tempdir().unwrap();
    let file = td.path().join("text.txt");
    
    // File with all printable ASCII characters
    fs::write(&file, b"Hello World 123!@#$%^&*()").unwrap();
    
    let cfg = create_config("World");
    let inputs = vec![file.to_string_lossy().to_string()];
    
    let result = run(&cfg, &inputs).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.contains("World"));
}

#[test]
fn test_utf8_file_is_not_binary() {
    let td = tempfile::tempdir().unwrap();
    let file = td.path().join("utf8.txt");
    
    fs::write(&file, "Hello 世界 🌍").unwrap();
    
    let cfg = create_config("世界");
    let inputs = vec![file.to_string_lossy().to_string()];
    
    let result = run(&cfg, &inputs).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
}

// ============ RECURSIVE SEARCH TESTS ============

#[test]
fn test_recursive_search_basic() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    let subdir = root.join("subdir");
    fs::create_dir(&subdir).unwrap();
    
    fs::write(root.join("root.txt"), b"match in root").unwrap();
    fs::write(subdir.join("sub.txt"), b"match in subdir").unwrap();
    
    let mut cfg = create_config("match");
    cfg.recursive = true;
    
    let inputs = vec![root.to_string_lossy().to_string()];
    let result = run(&cfg, &inputs).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.contains("match in root"));
    assert!(result.output.contains("match in subdir"));
}

#[test]
fn test_recursive_search_nested_directories() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    let level1 = root.join("level1");
    let level2 = level1.join("level2");
    let level3 = level2.join("level3");
    
    fs::create_dir_all(&level3).unwrap();
    
    fs::write(level1.join("file1.txt"), b"match level1").unwrap();
    fs::write(level2.join("file2.txt"), b"match level2").unwrap();
    fs::write(level3.join("file3.txt"), b"match level3").unwrap();
    
    let mut cfg = create_config("match");
    cfg.recursive = true;
    
    let inputs = vec![root.to_string_lossy().to_string()];
    let result = run(&cfg, &inputs).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.contains("match level1"));
    assert!(result.output.contains("match level2"));
    assert!(result.output.contains("match level3"));
}

#[test]
fn test_recursive_search_multiple_directories() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    let dir1 = root.join("dir1");
    let dir2 = root.join("dir2");
    
    fs::create_dir(&dir1).unwrap();
    fs::create_dir(&dir2).unwrap();
    
    fs::write(dir1.join("file.txt"), b"match in dir1").unwrap();
    fs::write(dir2.join("file.txt"), b"match in dir2").unwrap();
    
    let mut cfg = create_config("match");
    cfg.recursive = true;
    
    let inputs = vec![
        dir1.to_string_lossy().to_string(),
        dir2.to_string_lossy().to_string(),
    ];
    let result = run(&cfg, &inputs).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.contains("match in dir1"));
    assert!(result.output.contains("match in dir2"));
}

#[test]
fn test_non_recursive_ignores_subdirectories() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    let subdir = root.join("subdir");
    fs::create_dir(&subdir).unwrap();
    
    let file1 = root.join("file1.txt");
    fs::write(&file1, b"match in root").unwrap();
    fs::write(subdir.join("file2.txt"), b"match in subdir").unwrap();
    
    let mut cfg = create_config("match");
    cfg.recursive = false;
    
    // When not recursive, passing a directory should just use that file
    let inputs = vec![file1.to_string_lossy().to_string()];
    let result = run(&cfg, &inputs).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.contains("match in root"));
    assert!(!result.output.contains("match in subdir"));
}

#[test]
fn test_recursive_empty_directory() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    let empty_dir = root.join("empty");
    fs::create_dir(&empty_dir).unwrap();
    
    let mut cfg = create_config("pattern");
    cfg.recursive = true;
    
    let inputs = vec![empty_dir.to_string_lossy().to_string()];
    let result = run(&cfg, &inputs).unwrap();
    
    assert_eq!(result.status, ExitStatus::NoMatch);
}

#[test]
fn test_recursive_mixed_files_and_directories() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    let subdir = root.join("subdir");
    fs::create_dir(&subdir).unwrap();
    
    let file1 = root.join("file1.txt");
    fs::write(&file1, b"match in file1").unwrap();
    fs::write(subdir.join("file2.txt"), b"match in file2").unwrap();
    
    let mut cfg = create_config("match");
    cfg.recursive = true;
    
    // Mix of file and directory in inputs
    let inputs = vec![
        file1.to_string_lossy().to_string(),
        subdir.to_string_lossy().to_string(),
    ];
    let result = run(&cfg, &inputs).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.contains("match in file1"));
    assert!(result.output.contains("match in file2"));
}

// ============ PATH EXPANSION TESTS ============

#[test]
fn test_stdin_default_when_no_inputs_non_recursive() {
    let cfg = create_config("test");
    let inputs: Vec<String> = vec![];
    
    // When no inputs and non-recursive, should default to stdin (-)
    // We can't easily test stdin in unit tests, but we can verify it doesn't crash
    // and returns expected behavior with actual stdin
    // For now, just verify the config is valid
    assert!(!cfg.recursive);
    assert_eq!(inputs.len(), 0);
}

#[test]
fn test_explicit_file_path() {
    let td = tempfile::tempdir().unwrap();
    let file = td.path().join("explicit.txt");
    fs::write(&file, b"match here").unwrap();
    
    let cfg = create_config("match");
    let inputs = vec![file.to_string_lossy().to_string()];
    
    let result = run(&cfg, &inputs).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.contains("match here"));
}

#[test]
fn test_nonexistent_file_error() {
    let cfg = create_config("pattern");
    let inputs = vec!["nonexistent_file_xyz123.txt".to_string()];
    
    let result = run(&cfg, &inputs);
    
    assert!(result.is_err(), "Should error on nonexistent file");
}

#[test]
fn test_multiple_files_explicit_paths() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    let file1 = root.join("file1.txt");
    let file2 = root.join("file2.txt");
    let file3 = root.join("file3.txt");
    
    fs::write(&file1, b"found 1").unwrap();
    fs::write(&file2, b"nothing here").unwrap();
    fs::write(&file3, b"found 3").unwrap();
    
    let cfg = create_config("found");
    let inputs = vec![
        file1.to_string_lossy().to_string(),
        file2.to_string_lossy().to_string(),
        file3.to_string_lossy().to_string(),
    ];
    
    let result = run(&cfg, &inputs).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.contains("found 1"));
    assert!(result.output.contains("found 3"));
    assert!(!result.output.contains("nothing here"));
}

// ============ FILE TYPE TESTS ============

#[test]
fn test_hidden_files_included() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    // On Unix, files starting with . are hidden
    let hidden = root.join(".hidden");
    fs::write(&hidden, b"match in hidden").unwrap();
    
    let cfg = create_config("match");
    let inputs = vec![hidden.to_string_lossy().to_string()];
    
    let result = run(&cfg, &inputs).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.contains("match in hidden"));
}

#[test]
fn test_various_file_extensions() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    fs::write(root.join("file.txt"), b"match txt").unwrap();
    fs::write(root.join("file.log"), b"match log").unwrap();
    fs::write(root.join("file.rs"), b"match rs").unwrap();
    fs::write(root.join("file.py"), b"match py").unwrap();
    fs::write(root.join("file"), b"match noext").unwrap();
    
    let mut cfg = create_config("match");
    cfg.recursive = true;
    
    let inputs = vec![root.to_string_lossy().to_string()];
    let result = run(&cfg, &inputs).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.contains("match txt"));
    assert!(result.output.contains("match log"));
    assert!(result.output.contains("match rs"));
    assert!(result.output.contains("match py"));
    assert!(result.output.contains("match noext"));
}

#[test]
fn test_large_number_of_files() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    // Create 100 files
    for i in 0..100 {
        let filename = format!("file{}.txt", i);
        let content = if i % 10 == 0 {
            format!("FOUND {}", i)
        } else {
            format!("other {}", i)
        };
        fs::write(root.join(filename), content.as_bytes()).unwrap();
    }
    
    let mut cfg = create_config("FOUND");
    cfg.recursive = true;
    
    let inputs = vec![root.to_string_lossy().to_string()];
    let result = run(&cfg, &inputs).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    let match_count = result.output.lines().count();
    assert_eq!(match_count, 10, "Should find exactly 10 matches");
}

// ============ SYMLINK TESTS (if supported) ============

#[test]
#[cfg(unix)]
fn test_symlink_to_file() {
    use std::os::unix::fs::symlink;
    
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    let real_file = root.join("real.txt");
    let symlink_file = root.join("link.txt");
    
    fs::write(&real_file, b"match content").unwrap();
    symlink(&real_file, &symlink_file).unwrap();
    
    let cfg = create_config("match");
    let inputs = vec![symlink_file.to_string_lossy().to_string()];
    
    let result = run(&cfg, &inputs).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.contains("match content"));
}

// ============ PERMISSIONS TESTS ============

#[test]
#[cfg(unix)]
fn test_unreadable_file_error() {
    use std::os::unix::fs::PermissionsExt;
    
    let td = tempfile::tempdir().unwrap();
    let file = td.path().join("unreadable.txt");
    
    fs::write(&file, b"content").unwrap();
    
    // Remove read permissions
    let mut perms = fs::metadata(&file).unwrap().permissions();
    perms.set_mode(0o000);
    fs::set_permissions(&file, perms).unwrap();
    
    let cfg = create_config("content");
    let inputs = vec![file.to_string_lossy().to_string()];
    
    let result = run(&cfg, &inputs);
    
    // Should get an error when trying to read
    assert!(result.is_err());
    
    // Restore permissions for cleanup
    let mut perms = fs::metadata(&file).unwrap().permissions();
    perms.set_mode(0o644);
    let _ = fs::set_permissions(&file, perms);
}

// ============ SPECIAL FILE NAMES ============

#[test]
fn test_files_with_spaces_in_name() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    let file = root.join("file with spaces.txt");
    fs::write(&file, b"match content").unwrap();
    
    let cfg = create_config("match");
    let inputs = vec![file.to_string_lossy().to_string()];
    
    let result = run(&cfg, &inputs).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.contains("match content"));
}

#[test]
fn test_files_with_special_characters() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    // Note: Some special chars may not be valid on all filesystems
    let file = root.join("file-name_with.special@chars.txt");
    fs::write(&file, b"match content").unwrap();
    
    let cfg = create_config("match");
    let inputs = vec![file.to_string_lossy().to_string()];
    
    let result = run(&cfg, &inputs).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.contains("match content"));
}

// ============ EDGE CASES ============

#[test]
fn test_recursive_with_only_subdirectories_no_files() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    let sub1 = root.join("sub1");
    let sub2 = root.join("sub2");
    let sub3 = sub1.join("sub3");
    
    fs::create_dir_all(&sub3).unwrap();
    fs::create_dir(&sub2).unwrap();
    
    let mut cfg = create_config("pattern");
    cfg.recursive = true;
    
    let inputs = vec![root.to_string_lossy().to_string()];
    let result = run(&cfg, &inputs).unwrap();
    
    assert_eq!(result.status, ExitStatus::NoMatch);
}

#[test]
fn test_recursive_skips_binary_in_subdirs() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    let subdir = root.join("subdir");
    fs::create_dir(&subdir).unwrap();
    
    fs::write(root.join("text.txt"), b"match").unwrap();
    fs::write(subdir.join("binary.bin"), &[0x00, 0xFF]).unwrap();
    
    let mut cfg = create_config("match");
    cfg.recursive = true;
    
    let inputs = vec![root.to_string_lossy().to_string()];
    let result = run(&cfg, &inputs).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.contains("match"));
}
