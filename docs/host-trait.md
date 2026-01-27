# Host Trait Guide

Guide to implementing custom Host adapters for IntentScript runtime.

## Overview

The Host trait is the interface between the IntentScript runtime and the external world. It provides capability-gated access to:

- Filesystem operations
- Template rendering
- Domain-specific parsing
- Export functions
- Audit logging

By implementing the Host trait, you can customize how IntentScript tasks interact with their environment.

## Host Trait Definition

```rust
pub trait Host {
    // Filesystem operations
    fn read_file(&self, path: &str) -> Result<Vec<u8>, HostError>;
    fn write_file(&self, path: &str, content: &[u8]) -> Result<(), HostError>;
    
    // Template operations
    fn render_template(&self, name: &str, vars: serde_json::Value) 
        -> Result<String, HostError>;
    
    // Domain parsers
    fn parse_openapi(&self, bytes: &[u8]) -> Result<OpenApiDoc, HostError>;
    fn parse_markdown(&self, bytes: &[u8]) -> Result<MarkdownDoc, HostError>;
    
    // Export operations
    fn export_xlsx(&self, spec: XlsxSpec, rows: Vec<Row>) 
        -> Result<Vec<u8>, HostError>;
    fn export_pdf(&self, spec: PdfSpec, content: &str) 
        -> Result<Vec<u8>, HostError>;
    
    // Audit logging
    fn log_operation(&self, op: Operation) -> Result<(), HostError>;
}
```

## Default Host Implementation

The default Host implementation provides standard behavior:

```rust
use intentscript_runtime::host::DefaultHost;

let host = DefaultHost::new();
let result = executor.execute(&plan, &inputs, &host)?;
```

### Features

- **Filesystem**: Standard file I/O with path validation
- **Templates**: Handlebars template engine
- **Parsers**: OpenAPI 3.0, CommonMark
- **Exports**: Basic XLSX and PDF generation
- **Logging**: File-based audit log

## Creating a Custom Host

### Step 1: Define Your Host Struct

```rust
use intentscript_runtime::host::{Host, HostError, Operation};
use serde_json::Value;

pub struct CustomHost {
    // Your custom state
    base_path: PathBuf,
    template_engine: TemplateEngine,
    audit_log: Vec<Operation>,
}

impl CustomHost {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            template_engine: TemplateEngine::new(),
            audit_log: Vec::new(),
        }
    }
}
```

### Step 2: Implement Required Methods

```rust
impl Host for CustomHost {
    fn read_file(&self, path: &str) -> Result<Vec<u8>, HostError> {
        // Validate path is within allowed roots
        let full_path = self.base_path.join(path);
        if !full_path.starts_with(&self.base_path) {
            return Err(HostError::PathViolation(path.to_string()));
        }
        
        // Read file
        std::fs::read(&full_path)
            .map_err(|e| HostError::IoError(e.to_string()))
    }
    
    fn write_file(&self, path: &str, content: &[u8]) -> Result<(), HostError> {
        // Validate and write
        let full_path = self.base_path.join(path);
        if !full_path.starts_with(&self.base_path) {
            return Err(HostError::PathViolation(path.to_string()));
        }
        
        // Create parent directories
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| HostError::IoError(e.to_string()))?;
        }
        
        std::fs::write(&full_path, content)
            .map_err(|e| HostError::IoError(e.to_string()))
    }
    
    fn render_template(&self, name: &str, vars: Value) 
        -> Result<String, HostError> 
    {
        self.template_engine.render(name, vars)
            .map_err(|e| HostError::TemplateError(e.to_string()))
    }
    
    fn parse_openapi(&self, bytes: &[u8]) -> Result<OpenApiDoc, HostError> {
        // Parse OpenAPI spec
        serde_json::from_slice(bytes)
            .or_else(|_| serde_yaml::from_slice(bytes))
            .map_err(|e| HostError::ParseError(e.to_string()))
    }
    
    fn parse_markdown(&self, bytes: &[u8]) -> Result<MarkdownDoc, HostError> {
        // Parse markdown
        let text = std::str::from_utf8(bytes)
            .map_err(|e| HostError::ParseError(e.to_string()))?;
        
        let parser = pulldown_cmark::Parser::new(text);
        // ... parse and build MarkdownDoc
        Ok(doc)
    }
    
    fn export_xlsx(&self, spec: XlsxSpec, rows: Vec<Row>) 
        -> Result<Vec<u8>, HostError> 
    {
        // Generate XLSX
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet(Some(&spec.sheet_name))?;
        
        // Write headers
        if spec.include_headers {
            for (col, header) in spec.headers.iter().enumerate() {
                worksheet.write_string(0, col as u16, header, None)?;
            }
        }
        
        // Write rows
        for (row_idx, row) in rows.iter().enumerate() {
            for (col_idx, cell) in row.cells.iter().enumerate() {
                let row_num = row_idx + if spec.include_headers { 1 } else { 0 };
                worksheet.write(row_num as u32, col_idx as u16, cell)?;
            }
        }
        
        // Serialize to bytes
        let mut buffer = Vec::new();
        workbook.save_to_buffer(&mut buffer)?;
        Ok(buffer)
    }
    
    fn export_pdf(&self, spec: PdfSpec, content: &str) 
        -> Result<Vec<u8>, HostError> 
    {
        // Generate PDF
        let doc = PdfDocument::new(&spec.title);
        // ... render content to PDF
        Ok(doc.to_bytes())
    }
    
    fn log_operation(&self, op: Operation) -> Result<(), HostError> {
        // Log operation
        self.audit_log.push(op);
        Ok(())
    }
}
```

