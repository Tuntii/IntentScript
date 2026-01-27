/// Host trait for capability-gated effectful operations
/// 
/// The Host trait provides the interface between the IntentScript runtime and
/// the external environment. All side effects (file I/O, network, templates, etc.)
/// are delegated through this trait, enabling capability-based security and
/// testability through mock implementations.
use intentscript_core::Result;
use serde_json::Value as JsonValue;

/// Domain-specific document types that can be parsed by the Host
#[derive(Debug, Clone, PartialEq)]
pub struct OpenApiDoc {
    pub content: JsonValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarkdownDoc {
    pub content: String,
}

/// Specification for XLSX export
#[derive(Debug, Clone, PartialEq)]
pub struct XlsxSpec {
    pub sheet_name: String,
    pub headers: Vec<String>,
}

/// Specification for PDF export
#[derive(Debug, Clone, PartialEq)]
pub struct PdfSpec {
    pub title: Option<String>,
    pub author: Option<String>,
}

/// A row of data for export operations
pub type Row = Vec<JsonValue>;

/// Operation details for audit logging
#[derive(Debug, Clone, PartialEq)]
pub struct Operation {
    pub kind: OperationKind,
    pub details: JsonValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperationKind {
    ReadFile,
    WriteFile,
    RenderTemplate,
    ParseOpenApi,
    ParseMarkdown,
    ExportXlsx,
    ExportPdf,
}

/// Host trait for capability-gated effectful operations
/// 
/// All methods return Result<T> to handle errors uniformly.
/// Implementations should use Error::host() for Host-specific errors.
pub trait Host {
    // File system operations (gated by fs capability)
    
    /// Read a file from the file system
    /// 
    /// # Arguments
    /// * `path` - The file path to read
    /// 
    /// # Returns
    /// * `Ok(Vec<u8>)` - The file contents as bytes
    /// * `Err(Error::Host)` - If the file cannot be read
    fn read_file(&self, path: &str) -> Result<Vec<u8>>;
    
    /// Write a file to the file system
    /// 
    /// # Arguments
    /// * `path` - The file path to write
    /// * `content` - The content to write as bytes
    /// 
    /// # Returns
    /// * `Ok(())` - If the file was written successfully
    /// * `Err(Error::Host)` - If the file cannot be written
    fn write_file(&self, path: &str, content: &[u8]) -> Result<()>;
    
    // Template operations (gated by templates capability)
    
    /// Render a template with the given variables
    /// 
    /// # Arguments
    /// * `name` - The template name or identifier
    /// * `vars` - Variables to substitute in the template
    /// 
    /// # Returns
    /// * `Ok(String)` - The rendered template content
    /// * `Err(Error::Host)` - If the template cannot be rendered
    fn render_template(&self, name: &str, vars: JsonValue) -> Result<String>;
    
    // Domain parsers
    
    /// Parse an OpenAPI specification from bytes
    /// 
    /// # Arguments
    /// * `bytes` - The OpenAPI specification as bytes (JSON or YAML)
    /// 
    /// # Returns
    /// * `Ok(OpenApiDoc)` - The parsed OpenAPI document
    /// * `Err(Error::Host)` - If the specification cannot be parsed
    fn parse_openapi(&self, bytes: &[u8]) -> Result<OpenApiDoc>;
    
    /// Parse a Markdown document from bytes
    /// 
    /// # Arguments
    /// * `bytes` - The Markdown content as bytes
    /// 
    /// # Returns
    /// * `Ok(MarkdownDoc)` - The parsed Markdown document
    /// * `Err(Error::Host)` - If the document cannot be parsed
    fn parse_markdown(&self, bytes: &[u8]) -> Result<MarkdownDoc>;
    
    // Export operations (gated by exports capability)
    
    /// Export data to XLSX format
    /// 
    /// # Arguments
    /// * `spec` - Specification for the XLSX file (sheet name, headers)
    /// * `rows` - Data rows to export
    /// 
    /// # Returns
    /// * `Ok(Vec<u8>)` - The XLSX file as bytes
    /// * `Err(Error::Host)` - If the export fails
    fn export_xlsx(&self, spec: &XlsxSpec, rows: &[Row]) -> Result<Vec<u8>>;
    
    /// Export content to PDF format
    /// 
    /// # Arguments
    /// * `spec` - Specification for the PDF (title, author, etc.)
    /// * `content` - The content to render as PDF
    /// 
    /// # Returns
    /// * `Ok(Vec<u8>)` - The PDF file as bytes
    /// * `Err(Error::Host)` - If the export fails
    fn export_pdf(&self, spec: &PdfSpec, content: &str) -> Result<Vec<u8>>;
    
    // Audit logging
    
    /// Log an operation for audit trail
    /// 
    /// # Arguments
    /// * `op` - The operation to log
    /// 
    /// # Returns
    /// * `Ok(())` - If the operation was logged successfully
    /// * `Err(Error::Host)` - If logging fails
    fn log_operation(&self, op: Operation) -> Result<()>;
}
