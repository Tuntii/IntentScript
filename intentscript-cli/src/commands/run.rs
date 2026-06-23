use crate::error::{CliError, Result};
use intentscript_compiler::ExecutionPlan;
use intentscript_runtime::{Executor, Host, RealHost, Value};
use serde_json;
use std::collections::HashMap;
use std::fs;

/// Execute the run command
pub fn execute(
    input: &str,
    host: Option<&str>,
    inputs: &[(String, String)],
    json: bool,
) -> Result<i32> {
    let ir_json = fs::read_to_string(input).map_err(|e| {
        CliError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to read IR file '{}': {}", input, e),
        ))
    })?;

    let execution_plan: ExecutionPlan = serde_json::from_str(&ir_json)?;

    let host_impl = create_host(host)?;
    let mut executor = Executor::new(&*host_impl);

    let mut input_map = HashMap::new();
    for (key, value) in inputs {
        input_map.insert(key.clone(), serde_json::json!(value));
    }

    let result = executor.execute(execution_plan, input_map)?;

    if !json {
        let status = if result.success {
            "Execution completed successfully"
        } else {
            "Execution completed with validation failures"
        };
        println!("{}", status);
        println!("\nArtifacts:");
        for artifact in &result.artifacts {
            let preview = match &artifact.content {
                Value::String(s) => {
                    let truncated: String = s.chars().take(120).collect();
                    if s.len() > 120 {
                        format!("{}...", truncated)
                    } else {
                        truncated
                    }
                }
                other => format!("{:?}", other),
            };
            println!(
                "  - {} ({}): {} chars — {}",
                artifact.path,
                artifact.type_name,
                artifact.content.content_size(),
                preview
            );
        }

        println!("\nAudit Log ({} entries):", result.audit_log.len());
        for entry in result.audit_log.entries() {
            println!("  [{}] {}", entry.timestamp, entry.operation);
        }
    } else {
        let output = serde_json::json!({
            "status": if result.success { "success" } else { "failed" },
            "success": result.success,
            "artifacts": result.artifacts.iter().map(|a| {
                serde_json::json!({
                    "path": a.path,
                    "type": a.type_name,
                    "size": a.content.content_size(),
                })
            }).collect::<Vec<_>>(),
            "audit_log_entries": result.audit_log.entries().len(),
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    }

    Ok(if result.success { 0 } else { 2 })
}

fn create_host(host_type: Option<&str>) -> Result<Box<dyn Host>> {
    match host_type {
        None | Some("real") => Ok(Box::new(RealHost::new())),
        Some("mock") => Ok(Box::new(MockHost::new())),
        Some(other) => Err(CliError::InvalidInput(format!(
            "Unknown host type: {}. Available: real, mock",
            other
        ))),
    }
}

struct MockHost;

impl MockHost {
    fn new() -> Self {
        Self
    }
}

impl Host for MockHost {
    fn read_file(&self, path: &str) -> std::result::Result<Vec<u8>, intentscript_core::Error> {
        std::fs::read(path).map_err(|e| intentscript_core::Error::host(e.to_string()))
    }

    fn write_file(
        &self,
        path: &str,
        content: &[u8],
    ) -> std::result::Result<(), intentscript_core::Error> {
        std::fs::write(path, content).map_err(|e| intentscript_core::Error::host(e.to_string()))
    }

    fn render_template(
        &self,
        _name: &str,
        _vars: serde_json::Value,
    ) -> std::result::Result<String, intentscript_core::Error> {
        Ok("Mock template output".to_string())
    }

    fn parse_openapi(
        &self,
        bytes: &[u8],
    ) -> std::result::Result<intentscript_runtime::OpenApiDoc, intentscript_core::Error> {
        let content: serde_json::Value = serde_json::from_slice(bytes)
            .map_err(|e| intentscript_core::Error::host(e.to_string()))?;
        Ok(intentscript_runtime::OpenApiDoc { content })
    }

    fn parse_markdown(
        &self,
        bytes: &[u8],
    ) -> std::result::Result<intentscript_runtime::MarkdownDoc, intentscript_core::Error> {
        let content = String::from_utf8(bytes.to_vec())
            .map_err(|e| intentscript_core::Error::host(e.to_string()))?;
        Ok(intentscript_runtime::MarkdownDoc { content })
    }

    fn export_xlsx(
        &self,
        _spec: &intentscript_runtime::XlsxSpec,
        _rows: &[intentscript_runtime::Row],
    ) -> std::result::Result<Vec<u8>, intentscript_core::Error> {
        Ok(vec![])
    }

    fn export_pdf(
        &self,
        _spec: &intentscript_runtime::PdfSpec,
        _content: &str,
    ) -> std::result::Result<Vec<u8>, intentscript_core::Error> {
        Ok(vec![])
    }

    fn log_operation(
        &self,
        _op: intentscript_runtime::Operation,
    ) -> std::result::Result<(), intentscript_core::Error> {
        Ok(())
    }
}

trait ValueSize {
    fn content_size(&self) -> usize;
}

impl ValueSize for Value {
    fn content_size(&self) -> usize {
        match self {
            Value::Bytes(b) => b.len(),
            Value::String(s) => s.len(),
            Value::Int(_) => 8,
            Value::Float(_) => 8,
            Value::Bool(_) => 1,
            Value::Json(j) => j.to_string().len(),
            Value::OpenApiDoc(doc) => doc.content.to_string().len(),
            Value::MarkdownDoc(doc) => doc.content.len(),
        }
    }
}