// Unit tests for Host trait
// Tests mock Host implementation and validates all Host methods

use intentscript_core::{Error, Result};
use intentscript_runtime::{Host, OpenApiDoc, MarkdownDoc, XlsxSpec, PdfSpec, Row, Operation, OperationKind};
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock Host implementation for testing
/// 
/// This mock tracks all operations and allows configuring responses
#[derive(Clone)]
struct MockHost {
    files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    templates: Arc<Mutex<HashMap<String, String>>>,
    operations: Arc<Mutex<Vec<Operation>>>,
    should_fail: Arc<Mutex<bool>>,
}

impl MockHost {
    fn new() -> Self {
        Self {
            files: Arc::new(Mutex::new(HashMap::new())),
            templates: Arc::new(Mutex::new(HashMap::new())),
            operations: Arc::new(Mutex::new(Vec::new())),
            should_fail: Arc::new(Mutex::new(false)),
        }
    }
    
    fn add_file(&self, path: &str, content: Vec<u8>) {
        self.files.lock().unwrap().insert(path.to_string(), content);
    }
    
    fn add_template(&self, name: &str, template: &str) {
        self.templates.lock().unwrap().insert(name.to_string(), template.to_string());
    }
    
    fn set_should_fail(&self, should_fail: bool) {
        *self.should_fail.lock().unwrap() = should_fail;
    }
    
    fn get_operations(&self) -> Vec<Operation> {
        self.operations.lock().unwrap().clone()
    }
    
    fn get_file(&self, path: &str) -> Option<Vec<u8>> {
        self.files.lock().unwrap().get(path).cloned()
    }
}

impl Host for MockHost {
    fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        if *self.should_fail.lock().unwrap() {
            return Err(Error::host(format!("Failed to read file: {}", path)));
        }
        
        self.files
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .ok_or_else(|| Error::host(format!("File not found: {}", path)))
    }
    
    fn write_file(&self, path: &str, content: &[u8]) -> Result<()> {
        if *self.should_fail.lock().unwrap() {
            return Err(Error::host(format!("Failed to write file: {}", path)));
        }
        
        self.files.lock().unwrap().insert(path.to_string(), content.to_vec());
        Ok(())
    }
    
    fn render_template(&self, name: &str, vars: JsonValue) -> Result<String> {
        if *self.should_fail.lock().unwrap() {
            return Err(Error::host(format!("Failed to render template: {}", name)));
        }
        
        let template = self.templates
            .lock()
            .unwrap()
            .get(name)
            .cloned()
            .ok_or_else(|| Error::host(format!("Template not found: {}", name)))?;
        
        // Simple variable substitution for testing
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
        
        Ok(result)
    }
    
    fn parse_openapi(&self, bytes: &[u8]) -> Result<OpenApiDoc> {
        if *self.should_fail.lock().unwrap() {
            return Err(Error::host("Failed to parse OpenAPI"));
        }
        
        let content: JsonValue = serde_json::from_slice(bytes)
            .map_err(|e| Error::host(format!("Invalid OpenAPI JSON: {}", e)))?;
        
        Ok(OpenApiDoc { content })
    }
    
    fn parse_markdown(&self, bytes: &[u8]) -> Result<MarkdownDoc> {
        if *self.should_fail.lock().unwrap() {
            return Err(Error::host("Failed to parse Markdown"));
        }
        
        let content = String::from_utf8(bytes.to_vec())
            .map_err(|e| Error::host(format!("Invalid UTF-8 in Markdown: {}", e)))?;
        
        Ok(MarkdownDoc { content })
    }
    
    fn export_xlsx(&self, _spec: &XlsxSpec, _rows: &[Row]) -> Result<Vec<u8>> {
        if *self.should_fail.lock().unwrap() {
            return Err(Error::host("Failed to export XLSX"));
        }
        
        // Return mock XLSX bytes
        Ok(b"MOCK_XLSX_DATA".to_vec())
    }
    
    fn export_pdf(&self, _spec: &PdfSpec, _content: &str) -> Result<Vec<u8>> {
        if *self.should_fail.lock().unwrap() {
            return Err(Error::host("Failed to export PDF"));
        }
        
        // Return mock PDF bytes
        Ok(b"MOCK_PDF_DATA".to_vec())
    }
    
    fn log_operation(&self, op: Operation) -> Result<()> {
        if *self.should_fail.lock().unwrap() {
            return Err(Error::host("Failed to log operation"));
        }
        
        self.operations.lock().unwrap().push(op);
        Ok(())
    }
}

