use crate::error::{CliError, Result};
use crate::formatter::Formatter;
use intentscript_parser::Parser;
use serde_json;
use std::fs;

/// Execute the fmt command
/// Formats IntentScript source code
pub fn execute(input: &str, check: bool, json: bool) -> Result<i32> {
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
                eprintln!("Format failed: cannot format file with parse errors");
                for error in &errors {
                    eprintln!("  {}", error);
                }
            }
            return Ok(1); // Exit code 1 for errors
        }
    };

    // Format the AST
    let mut formatter = Formatter::new();
    let formatted = formatter.format_file(&file);

    if check {
        // Check mode: verify formatting without writing
        if source.trim() == formatted.trim() {
            if !json {
                println!("File {} is already formatted", input);
            } else {
                let output = serde_json::json!({
                    "status": "success",
                    "input": input,
                    "formatted": true,
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            }
            Ok(0)
        } else {
            if !json {
                eprintln!("File {} needs formatting", input);
            } else {
                let output = serde_json::json!({
                    "status": "needs_formatting",
                    "input": input,
                    "formatted": false,
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            }
            Ok(2) // Exit code 2 for warnings (needs formatting)
        }
    } else {
        // Write formatted source back to file
        fs::write(input, &formatted).map_err(|e| {
            CliError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to write formatted file '{}': {}", input, e),
            ))
        })?;

        if !json {
            println!("Formatted {}", input);
        } else {
            let output = serde_json::json!({
                "status": "success",
                "input": input,
                "formatted": true,
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
