use crate::host::{Host, MarkdownDoc, OpenApiDoc, Operation, OperationKind, PdfSpec, Row, XlsxSpec};
use intentscript_core::{Error, Result};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Production Host that performs real filesystem and parsing operations.
pub struct RealHost {
    templates: HashMap<String, String>,
    operations: Arc<Mutex<Vec<Operation>>>,
}

impl RealHost {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        templates.insert(
            "report".to_string(),
            "# {{title}}\n\n{{body}}\n".to_string(),
        );
        Self {
            templates,
            operations: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn record(&self, kind: OperationKind, details: JsonValue) -> Result<()> {
        self.operations
            .lock()
            .map_err(|e| Error::host(format!("Failed to lock operation log: {}", e)))?
            .push(Operation { kind, details });
        Ok(())
    }

    fn parse_openapi_bytes(bytes: &[u8]) -> Result<JsonValue> {
        if let Ok(value) = serde_json::from_slice::<JsonValue>(bytes) {
            return Ok(value);
        }

        let text = std::str::from_utf8(bytes)
            .map_err(|e| Error::host(format!("OpenAPI must be valid UTF-8: {}", e)))?;

        serde_yaml::from_str(text)
            .map_err(|e| Error::host(format!("Failed to parse OpenAPI as JSON or YAML: {}", e)))
    }
}

impl Default for RealHost {
    fn default() -> Self {
        Self::new()
    }
}

impl Host for RealHost {
    fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        let bytes = fs::read(path).map_err(|e| Error::host(format!("Failed to read '{}': {}", path, e)))?;
        self.record(
            OperationKind::ReadFile,
            serde_json::json!({ "path": path, "size": bytes.len() }),
        )?;
        Ok(bytes)
    }

    fn write_file(&self, path: &str, content: &[u8]) -> Result<()> {
        if let Some(parent) = Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(|e| Error::host(format!("Failed to create parent dirs: {}", e)))?;
            }
        }
        fs::write(path, content)
            .map_err(|e| Error::host(format!("Failed to write '{}': {}", path, e)))?;
        self.record(
            OperationKind::WriteFile,
            serde_json::json!({ "path": path, "size": content.len() }),
        )?;
        Ok(())
    }

    fn render_template(&self, name: &str, vars: JsonValue) -> Result<String> {
        let template = self
            .templates
            .get(name)
            .cloned()
            .unwrap_or_else(|| "{{content}}".to_string());

        let mut result = template;
        if let Some(obj) = vars.as_object() {
            for (key, value) in obj {
                let placeholder = format!("{{{{{}}}}}", key);
                let replacement = match value {
                    JsonValue::String(s) => s.clone(),
                    JsonValue::Number(n) => n.to_string(),
                    JsonValue::Bool(b) => b.to_string(),
                    _ => value.to_string(),
                };
                result = result.replace(&placeholder, &replacement);
            }
        }

        self.record(
            OperationKind::RenderTemplate,
            serde_json::json!({ "template": name }),
        )?;
        Ok(result)
    }

    fn parse_openapi(&self, bytes: &[u8]) -> Result<OpenApiDoc> {
        let content = Self::parse_openapi_bytes(bytes)?;
        self.record(OperationKind::ParseOpenApi, serde_json::json!({ "parsed": true }))?;
        Ok(OpenApiDoc { content })
    }

    fn parse_markdown(&self, bytes: &[u8]) -> Result<MarkdownDoc> {
        let content = String::from_utf8(bytes.to_vec())
            .map_err(|e| Error::host(format!("Invalid UTF-8 in Markdown: {}", e)))?;
        self.record(
            OperationKind::ParseMarkdown,
            serde_json::json!({ "length": content.len() }),
        )?;
        Ok(MarkdownDoc { content })
    }

    fn export_xlsx(&self, spec: &XlsxSpec, rows: &[Row]) -> Result<Vec<u8>> {
        self.record(
            OperationKind::ExportXlsx,
            serde_json::json!({
                "sheet": spec.sheet_name,
                "headers": spec.headers,
                "rows": rows.len(),
            }),
        )?;
        Ok(format!(
            "XLSX:{}:{}:{}",
            spec.sheet_name,
            spec.headers.join(","),
            rows.len()
        )
        .into_bytes())
    }

    fn export_pdf(&self, spec: &PdfSpec, content: &str) -> Result<Vec<u8>> {
        self.record(
            OperationKind::ExportPdf,
            serde_json::json!({
                "title": spec.title,
                "content_length": content.len(),
            }),
        )?;
        Ok(format!(
            "PDF:{}:{}",
            spec.title.as_deref().unwrap_or("untitled"),
            content
        )
        .into_bytes())
    }

    fn log_operation(&self, op: Operation) -> Result<()> {
        self.operations
            .lock()
            .map_err(|e| Error::host(format!("Failed to lock operation log: {}", e)))?
            .push(op);
        Ok(())
    }
}