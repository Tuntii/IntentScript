use crate::error::{CliError, Result};
use intentscript_compiler::ExecutionPlan;
use intentscript_runtime::{Executor, Host};
use serde_json;
use std::collections::HashMap;
use std::fs;

/// Execute the run command
/// Executes an IR file with the runtime
pub fn execute(input: &str, host: Option<&str>, json: bool) -> Result<i32> {
    // Read IR file
    let ir_json = fs::read_to_string(input).map_err(|e| {
        CliError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to read IR file '{}': {}", input, e),
        ))
    })?;

    // Deserialize ExecutionPlan
    let execution_plan: ExecutionPlan = serde_json::from_str(&ir_json)?;

    // Create host adapter
    let host_impl = create_host(host)?;

    // Create executor with host reference
    let mut executor = Executor::new(&*host_impl);

    // For now, use empty inputs (in a full implementation, we'd read inputs from CLI or file)
    let inputs = HashMap::new();

    // Execute (executor takes ownership of the plan)
    let result = executor.execute(execution_plan, inputs)?;

    // Display results
    if !json {
        println!("Execution completed successfully");
        println!("\nArtifacts:");
        for artifact in &result.artifacts {
            println!("  - {}: {} bytes", artifact.path, artifact.content.size());
        }

        println!("\nAudit Log:");
        for entry in result.audit_log.entries() {
            println!("  [{}] {}", entry.timestamp, entry.operation);
        }
    } else {
        // Output in JSON format
        let output = serde_json::json!({
            "status": "success",
            "artifacts": result.artifacts.iter().map(|a| {
                serde_json::json!({
                    "path": a.path,
                    "type": a.type_name,
                })
            }).collect::<Vec<_>>(),
            "audit_log_entries": result.audit_log.entries().len(),
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    }

    Ok(0) // Exit code 0 for success
}

/// Create a host adapter based on the specified type
fn create_host(host_type: Option<&str>) -> Result<Box<dyn Host>> {
    match host_type {
        None | Some("mock") => Ok(Box::new(MockHost::new())),
        Some(other) => Err(CliError::InvalidInput(format!(
            "Unknown host type: {}. Available: mock",
            other
        ))),
    }
}

/// Mock host implementation for testing
struct MockHost;

impl MockHost {
    fn new() -> Self {
        Self
    }
}

impl Host for MockHost {
    fn read_file(&self, path: &str) -> std::result::Result<Vec<u8>, intentscript_core::Error> {
        // Mock implementation - read from actual filesystem
        std::fs::read(path).map_err(|e| intentscript_core::Error::host(e.to_string()))
    }

    fn write_file(&self, path: &str, content: &[u8]) -> std::result::Result<(), intentscript_core::Error> {
        // Mock implementation - write to actual filesystem
        std::fs::write(path, content).map_err(|e| intentscript_core::Error::host(e.to_string()))
    }

    fn render_template(
        &self,
        _name: &str,
        _vars: serde_json::Value,
    ) -> std::result::Result<String, intentscript_core::Error> {
        // Mock implementation
        Ok("Mock template output".to_string())
    }

    fn parse_openapi(
        &self,
        _bytes: &[u8],
    ) -> std::result::Result<intentscript_runtime::OpenApiDoc, intentscript_core::Error> {
        // Mock implementation
        Ok(intentscript_runtime::OpenApiDoc {
            content: serde_json::json!({
                "openapi": "3.0.0",
                "info": {
                    "title": "Mock API",
                    "version": "1.0.0"
                }
            }),
        })
    }

    fn parse_markdown(
        &self,
        _bytes: &[u8],
    ) -> std::result::Result<intentscript_runtime::MarkdownDoc, intentscript_core::Error> {
        // Mock implementation
        Ok(intentscript_runtime::MarkdownDoc {
            content: "Mock markdown".to_string(),
        })
    }

    fn export_xlsx(
        &self,
        _spec: &intentscript_runtime::XlsxSpec,
        _rows: &[intentscript_runtime::Row],
    ) -> std::result::Result<Vec<u8>, intentscript_core::Error> {
        // Mock implementation
        Ok(vec![])
    }

    fn export_pdf(
        &self,
        _spec: &intentscript_runtime::PdfSpec,
        _content: &str,
    ) -> std::result::Result<Vec<u8>, intentscript_core::Error> {
        // Mock implementation
        Ok(vec![])
    }

    fn log_operation(&self, _op: intentscript_runtime::Operation) -> std::result::Result<(), intentscript_core::Error> {
        // Mock implementation - just log to stdout
        Ok(())
    }
}

// Helper trait for Value to get size
trait ValueSize {
    fn size(&self) -> usize;
}

impl ValueSize for intentscript_runtime::Value {
    fn size(&self) -> usize {
        use intentscript_runtime::Value;
        match self {
            Value::Bytes(b) => b.len(),
            Value::String(s) => s.len(),
            Value::Int(_) => 8,
            Value::Float(_) => 8,
            Value::Bool(_) => 1,
            Value::Json(j) => j.to_string().len(),
            Value::OpenApiDoc(_) => 0, // Placeholder
            Value::MarkdownDoc(_) => 0, // Placeholder
        }
    }
}