#[test]
fn test_read_file_success() {
    let host = MockHost::new();
    let content = b"Hello, World!".to_vec();
    host.add_file("test.txt", content.clone());
    
    let result = host.read_file("test.txt");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), content);
}

#[test]
fn test_read_file_not_found() {
    let host = MockHost::new();
    
    let result = host.read_file("nonexistent.txt");
    assert!(result.is_err());
    match result {
        Err(Error::Host { message }) => {
            assert!(message.contains("File not found"));
        }
        _ => panic!("Expected Host error"),
    }
}

#[test]
fn test_read_file_failure() {
    let host = MockHost::new();
    host.add_file("test.txt", b"content".to_vec());
    host.set_should_fail(true);
    
    let result = host.read_file("test.txt");
    assert!(result.is_err());
}

#[test]
fn test_write_file_success() {
    let host = MockHost::new();
    let content = b"Test content".to_vec();
    
    let result = host.write_file("output.txt", &content);
    assert!(result.is_ok());
    
    // Verify file was written
    let read_result = host.get_file("output.txt");
    assert_eq!(read_result, Some(content));
}

#[test]
fn test_write_file_failure() {
    let host = MockHost::new();
    host.set_should_fail(true);
    
    let result = host.write_file("output.txt", b"content");
    assert!(result.is_err());
}

#[test]
fn test_render_template_success() {
    let host = MockHost::new();
    host.add_template("greeting", "Hello, {{name}}!");
    
    let vars = json!({
        "name": "Alice"
    });
    
    let result = host.render_template("greeting", vars);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello, Alice!");
}

#[test]
fn test_render_template_multiple_vars() {
    let host = MockHost::new();
    host.add_template("message", "{{greeting}}, {{name}}! You are {{age}} years old.");
    
    let vars = json!({
        "greeting": "Hi",
        "name": "Bob",
        "age": 30
    });
    
    let result = host.render_template("message", vars);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hi, Bob! You are 30 years old.");
}

#[test]
fn test_render_template_not_found() {
    let host = MockHost::new();
    
    let result = host.render_template("nonexistent", json!({}));
    assert!(result.is_err());
}

#[test]
fn test_render_template_failure() {
    let host = MockHost::new();
    host.add_template("test", "template");
    host.set_should_fail(true);
    
    let result = host.render_template("test", json!({}));
    assert!(result.is_err());
}

#[test]
fn test_parse_openapi_success() {
    let host = MockHost::new();
    let openapi_json = json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Test API",
            "version": "1.0.0"
        },
        "paths": {}
    });
    
    let bytes = serde_json::to_vec(&openapi_json).unwrap();
    let result = host.parse_openapi(&bytes);
    
    assert!(result.is_ok());
    let doc = result.unwrap();
    assert_eq!(doc.content["openapi"], "3.0.0");
    assert_eq!(doc.content["info"]["title"], "Test API");
}

#[test]
fn test_parse_openapi_invalid_json() {
    let host = MockHost::new();
    let invalid_bytes = b"not valid json";
    
    let result = host.parse_openapi(invalid_bytes);
    assert!(result.is_err());
}

#[test]
fn test_parse_openapi_failure() {
    let host = MockHost::new();
    host.set_should_fail(true);
    
    let result = host.parse_openapi(b"{}");
    assert!(result.is_err());
}

#[test]
fn test_parse_markdown_success() {
    let host = MockHost::new();
    let markdown = "# Hello\n\nThis is a test.";
    
    let result = host.parse_markdown(markdown.as_bytes());
    assert!(result.is_ok());
    
    let doc = result.unwrap();
    assert_eq!(doc.content, markdown);
}

