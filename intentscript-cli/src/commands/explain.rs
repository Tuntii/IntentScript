use crate::error::{CliError, Result};
use intentscript_compiler::ExecutionPlan;
use serde_json;
use std::fs;

/// Execute the explain command
/// Displays a human-readable explanation of an execution plan
pub fn execute(input: &str, _log: Option<&str>) -> Result<i32> {
    // Read IR file
    let ir_json = fs::read_to_string(input).map_err(|e| {
        CliError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to read IR file '{}': {}", input, e),
        ))
    })?;

    // Deserialize ExecutionPlan
    let execution_plan: ExecutionPlan = serde_json::from_str(&ir_json)?;

    // Display explanation
    println!("=== Execution Plan Explanation ===\n");

    // Metadata
    println!("Task: {} (version {})", execution_plan.meta.task_name, execution_plan.meta.task_version);
    println!("Compiler Version: {}", execution_plan.meta.compiler_version);
    println!("Policy Hash: {}", execution_plan.meta.policy_hash);
    println!("IR Schema Version: {}\n", execution_plan.schema_version);

    // Inputs
    println!("Inputs:");
    if execution_plan.inputs.is_empty() {
        println!("  (none)");
    } else {
        for input in &execution_plan.inputs {
            let required = if input.required { "required" } else { "optional" };
            println!("  - {}: {} ({})", input.name, input.type_name, required);
            if let Some(default) = &input.default {
                println!("    default: {}", default);
            }
        }
    }
    println!();

    // Capabilities
    println!("Capabilities:");
    if let Some(fs) = &execution_plan.capabilities.fs {
        println!("  - Filesystem:");
        if !fs.read_roots.is_empty() {
            println!("    Read roots: {}", fs.read_roots.join(", "));
        }
        if !fs.write_roots.is_empty() {
            println!("    Write roots: {}", fs.write_roots.join(", "));
        }
    }
    if execution_plan.capabilities.net {
        println!("  - Network: enabled");
    }
    if execution_plan.capabilities.exec {
        println!("  - Exec: enabled");
    }
    if execution_plan.capabilities.templates {
        println!("  - Templates: enabled");
    }
    if execution_plan.capabilities.exports {
        println!("  - Exports: enabled");
    }
    println!();

    // Limits
    println!("Limits:");
    println!("  - Max repairs: {}", execution_plan.limits.max_repairs);
    if let Some(timeout) = execution_plan.limits.timeout_ms {
        println!("  - Timeout: {}ms", timeout);
    }
    println!();

    // Steps
    println!("Execution Steps:");
    for (i, step) in execution_plan.steps.iter().enumerate() {
        println!("  {}. {} ({})", i + 1, step.id, format_step_kind(&step.kind));
        
        if !step.args.is_empty() {
            println!("     Arguments:");
            for (key, value) in &step.args {
                println!("       {}: {}", key, value);
            }
        }

        if let Some(produces) = &step.produces {
            println!("     Produces: {}", produces);
        }

        if !step.checks.is_empty() {
            println!("     Checks:");
            for check in &step.checks {
                println!("       - {}", check.name);
            }
        }
    }
    println!();

    // Outputs
    println!("Output Artifacts:");
    if execution_plan.outputs.is_empty() {
        println!("  (none)");
    } else {
        for output in &execution_plan.outputs {
            println!("  - {}: {}", output.path, output.type_name);
        }
    }

    Ok(0) // Exit code 0 for success
}

/// Format a step kind for display
fn format_step_kind(kind: &intentscript_compiler::StepKind) -> String {
    use intentscript_compiler::StepKind;
    match kind {
        StepKind::ReadFile => "Read File".to_string(),
        StepKind::WriteFile => "Write File".to_string(),
        StepKind::ParseOpenApi => "Parse OpenAPI".to_string(),
        StepKind::ParseMarkdown => "Parse Markdown".to_string(),
        StepKind::RenderTemplate => "Render Template".to_string(),
        StepKind::ExportXlsx => "Export XLSX".to_string(),
        StepKind::ExportPdf => "Export PDF".to_string(),
        StepKind::Validate => "Validate".to_string(),
        StepKind::Report => "Report".to_string(),
        StepKind::Custom { name } => format!("Custom: {}", name),
    }
}
