use crate::audit::AuditLog;
use crate::capability::CapabilityChecker;
use crate::host::Host;
use crate::validator::Validator;
use intentscript_compiler::ir::{ExecutionPlan, IRStep, StepKind};
use intentscript_core::{Error, Result};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Runtime value types that can be stored in execution state
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bytes(Vec<u8>),
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Json(JsonValue),
    OpenApiDoc(crate::host::OpenApiDoc),
    MarkdownDoc(crate::host::MarkdownDoc),
}

/// An artifact produced during execution
#[derive(Debug, Clone, PartialEq)]
pub struct Artifact {
    pub path: String,
    pub content: Value,
    pub type_name: String,
}

/// Execution state tracking variables, artifacts, and audit log
#[derive(Debug, Clone)]
pub struct ExecutionState {
    pub plan: ExecutionPlan,
    pub variables: HashMap<String, Value>,
    pub artifacts: Vec<Artifact>,
    pub audit_log: AuditLog,
    pub repair_count: u32,
}

impl ExecutionState {
    /// Create a new execution state from a plan
    pub fn new(plan: ExecutionPlan) -> Self {
        Self {
            plan,
            variables: HashMap::new(),
            artifacts: Vec::new(),
            audit_log: AuditLog::new(),
            repair_count: 0,
        }
    }

    /// Add a log entry to the audit trail
    pub fn log(&mut self, operation: impl Into<String>, details: JsonValue) {
        self.audit_log.log(operation, details);
    }

    /// Store a variable value
    pub fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    /// Get a variable value
    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    /// Add an artifact
    pub fn add_artifact(&mut self, artifact: Artifact) {
        self.artifacts.push(artifact);
    }

    /// Increment repair count
    pub fn increment_repair(&mut self) -> Result<()> {
        self.repair_count += 1;
        if self.repair_count > self.plan.limits.max_repairs {
            return Err(Error::resource_limit(format!(
                "Exceeded max_repairs limit of {}",
                self.plan.limits.max_repairs
            )));
        }
        Ok(())
    }
}

/// Result of a successful execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub artifacts: Vec<Artifact>,
    pub audit_log: AuditLog,
    pub success: bool,
}

/// Runtime executor for IntentScript execution plans
pub struct Executor<'a> {
    host: &'a dyn Host,
    capability_checker: CapabilityChecker,
    validator: Validator,
}

impl<'a> Executor<'a> {
    /// Create a new executor with a Host reference
    pub fn new(host: &'a dyn Host) -> Self {
        // Default capabilities - will be overridden by execution plan
        let default_caps = intentscript_compiler::ir::Capabilities {
            fs: None,
            net: false,
            exec: false,
            templates: false,
            exports: false,
        };
        
        Self {
            host,
            capability_checker: CapabilityChecker::new(default_caps),
            validator: Validator::new(),
        }
    }

    /// Execute an execution plan with the given inputs
    /// 
    /// Lifecycle: plan -> generate -> validate -> repair -> finalize
    pub fn execute(
        &mut self,
        plan: ExecutionPlan,
        inputs: HashMap<String, JsonValue>,
    ) -> Result<ExecutionResult> {
        // Initialize execution state
        let mut state = ExecutionState::new(plan.clone());
        
        // Update capability checker with plan's capabilities
        self.capability_checker = CapabilityChecker::new(plan.capabilities.clone());
        
        // Log execution start
        state.log(
            "execution_start",
            serde_json::json!({
                "task_name": plan.meta.task_name,
                "task_version": plan.meta.task_version,
            }),
        );

        // Validate and store inputs
        for input_spec in &plan.inputs {
            if input_spec.required && !inputs.contains_key(&input_spec.name) {
                if let Some(default) = &input_spec.default {
                    state.set_variable(
                        input_spec.name.clone(),
                        Value::Json(default.clone()),
                    );
                } else {
                    return Err(Error::runtime(format!(
                        "Required input '{}' not provided",
                        input_spec.name
                    )));
                }
            } else if let Some(value) = inputs.get(&input_spec.name) {
                state.set_variable(input_spec.name.clone(), Value::Json(value.clone()));
            }
        }

        // Execute steps in sequence
        for step in &plan.steps {
            self.execute_step(step, &mut state)?;
        }

        // Finalize execution
        state.log("execution_complete", serde_json::json!({}));

        Ok(ExecutionResult {
            artifacts: state.artifacts,
            audit_log: state.audit_log,
            success: true,
        })
    }

    /// Execute a single IR step
    fn execute_step(&self, step: &IRStep, state: &mut ExecutionState) -> Result<()> {
        state.log(
            "step_start",
            serde_json::json!({
                "step_id": step.id,
                "step_kind": format!("{:?}", step.kind),
            }),
        );

        let result = match &step.kind {
            StepKind::ReadFile => self.execute_read_file(step, state)?,
            StepKind::WriteFile => self.execute_write_file(step, state)?,
            StepKind::ParseOpenApi => self.execute_parse_openapi(step, state)?,
            StepKind::ParseMarkdown => self.execute_parse_markdown(step, state)?,
            StepKind::RenderTemplate => self.execute_render_template(step, state)?,
            StepKind::ExportXlsx => self.execute_export_xlsx(step, state)?,
            StepKind::ExportPdf => self.execute_export_pdf(step, state)?,
            StepKind::Validate => self.execute_validate(step, state)?,
            StepKind::Report => self.execute_report(step, state)?,
            StepKind::Custom { name } => {
                return Err(Error::runtime(format!("Custom step '{}' not supported", name)))
            }
        };

        // Store result if step produces a variable
        if let Some(var_name) = &step.produces {
            state.set_variable(var_name.clone(), result.clone());
        }

        // Run checks if any are defined for this step
        if !step.checks.is_empty() {
            let failures = self.validator.validate_checks(&step.checks, &result)?;
            
            if !failures.is_empty() {
                // Log check failures
                for failure in &failures {
                    state.log(
                        "check_failure",
                        serde_json::json!({
                            "step_id": step.id,
                            "check_name": failure.check_name,
                            "expected": failure.expected,
                            "actual": failure.actual,
                            "message": failure.message,
                        }),
                    );
                }

                // Attempt repair if within limits
                if state.repair_count < state.plan.limits.max_repairs {
                    state.increment_repair()?;
                    state.log(
                        "repair_attempt",
                        serde_json::json!({
                            "step_id": step.id,
                            "repair_count": state.repair_count,
                        }),
                    );
                    
                    // In a full implementation, this would trigger repair logic
                    // For now, we just log and continue
                } else {
                    return Err(Error::validation(format!(
                        "Check failures in step '{}': {:?}",
                        step.id, failures
                    )));
                }
            }
        }

        state.log(
            "step_complete",
            serde_json::json!({
                "step_id": step.id,
            }),
        );

        Ok(())
    }

