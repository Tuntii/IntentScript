use crate::error::{CliError, Result};
use intentscript_compiler::{Lowering, Policy, SemanticAnalyzer};
use intentscript_parser::Parser;
use serde_json;
use std::fs;
use std::path::Path;

/// Execute the build command
/// Compiles IntentScript source to IR
pub fn execute(input: &str, output: Option<&str>, json: bool) -> Result<i32> {
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
                for error in &errors {
                    eprintln!("{}", error);
                }
            }
            return Ok(1); // Exit code 1 for errors
        }
    };

    // Semantic analysis
    let policy = Policy::new();
    let mut analyzer = SemanticAnalyzer::with_policy(policy.clone());
    if let Err(errors) = analyzer.analyze(&file) {
        if json {
            output_json_diagnostics(&errors, 1)?;
        } else {
            for error in &errors {
                eprintln!("{}", error);
            }
        }
        return Ok(1); // Exit code 1 for errors
    }

    // Lower to IR
    let lowering = Lowering::new(policy);
    
    // For now, lower the first task (in a full implementation, we'd handle multiple tasks)
    if file.tasks.is_empty() {
        return Err(CliError::InvalidInput("No tasks found in source file".to_string()));
    }

    let execution_plan = lowering.lower_task(&file.tasks[0])?;

    // Serialize IR to JSON with canonical formatting
    let ir_json = serde_json::to_string_pretty(&execution_plan)?;

    // Determine output path
    let output_path = output.map(|s| s.to_string()).unwrap_or_else(|| {
        // Default: replace .intent extension with .ir.json
        let input_path = Path::new(input);
        let stem = input_path.file_stem().unwrap_or_default();
        format!("{}.ir.json", stem.to_string_lossy())
    });

    // Write IR to file
    fs::write(&output_path, ir_json).map_err(|e| {
        CliError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to write output file '{}': {}", output_path, e),
        ))
    })?;

    if !json {
        println!("Successfully compiled {} to {}", input, output_path);
    } else {
        // Output success in JSON format
        let success = serde_json::json!({
            "status": "success",
            "input": input,
            "output": output_path,
        });
        println!("{}", serde_json::to_string_pretty(&success)?);
    }

    Ok(0) // Exit code 0 for success
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
