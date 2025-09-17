use rgrep::boolean_parser::{parse_boolean_expression, build_pattern_regexes, BooleanExpr};
use rgrep::Config;

fn main() {
    println!("=== Testing Boolean Pattern Parser ===");

    // Test 1: LineConnectDriver_&(oSTART|oLOAD)
    println!("\n1. Testing: LineConnectDriver_&(oSTART|oLOAD)");
    match parse_boolean_expression("LineConnectDriver_&(oSTART|oLOAD)") {
        Ok(expr) => {
            println!("✓ Parsed successfully");
            let cfg = Config::default();
            match build_pattern_regexes(&expr, &cfg) {
                Ok(regexes) => {
                    println!("✓ Regex compilation successful");
                    
                    // Test against sample lines
                    let test_lines = [
                        "LineConnectDriver_oSTART found here",      // Should match
                        "LineConnectDriver_oLOAD is working",      // Should match 
                        "Some other line with oSTART",             // Should NOT match
                        "Another line with oLOAD only",            // Should NOT match
                        "LineConnectDriver_ without suffix",       // Should NOT match
                        "Just oSTART alone",                       // Should NOT match
                    ];
                    
                    for line in &test_lines {
                        let matches = expr.matches(line, &regexes);
                        println!("  '{}' -> {}", line, if matches { "MATCH" } else { "NO MATCH" });
                    }
                }
                Err(e) => println!("✗ Regex compilation failed: {}", e),
            }
        }
        Err(e) => println!("✗ Parse failed: {}", e),
    }

    // Test 2: LineConnectDriver_&oSTART|oLOAD  
    println!("\n2. Testing: LineConnectDriver_&oSTART|oLOAD");
    match parse_boolean_expression("LineConnectDriver_&oSTART|oLOAD") {
        Ok(expr) => {
            println!("✓ Parsed successfully");
            let cfg = Config::default();
            match build_pattern_regexes(&expr, &cfg) {
                Ok(regexes) => {
                    println!("✓ Regex compilation successful");
                    
                    // Test against sample lines
                    let test_lines = [
                        "LineConnectDriver_oSTART found here",      // Should match (first part)
                        "LineConnectDriver_oLOAD is working",      // Should match (second part)
                        "Some other line with oSTART",             // Should NOT match
                        "Another line with oLOAD only",            // Should match (second part)
                        "LineConnectDriver_ without suffix",       // Should NOT match
                        "Just oSTART alone",                       // Should NOT match
                    ];
                    
                    for line in &test_lines {
                        let matches = expr.matches(line, &regexes);
                        println!("  '{}' -> {}", line, if matches { "MATCH" } else { "NO MATCH" });
                    }
                }
                Err(e) => println!("✗ Regex compilation failed: {}", e),
            }
        }
        Err(e) => println!("✗ Parse failed: {}", e),
    }

    println!("\n=== Testing Complete ===");
}