    fn execute_read_file(&self, step: &IRStep, state: &mut ExecutionState) -> Result<Value> {
        let path = step
            .args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::runtime("ReadFile step missing 'path' argument"))?;

        // Check filesystem read capability
        self.capability_checker.check_fs_read(path)?;

        let bytes = self.host.read_file(path)?;
        
        state.log(
            "read_file",
            serde_json::json!({
                "path": path,
                "size": bytes.len(),
            }),
        );

        Ok(Value::Bytes(bytes))
    }

    fn execute_write_file(&self, step: &IRStep, state: &mut ExecutionState) -> Result<Value> {
        let path = step
            .args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::runtime("WriteFile step missing 'path' argument"))?;

        // Check filesystem write capability
        self.capability_checker.check_fs_write(path)?;

        let content_var = step
            .args
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::runtime("WriteFile step missing 'content' argument"))?;

        let content = state
            .get_variable(content_var)
            .ok_or_else(|| Error::runtime(format!("Variable '{}' not found", content_var)))?;

        let bytes = match content {
            Value::Bytes(b) => b.clone(),
            Value::String(s) => s.as_bytes().to_vec(),
            _ => return Err(Error::runtime("WriteFile content must be Bytes or String")),
        };

        self.host.write_file(path, &bytes)?;
        
        state.log(
            "write_file",
            serde_json::json!({
                "path": path,
                "size": bytes.len(),
            }),
        );

        Ok(Value::Bool(true))
    }

    fn execute_parse_openapi(&self, step: &IRStep, state: &mut ExecutionState) -> Result<Value> {
        let content_var = step
            .args
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::runtime("ParseOpenApi step missing 'content' argument"))?;

        let content = state
            .get_variable(content_var)
            .ok_or_else(|| Error::runtime(format!("Variable '{}' not found", content_var)))?;

        let bytes = match content {
            Value::Bytes(b) => b,
            _ => return Err(Error::runtime("ParseOpenApi content must be Bytes")),
        };

        let doc = self.host.parse_openapi(bytes)?;
        
        state.log("parse_openapi", serde_json::json!({}));

        Ok(Value::OpenApiDoc(doc))
    }

    fn execute_parse_markdown(&self, step: &IRStep, state: &mut ExecutionState) -> Result<Value> {
        let content_var = step
            .args
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::runtime("ParseMarkdown step missing 'content' argument"))?;

        let content = state
            .get_variable(content_var)
            .ok_or_else(|| Error::runtime(format!("Variable '{}' not found", content_var)))?;

        let bytes = match content {
            Value::Bytes(b) => b,
            _ => return Err(Error::runtime("ParseMarkdown content must be Bytes")),
        };

        let doc = self.host.parse_markdown(bytes)?;
        
        state.log("parse_markdown", serde_json::json!({}));

        Ok(Value::MarkdownDoc(doc))
    }

    fn execute_render_template(&self, step: &IRStep, state: &mut ExecutionState) -> Result<Value> {
        // Check templates capability
        self.capability_checker.check_templates_capability()?;

        let template_name = step
            .args
            .get("template")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::runtime("RenderTemplate step missing 'template' argument"))?;

        let vars = step
            .args
            .get("vars")
            .cloned()
            .unwrap_or(JsonValue::Object(serde_json::Map::new()));

        let rendered = self.host.render_template(template_name, vars)?;
        
        state.log(
            "render_template",
            serde_json::json!({
                "template": template_name,
            }),
        );

        Ok(Value::String(rendered))
    }

    fn execute_export_xlsx(&self, _step: &IRStep, _state: &mut ExecutionState) -> Result<Value> {
        // Check exports capability
        self.capability_checker.check_exports_capability()?;
        
        // Placeholder - would need to extract spec and rows from args
        Err(Error::runtime("ExportXlsx not yet implemented"))
    }

    fn execute_export_pdf(&self, _step: &IRStep, _state: &mut ExecutionState) -> Result<Value> {
        // Check exports capability
        self.capability_checker.check_exports_capability()?;
        
        // Placeholder - would need to extract spec and content from args
        Err(Error::runtime("ExportPdf not yet implemented"))
    }

    fn execute_validate(&self, _step: &IRStep, _state: &mut ExecutionState) -> Result<Value> {
        // Placeholder - validation will be implemented in task 12.3
        Ok(Value::Bool(true))
    }

    fn execute_report(&self, _step: &IRStep, _state: &mut ExecutionState) -> Result<Value> {
        // Placeholder - reporting logic
        Ok(Value::Bool(true))
    }
}
