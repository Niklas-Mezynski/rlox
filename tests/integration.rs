use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use test_generator::test_resources;

#[test_resources("test-scripts/integration/**/*.lox")]
fn run_lox_test(test_path: &str) {
    let test_file = PathBuf::from(test_path);
    println!("Running test: {}", test_file.display());

    let (expected_output, expected_errors) =
        parse_expectations(&test_file).expect("Failed to parse test expectations");

    let interpreter_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/debug/rlox");

    let output = Command::new(&interpreter_path)
        .arg(&test_file)
        .output()
        .expect("Failed to execute interpreter");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    for expected in &expected_output {
        assert!(
            stdout.contains(expected),
            "Expected output '{}' not found in stdout: {}",
            expected,
            stdout
        );
    }

    for expected in &expected_errors {
        assert!(
            stderr.contains(expected),
            "Expected error '{}' not found in stderr: {}",
            expected,
            stderr
        );
    }
}

fn parse_expectations(test_file: &Path) -> Result<(Vec<String>, Vec<String>), std::io::Error> {
    let content = fs::read_to_string(test_file)?;

    let mut expected_output = Vec::new();
    let mut expected_errors = Vec::new();

    let expect_regex = Regex::new(r"// expect:\s*(.+)").unwrap();
    let error_regex = Regex::new(r"// (error|Error).*:\s*(.+)").unwrap();

    for line in content.lines() {
        if let Some(captures) = expect_regex.captures(line) {
            if let Some(expected) = captures.get(1) {
                expected_output.push(expected.as_str().trim().to_string());
            }
        }

        if let Some(captures) = error_regex.captures(line) {
            if let Some(expected) = captures.get(2) {
                expected_errors.push(expected.as_str().trim().to_string());
            }
        }
    }

    Ok((expected_output, expected_errors))
}
