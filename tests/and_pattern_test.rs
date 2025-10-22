use rgrep::Config;
use std::io::Cursor;

fn create_test_config(pattern: &str) -> Config {
    let mut cfg = Config::default();
    cfg.patterns = vec![pattern.to_string()];
    cfg
}

#[test]
fn test_and_pattern_matching() {
    let test_data = r#"AIS.CometYxlon.CA20.CoW.KernelApp-20250917_001.log
2025-09-17 14:15:42.413 +02:00  DBG     AIS.CometYxlon.CA20.LineConnect.Kernel.LineConnectDriver_       Transmit message to device: oLDRQ:Substrate-CARRIER123456789.22,WAFER@XXXX_XXX_XXX
2025-09-17 14:15:43.452 +02:00  DBG     AIS.CometYxlon.CA20.LineConnect.Kernel.LineConnectDriver_       Received message from device: aLDRQ:
2025-09-17 14:15:43.455 +02:00  DBG     AIS.CometYxlon.CA20.LineConnect.Kernel.LineConnectDriver_       Transmit message to device: oINLK:1
2025-09-17 14:15:43.962 +02:00  DBG     AIS.CometYxlon.CA20.LineConnect.Kernel.LineConnectDriver_       Received message from device: eSTATUS:0101000000000000
2025-09-17 14:15:44.456 +02:00  DBG     AIS.CometYxlon.CA20.LineConnect.Kernel.LineConnectDriver_       Received message from device: aINLK:
2025-09-17 14:15:46.982 +02:00  DBG     AIS.CometYxlon.CA20.LineConnect.Kernel.LineConnectDriver_       Transmit message to device: oINLK:0
2025-09-17 14:15:47.493 +02:00  DBG     AIS.CometYxlon.CA20.LineConnect.Kernel.LineConnectDriver_       Received message from device: eSTATUS:0100000000000000
2025-09-17 14:15:47.988 +02:00  DBG     AIS.CometYxlon.CA20.LineConnect.Kernel.LineConnectDriver_       Received message from device: aINLK:
2025-09-17 14:15:49.028 +02:00  DBG     AIS.CometYxlon.CA20.LineConnect.Kernel.LineConnectDriver_       Transmit message to device: oSTART:XXXX_XXX_XXX@Substrate-CARRIER123456789.22_1,38@Substrate-CARRIER123456789.22_2,37@Substrate-CARRIER123456789.22_3,36@Substrate-CARRIER123456789.22_4,30@Substrate-CARRIER123456789.22_5,31@Substrate-CARRIER123456789.22_6,32@Substrate-CARRIER123456789.22_7,33@Substrate-CARRIER123456789.22_8,34@Substrate-CARRIER123456789.22_9,29@Substrate-CARRIER123456789.22_10,28@Substrate-CARRIER123456789.22_11,27@Substrate-CARRIER123456789.22_12,26@Substrate-CARRIER123456789.22_13,25@Substrate-CARRIER123456789.22_14,20@Substrate-CARRIER123456789.22_15,21@Substrate-CARRIER123456789.22_16,22@Substrate-CARRIER123456789.22_17,23@Substrate-CARRIER123456789.22_18,24@Substrate-CARRIER123456789.22_19,19@Substrate-CARRIER123456789.22_20,18@Substrate-CARRIER123456789.22_21,17@Substrate-CARRIER123456789.22_22,16@Substrate-CARRIER123456789.22_23,15@Substrate-CARRIER123456789.22_24,10@Substrate-CARRIER123456789.22_25,11@Substrate-CARRIER123456789.22_26,12@Substrate-CARRIER123456789.22_27,13@Substrate-CARRIER123456789.22_28,14@Substrate-CARRIER123456789.22_29,8@Substrate-CARRIER123456789.22_30,7@Substrate-CARRIER123456789.22_31,6@Substrate-CARRIER123456789.22_32,2"#;

    // Test AND pattern - should match only lines with both "LineConnectDriver_" AND "oSTART"
    let cfg = create_test_config("LineConnectDriver_&oSTART");
    let reader = Cursor::new(test_data);
    let result = rgrep::run_on_reader(&cfg, reader, None).unwrap();
    
    // Should find exactly 1 match (the line containing both patterns)
    let line_count = result.output.lines().count();
    assert_eq!(line_count, 1, "AND pattern should match exactly 1 line");
    assert!(result.output.contains("oSTART"), "Output should contain oSTART");
    assert!(result.output.contains("LineConnectDriver_"), "Output should contain LineConnectDriver_");

    // Test single pattern - should match multiple lines
    let cfg = create_test_config("LineConnectDriver_");
    let reader = Cursor::new(test_data);
    let result = rgrep::run_on_reader(&cfg, reader, None).unwrap();
    
    // Should find 9 matches (all lines with LineConnectDriver_)
    let line_count = result.output.lines().count();
    assert_eq!(line_count, 9, "Single pattern should match 9 lines");

    // Test OR pattern - should match lines with either pattern
    let cfg = create_test_config("LineConnectDriver_|oSTART");
    let reader = Cursor::new(test_data);
    let result = rgrep::run_on_reader(&cfg, reader, None).unwrap();
    
    // Should find 9 matches (same as LineConnectDriver_ alone since oSTART only appears with LineConnectDriver_)
    let line_count = result.output.lines().count();
    assert_eq!(line_count, 9, "OR pattern should match 9 lines");

    // Test AND pattern with non-existent second term
    let cfg = create_test_config("LineConnectDriver_&NONEXISTENT");
    let reader = Cursor::new(test_data);
    let result = rgrep::run_on_reader(&cfg, reader, None).unwrap();
    
    // Should find 0 matches
    let line_count = result.output.lines().filter(|l| !l.is_empty()).count();
    assert_eq!(line_count, 0, "AND pattern with non-existent term should match 0 lines");
}

#[test]
fn test_and_pattern_case_insensitive() {
    let test_data = "Hello World\nHELLO WORLD\nhello mars\nGoodbye";
    
    let mut cfg = create_test_config("hello&world");
    cfg.case_insensitive = true;
    
    let reader = Cursor::new(test_data);
    let result = rgrep::run_on_reader(&cfg, reader, None).unwrap();
    
    // Should find 2 matches (Hello World and HELLO WORLD - both have hello and world)
    let line_count = result.output.lines().count();
    assert_eq!(line_count, 2, "Case insensitive AND pattern should match 2 lines");
}

#[test]
fn test_and_pattern_with_word_boundaries() {
    let test_data = "test word\ntestword\nword test\nwordtest";
    
    let mut cfg = create_test_config("test&word");
    cfg.word = true;
    
    let reader = Cursor::new(test_data);
    let result = rgrep::run_on_reader(&cfg, reader, None).unwrap();
    
    // Should find 2 matches (only lines where both "test" and "word" appear as whole words)
    let line_count = result.output.lines().count();
    assert_eq!(line_count, 2, "Word boundary AND pattern should match 2 lines");
}