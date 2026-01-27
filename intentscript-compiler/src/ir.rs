// IR (Intermediate Representation) data structures
// These structures represent the deterministic, serializable Execution Plan format

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The top-level execution plan that represents a compiled IntentScript task
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub schema_version: String,
    pub meta: Metadata,
    pub inputs: Vec<InputSpec>,
    pub capabilities: Capabilities,
    pub limits: Limits,
    pub steps: Vec<IRStep>,
    pub outputs: Vec<ArtifactSpec>,
}

/// Metadata about the compiled task
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    pub task_name: String,
    pub task_version: String,
    pub compiler_version: String,
    pub policy_hash: String,
}

/// Specification for a task input
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputSpec {
    pub name: String,
    pub type_name: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
}

/// Capability gates for side effects
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Capabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fs: Option<FsCapability>,
    pub net: bool,
    pub exec: bool,
    pub templates: bool,
    pub exports: bool,
}

/// Filesystem capability with read/write roots
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FsCapability {
    pub read_roots: Vec<String>,
    pub write_roots: Vec<String>,
}

/// Resource limits for execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Limits {
    pub max_repairs: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

/// A single step in the execution plan
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IRStep {
    pub id: String,
    pub kind: StepKind,
    pub args: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub produces: Option<String>,
    pub checks: Vec<IRCheck>,
}

/// The kind of operation a step performs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StepKind {
    ReadFile,
    WriteFile,
    ParseOpenApi,
    ParseMarkdown,
    RenderTemplate,
    ExportXlsx,
    ExportPdf,
    Validate,
    Report,
    Custom { name: String },
}

/// A validation check embedded in a step
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IRCheck {
    pub name: String,
    pub args: HashMap<String, serde_json::Value>,
}

/// Specification for an output artifact
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactSpec {
    pub path: String,
    pub type_name: String,
}
