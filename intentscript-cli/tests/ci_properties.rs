// Property-based tests for CI mode JSON diagnostics
// Feature: intentscript-compiler, Property 48: CI JSON diagnostic validity
// Validates: Requirements 12.6

use quickcheck::{Arbitrary, Gen, TestResult};
use quickcheck_macros::quickcheck;
use serde_json::Value;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Property 48: CI JSON diagnostic validity
/// For any compilation result in CI mode, the JSON diagnostic output should be valid JSON
/// and contain stable exit codes
#[quickcheck]
fn property_ci_json_diagnostic_validity(source: ValidOrInvalidSource) -> TestResult {
    // Create a temporary directory for test files
    let temp_dir = match TempDir::new() {
        Ok(dir) => dir,
        Err(_) => return TestResult::discard(),
    };

    let input_path = temp_dir.path().join("test.intent");
    
    // Write source to file
    if fs::write(&input_path, &source.0).is_err() {
        return TestResult::discard();
    }

    // Run build command with --json flag
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--package",
            "intentscript-cli",
            "--bin",
            "intentscript",
            "--",
            "build",
            input_path.to_str().unwrap(),
            "--json",
        ])
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return TestResult::discard(),
    };

    // Check that output is valid JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_result: Result<Value, _> = serde_json::from_str(&stdout);

    if json_result.is_err() {
        return TestResult::failed();
    }

    let json = json_result.unwrap();

    // Check that JSON contains required fields
    if !json.is_object() {
        return TestResult::failed();
    }

    let obj = json.as_object().unwrap();

    // Must have "status" field
    if !obj.contains_key("status") {
        return TestResult::failed();
    }

    // Check exit code is stable (0 for success, 1 for error, 2 for warning)
    let exit_code = output.status.code().unwrap_or(-1);
    let is_stable_exit_code = exit_code == 0 || exit_code == 1 || exit_code == 2;

    TestResult::from_bool(is_stable_exit_code)
}

/// Wrapper for valid or invalid IntentScript source
#[derive(Clone, Debug)]
struct ValidOrInvalidSource(String);

impl Arbitrary for ValidOrInvalidSource {
    fn arbitrary(g: &mut Gen) -> Self {
        let choice = u8::arbitrary(g) % 3;

        let source = match choice {
            0 => {
                // Valid source
                let task_name = format!("task_{}", u8::arbitrary(g) % 10);
                format!(
                    "task \"{}\" v1.0 {{\n  goal: \"Test\"\n  run: step1\n}}\n",
                    task_name
                )
            }
            1 => {
                // Invalid source (missing required section)
                format!("task \"invalid\" v1.0 {{\n  goal: \"Test\"\n}}\n")
            }
            _ => {
                // Syntax error
                format!("task \"broken\" {{\n  invalid syntax here\n}}\n")
            }
        };

        ValidOrInvalidSource(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ci_json_valid_source() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("test.intent");

        let source = r#"task "test" v1.0 {
  goal: "Test"
  run: step1
}"#;

        fs::write(&input_path, source).unwrap();

        let output = Command::new("cargo")
            .args(&[
                "run",
                "--package",
                "intentscript-cli",
                "--bin",
                "intentscript",
                "--",
                "build",
                input_path.to_str().unwrap(),
                "--json",
            ])
            .output()
            .unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: Value = serde_json::from_str(&stdout).expect("Output should be valid JSON");

        assert!(json.is_object());
        assert!(json.as_object().unwrap().contains_key("status"));
    }

    #[test]
    fn test_ci_json_invalid_source() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("test.intent");

        let source = r#"task "broken" {
  invalid syntax
}"#;

        fs::write(&input_path, source).unwrap();

        let output = Command::new("cargo")
            .args(&[
                "run",
                "--package",
                "intentscript-cli",
                "--bin",
                "intentscript",
                "--",
                "build",
                input_path.to_str().unwrap(),
                "--json",
            ])
            .output()
            .unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: Value = serde_json::from_str(&stdout).expect("Output should be valid JSON even for errors");

        assert!(json.is_object());
        assert!(json.as_object().unwrap().contains_key("status"));
        
        // Should have error exit code (1)
        assert_eq!(output.status.code().unwrap(), 1);
    }
}