#[test]
fn test_parse_markdown_invalid_utf8() {
    let host = MockHost::new();
    let invalid_bytes = vec![0xFF, 0xFE, 0xFD];
    
    let result = host.parse_markdown(&invalid_bytes);
    assert!(result.is_err());
}

#[test]
fn test_parse_markdown_failure() {
    let host = MockHost::new();
    host.set_should_fail(true);
    
    let result = host.parse_markdown(b"# Test");
    assert!(result.is_err());
}

#[test]
fn test_export_xlsx_success() {
    let host = MockHost::new();
    let spec = XlsxSpec {
        sheet_name: "Sheet1".to_string(),
        headers: vec!["Name".to_string(), "Age".to_string()],
    };
    let rows = vec![
        vec![json!("Alice"), json!(30)],
        vec![json!("Bob"), json!(25)],
    ];
    
    let result = host.export_xlsx(&spec, &rows);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), b"MOCK_XLSX_DATA");
}

#[test]
fn test_export_xlsx_failure() {
    let host = MockHost::new();
    host.set_should_fail(true);
    
    let spec = XlsxSpec {
        sheet_name: "Sheet1".to_string(),
        headers: vec![],
    };
    
    let result = host.export_xlsx(&spec, &[]);
    assert!(result.is_err());
}

#[test]
fn test_export_pdf_success() {
    let host = MockHost::new();
    let spec = PdfSpec {
        title: Some("Test Document".to_string()),
        author: Some("Test Author".to_string()),
    };
    let content = "This is the PDF content.";
    
    let result = host.export_pdf(&spec, content);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), b"MOCK_PDF_DATA");
}

#[test]
fn test_export_pdf_failure() {
    let host = MockHost::new();
    host.set_should_fail(true);
    
    let spec = PdfSpec {
        title: None,
        author: None,
    };
    
    let result = host.export_pdf(&spec, "content");
    assert!(result.is_err());
}

#[test]
fn test_log_operation_success() {
    let host = MockHost::new();
    let op = Operation {
        kind: OperationKind::ReadFile,
        details: json!({
            "path": "test.txt"
        }),
    };
    
    let result = host.log_operation(op.clone());
    assert!(result.is_ok());
    
    let operations = host.get_operations();
    assert_eq!(operations.len(), 1);
    assert_eq!(operations[0], op);
}

#[test]
fn test_log_operation_multiple() {
    let host = MockHost::new();
    
    let op1 = Operation {
        kind: OperationKind::ReadFile,
        details: json!({"path": "file1.txt"}),
    };
    let op2 = Operation {
        kind: OperationKind::WriteFile,
        details: json!({"path": "file2.txt"}),
    };
    
    host.log_operation(op1.clone()).unwrap();
    host.log_operation(op2.clone()).unwrap();
    
    let operations = host.get_operations();
    assert_eq!(operations.len(), 2);
    assert_eq!(operations[0], op1);
    assert_eq!(operations[1], op2);
}

#[test]
fn test_log_operation_failure() {
    let host = MockHost::new();
    host.set_should_fail(true);
    
    let op = Operation {
        kind: OperationKind::ReadFile,
        details: json!({}),
    };
    
    let result = host.log_operation(op);
    assert!(result.is_err());
}

#[test]
fn test_all_operation_kinds() {
    let host = MockHost::new();
    
    let kinds = vec![
        OperationKind::ReadFile,
        OperationKind::WriteFile,
        OperationKind::RenderTemplate,
        OperationKind::ParseOpenApi,
        OperationKind::ParseMarkdown,
        OperationKind::ExportXlsx,
        OperationKind::ExportPdf,
    ];
    
    for kind in kinds {
        let op = Operation {
            kind: kind.clone(),
            details: json!({}),
        };
        
        let result = host.log_operation(op);
        assert!(result.is_ok());
    }
    
    let operations = host.get_operations();
    assert_eq!(operations.len(), 7);
}