### Step 3: Use Your Custom Host

```rust
use intentscript_runtime::Executor;

let host = CustomHost::new(PathBuf::from("./workspace"));
let executor = Executor::new();
let result = executor.execute(&plan, &inputs, &host)?;
```

## Advanced Patterns

### Sandboxed Filesystem

Restrict filesystem access to specific directories:

```rust
pub struct SandboxedHost {
    read_roots: Vec<PathBuf>,
    write_roots: Vec<PathBuf>,
}

impl SandboxedHost {
    fn validate_read_path(&self, path: &str) -> Result<PathBuf, HostError> {
        let path = PathBuf::from(path).canonicalize()
            .map_err(|e| HostError::PathViolation(e.to_string()))?;
        
        for root in &self.read_roots {
            if path.starts_with(root) {
                return Ok(path);
            }
        }
        
        Err(HostError::PathViolation(
            format!("Path {} not in allowed read roots", path.display())
        ))
    }
    
    fn validate_write_path(&self, path: &str) -> Result<PathBuf, HostError> {
        let path = PathBuf::from(path);
        
        for root in &self.write_roots {
            if path.starts_with(root) {
                return Ok(path);
            }
        }
        
        Err(HostError::PathViolation(
            format!("Path {} not in allowed write roots", path.display())
        ))
    }
}

impl Host for SandboxedHost {
    fn read_file(&self, path: &str) -> Result<Vec<u8>, HostError> {
        let validated_path = self.validate_read_path(path)?;
        std::fs::read(validated_path)
            .map_err(|e| HostError::IoError(e.to_string()))
    }
    
    fn write_file(&self, path: &str, content: &[u8]) -> Result<(), HostError> {
        let validated_path = self.validate_write_path(path)?;
        
        if let Some(parent) = validated_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| HostError::IoError(e.to_string()))?;
        }
        
        std::fs::write(validated_path, content)
            .map_err(|e| HostError::IoError(e.to_string()))
    }
    
    // ... other methods
}
```

### Network-Enabled Host

Add network capabilities:

```rust
pub struct NetworkHost {
    http_client: reqwest::blocking::Client,
    allowed_domains: Vec<String>,
}

impl NetworkHost {
    pub fn new(allowed_domains: Vec<String>) -> Self {
        Self {
            http_client: reqwest::blocking::Client::new(),
            allowed_domains,
        }
    }
    
    fn validate_url(&self, url: &str) -> Result<(), HostError> {
        let parsed = url::Url::parse(url)
            .map_err(|e| HostError::NetworkError(e.to_string()))?;
        
        if let Some(domain) = parsed.domain() {
            for allowed in &self.allowed_domains {
                if domain.ends_with(allowed) {
                    return Ok(());
                }
            }
        }
        
        Err(HostError::NetworkError(
            format!("Domain not in allowed list: {}", url)
        ))
    }
    
    pub fn http_get(&self, url: &str) -> Result<Vec<u8>, HostError> {
        self.validate_url(url)?;
        
        let response = self.http_client.get(url)
            .send()
            .map_err(|e| HostError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(HostError::NetworkError(
                format!("HTTP {} from {}", response.status(), url)
            ));
        }
        
        response.bytes()
            .map(|b| b.to_vec())
            .map_err(|e| HostError::NetworkError(e.to_string()))
    }
}
```

### Caching Host

Add caching layer:

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct CachingHost<H: Host> {
    inner: H,
    file_cache: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    template_cache: Arc<Mutex<HashMap<String, String>>>,
}

