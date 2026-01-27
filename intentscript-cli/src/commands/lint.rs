use crate::error::{CliError, Result};
use intentscript_compiler::{Policy, SemanticAnalyzer};
use intentscript_parser::Parser;
use serde_json;
use std::fs;

/// Execute the lint command
/// Runs static analysis without generating IR or executing
pub fn execute(input: &str, json: bool) -> Result<i32> {
    // Read source file
    let source = fs::read_to_string(input).map_err(|e| {
        CliError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to read input file '{}': {}", input, e),
        ))
    })?;

    // Parse source
    let mut parser = Parser::new(&source);
    let file = match parser.parse_file() {
        Ok(f) => f,
        Err(errors) => {
            if json {
                output_json_diagnostics(&errors, 1)?;
            } else {
                eprintln!("Lint failed with {} error(s):", errors.len());
                for error in &errors {
                    eprintln!("  {}", error);
                }
            }
            return Ok(1); // Exit code 1 for errors
        }
    };

    // Semantic analysis
    let policy = Policy::new();
    let mut analyzer = SemanticAnalyzer::with_policy(policy);
    
    let mut has_errors = false;
    let mut all_errors = Vec::new();

    if let Err(errors) = analyzer.analyze(&file) {
        has_errors = true;
        all_errors.extend(errors);
    }

    // Report results
    if has_errors {
        if json {
            output_json_diagnostics(&all_errors, 1)?;
        } else {
            eprintln!("Lint failed with {} error(s):", all_errors.len());
            for error in &all_errors {
                eprintln!("  {}", error);
            }
        }
        Ok(1) // Exit code 1 for errors
    } else {
        if !json {
            println!("Lint passed: no errors found in {}", input);
        } else {
            let output = serde_json::json!({
                "status": "success",
                "input": input,
                "errors": 0,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        Ok(0) // Exit code 0 for success
    }
}

/// Output diagnostics in JSON format for CI
fn output_json_diagnostics(errors: &[intentscript_core::Error], exit_code: i32) -> Result<()> {
    let diagnostics: Vec<_> = errors
        .iter()
        .map(|e| {
            serde_json::json!({
                "message": e.to_string(),
                "severity": "error",
            })
        })
        .collect();

    let output = serde_json::json!({
        "status": "error",
        "exit_code": exit_code,
        "diagnostics": diagnostics,
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
