// Test script to verify pattern matching behavior
use std::fs::File;
use std::io::Write;
use std::process::Command;

fn main() {
    // Create test file with sample lines
    let test_content = r#"LineConnectDriver_oSTART found here
LineConnectDriver_oLOAD is working
Some other line with oSTART
Another line with oLOAD only
LineConnectDriver_ without suffix
Just oSTART alone
Just oLOAD alone
LineConnectDriver_something_else
"#;

    let mut file = File::create("test_input.txt").unwrap();
    file.write_all(test_content.as_bytes()).unwrap();

    println!("=== Test Input ===");
    println!("{}", test_content);

    // Test case 1: LineConnectDriver_&(oSTART|oLOAD)
    // Expected: lines with LineConnectDriver_ AND (oSTART OR oLOAD)
    println!("=== Test 1: LineConnectDriver_&(oSTART|oLOAD) ===");
    let output1 = Command::new("cargo")
        .args(&["run", "--", "-r", "LineConnectDriver_&(oSTART|oLOAD)", "test_input.txt"])
        .output()
        .expect("Failed to execute cargo run");
    println!("stdout: {}", String::from_utf8_lossy(&output1.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output1.stderr));

    // Test case 2: LineConnectDriver_&oSTART|oLOAD
    // Expected: (LineConnectDriver_ AND oSTART) OR oLOAD
    println!("=== Test 2: LineConnectDriver_&oSTART|oLOAD ===");
    let output2 = Command::new("cargo")
        .args(&["run", "--", "-r", "LineConnectDriver_&oSTART|oLOAD", "test_input.txt"])
        .output()
        .expect("Failed to execute cargo run");
    println!("stdout: {}", String::from_utf8_lossy(&output2.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output2.stderr));

    // Test current simple AND behavior
    println!("=== Test 3: LineConnectDriver_&oSTART (current behavior) ===");
    let output3 = Command::new("cargo")
        .args(&["run", "--", "-r", "LineConnectDriver_&oSTART", "test_input.txt"])
        .output()
        .expect("Failed to execute cargo run");
    println!("stdout: {}", String::from_utf8_lossy(&output3.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output3.stderr));
}