impl<H: Host> CachingHost<H> {
    pub fn new(inner: H) -> Self {
        Self {
            inner,
            file_cache: Arc::new(Mutex::new(HashMap::new())),
            template_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn clear_cache(&self) {
        self.file_cache.lock().unwrap().clear();
        self.template_cache.lock().unwrap().clear();
    }
}

impl<H: Host> Host for CachingHost<H> {
    fn read_file(&self, path: &str) -> Result<Vec<u8>, HostError> {
        // Check cache
        {
            let cache = self.file_cache.lock().unwrap();
            if let Some(content) = cache.get(path) {
                return Ok(content.clone());
            }
        }
        
        // Read from inner host
        let content = self.inner.read_file(path)?;
        
        // Cache result
        {
            let mut cache = self.file_cache.lock().unwrap();
            cache.insert(path.to_string(), content.clone());
        }
        
        Ok(content)
    }
    
    fn render_template(&self, name: &str, vars: Value) 
        -> Result<String, HostError> 
    {
        let cache_key = format!("{}:{}", name, serde_json::to_string(&vars).unwrap());
        
        // Check cache
        {
            let cache = self.template_cache.lock().unwrap();
            if let Some(rendered) = cache.get(&cache_key) {
                return Ok(rendered.clone());
            }
        }
        
        // Render with inner host
        let rendered = self.inner.render_template(name, vars)?;
        
        // Cache result
        {
            let mut cache = self.template_cache.lock().unwrap();
            cache.insert(cache_key, rendered.clone());
        }
        
        Ok(rendered)
    }
    
    // Delegate other methods to inner host
    fn write_file(&self, path: &str, content: &[u8]) -> Result<(), HostError> {
        self.inner.write_file(path, content)
    }
    
    // ... other methods
}
```

### Testing Host

Mock Host for testing:

```rust
pub struct MockHost {
    files: HashMap<String, Vec<u8>>,
    templates: HashMap<String, String>,
    operations: Vec<Operation>,
}

impl MockHost {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            templates: HashMap::new(),
            operations: Vec::new(),
        }
    }
    
    pub fn add_file(&mut self, path: &str, content: Vec<u8>) {
        self.files.insert(path.to_string(), content);
    }
    
    pub fn add_template(&mut self, name: &str, template: &str) {
        self.templates.insert(name.to_string(), template.to_string());
    }
    
    pub fn get_operations(&self) -> &[Operation] {
        &self.operations
    }
}

impl Host for MockHost {
    fn read_file(&self, path: &str) -> Result<Vec<u8>, HostError> {
        self.files.get(path)
            .cloned()
            .ok_or_else(|| HostError::IoError(format!("File not found: {}", path)))
    }
    
    fn write_file(&self, path: &str, content: &[u8]) -> Result<(), HostError> {
        self.files.insert(path.to_string(), content.to_vec());
        Ok(())
    }
    
    fn render_template(&self, name: &str, vars: Value) 
        -> Result<String, HostError> 
    {
        let template = self.templates.get(name)
            .ok_or_else(|| HostError::TemplateError(
                format!("Template not found: {}", name)
            ))?;
        
        // Simple variable substitution for testing
        let mut result = template.clone();
        if let Value::Object(map) = vars {
            for (key, value) in map {
                let placeholder = format!("{{{{{}}}}}", key);
                let replacement = value.as_str().unwrap_or("");
                result = result.replace(&placeholder, replacement);
            }
        }
        
        Ok(result)
    }
    
    fn log_operation(&self, op: Operation) -> Result<(), HostError> {
        self.operations.push(op);
        Ok(())
    }
    
    // ... other methods with mock implementations
}
```

## Error Handling

### HostError Types

```rust
pub enum HostError {
    IoError(String),
    PathViolation(String),
    NetworkError(String),
    TemplateError(String),
    ParseError(String),
    ExportError(String),
    Other(String),
}
```

### Best Practices

1. **Provide context**: Include relevant details in error messages
2. **Validate early**: Check constraints before performing operations
3. **Log operations**: Always log to audit trail
4. **Handle errors gracefully**: Don't panic, return Result
5. **Test thoroughly**: Use MockHost for unit tests

## Integration Examples

### CLI Integration

```rust
// In intentscript-cli/src/commands/run.rs

use intentscript_runtime::{Executor, host::DefaultHost};

pub fn run_command(ir_path: &str, inputs: HashMap<String, Value>) 
    -> Result<(), Error> 
{
    // Load IR
    let plan = load_execution_plan(ir_path)?;
    
    // Create host
    let host = DefaultHost::new();
    
    // Execute
    let executor = Executor::new();
    let result = executor.execute(&plan, &inputs, &host)?;
    
    // Display results
    println!("Execution completed successfully");
    println!("Artifacts: {:?}", result.artifacts);
    
    Ok(())
}
```

### Custom CLI with Custom Host

```rust
use clap::Parser;
use intentscript_runtime::Executor;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    ir_file: String,
    
    #[arg(long)]
    workspace: String,
    
    #[arg(long)]
    cache: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Load IR
    let plan = load_execution_plan(&args.ir_file)?;
    
    // Create custom host
    let base_host = SandboxedHost::new(
        vec![PathBuf::from(&args.workspace)],
        vec![PathBuf::from(&args.workspace).join("output")],
    );
    
    let host = if args.cache {
        CachingHost::new(base_host)
    } else {
        base_host
    };
    
    // Execute
    let executor = Executor::new();
    let inputs = HashMap::new(); // Parse from CLI args
    let result = executor.execute(&plan, &inputs, &host)?;
    
    println!("Success!");
    Ok(())
}
```

## See Also

- [Getting Started Guide](getting-started.md)
- [Language Reference](language-reference.md)
- [CLI Reference](cli-reference.md)
- [Runtime API Documentation](https://docs.rs/intentscript-runtime)
