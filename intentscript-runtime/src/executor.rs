use crate::audit::AuditLog;
use crate::capability::CapabilityChecker;
use crate::host::Host;
use crate::validator::{CheckFailure, Validator};
use intentscript_compiler::ir::{ExecutionPlan, IRStep, StepKind};
use intentscript_core::{Error, Result};
use serde::{Deserialize, Serialize};
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

/// Record of a validation check outcome
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationRecord {
    pub check_name: String,
    pub passed: bool,
    pub message: String,
}

/// Execution state tracking variables, artifacts, and audit log
#[derive(Debug, Clone)]
pub struct ExecutionState {
    pub plan: ExecutionPlan,
    pub variables: HashMap<String, Value>,
    pub artifacts: Vec<Artifact>,
    pub audit_log: AuditLog,
    pub repair_count: u32,
    pub validation_records: Vec<ValidationRecord>,
    pub had_validation_failures: bool,
}

impl ExecutionState {
    pub fn new(plan: ExecutionPlan) -> Self {
        Self {
            plan,
            variables: HashMap::new(),
            artifacts: Vec::new(),
            audit_log: AuditLog::new(),
            repair_count: 0,
            validation_records: Vec::new(),
            had_validation_failures: false,
        }
    }

    pub fn log(&mut self, operation: impl Into<String>, details: JsonValue) {
        self.audit_log.log(operation, details);
    }

    pub fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    pub fn add_artifact(&mut self, artifact: Artifact) {
        self.artifacts.push(artifact);
    }

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

    pub fn record_validation_failure(&mut self, failure: &CheckFailure) {
        self.had_validation_failures = true;
        self.validation_records.push(ValidationRecord {
            check_name: failure.check_name.clone(),
            passed: false,
            message: failure.message.clone(),
        });
    }

    pub fn record_validation_pass(&mut self, check_name: &str) {
        self.validation_records.push(ValidationRecord {
            check_name: check_name.to_string(),
            passed: true,
            message: "passed".to_string(),
        });
    }

    pub fn latest_validatable_artifact(&self) -> Option<&Value> {
        for step in self.plan.steps.iter().rev() {
            if let Some(var_name) = &step.produces {
                if let Some(value) = self.variables.get(var_name) {
                    match value {
                        Value::OpenApiDoc(_) | Value::MarkdownDoc(_) | Value::String(_) | Value::Bytes(_) => {
                            return Some(value);
                        }
                        _ => {}
                    }
                }
            }
        }
        None
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
    pub fn new(host: &'a dyn Host) -> Self {
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

    pub fn execute(
        &mut self,
        plan: ExecutionPlan,
        inputs: HashMap<String, JsonValue>,
    ) -> Result<ExecutionResult> {
        let mut state = ExecutionState::new(plan.clone());

        self.capability_checker = CapabilityChecker::new(plan.capabilities.clone());

        state.log(
            "execution_start",
            serde_json::json!({
                "task_name": plan.meta.task_name,
                "task_version": plan.meta.task_version,
            }),
        );

        for input_spec in &plan.inputs {
            if let Some(value) = inputs.get(&input_spec.name) {
                state.set_variable(input_spec.name.clone(), Value::Json(value.clone()));
            } else if let Some(default) = &input_spec.default {
                state.set_variable(input_spec.name.clone(), Value::Json(default.clone()));
            } else if input_spec.required {
                return Err(Error::runtime(format!(
                    "Required input '{}' not provided",
                    input_spec.name
                )));
            }
        }

        for step in &plan.steps {
            self.execute_step(step, &mut state)?;
        }

        state.log(
            "execution_complete",
            serde_json::json!({
                "success": !state.had_validation_failures,
                "validation_records": state.validation_records.len(),
            }),
        );

        Ok(ExecutionResult {
            artifacts: state.artifacts,
            audit_log: state.audit_log,
            success: !state.had_validation_failures,
        })
    }

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

        if let Some(var_name) = &step.produces {
            state.set_variable(var_name.clone(), result.clone());
        }

        if !step.checks.is_empty() && !matches!(step.kind, StepKind::Validate) {
            self.run_step_checks(step, &result, state, false)?;
        }

        state.log(
            "step_complete",
            serde_json::json!({
                "step_id": step.id,
            }),
        );

        Ok(())
    }

    fn run_step_checks(
        &self,
        step: &IRStep,
        artifact: &Value,
        state: &mut ExecutionState,
        is_validate_step: bool,
    ) -> Result<()> {
        let failures = self.validator.validate_checks(&step.checks, artifact)?;

        for check in &step.checks {
            let failed = failures.iter().any(|f| f.check_name == check.name);
            if failed {
                if let Some(failure) = failures.iter().find(|f| f.check_name == check.name) {
                    state.record_validation_failure(failure);
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
            } else {
                state.record_validation_pass(&check.name);
                state.log(
                    "check_pass",
                    serde_json::json!({
                        "step_id": step.id,
                        "check_name": check.name,
                    }),
                );
            }
        }

        if failures.is_empty() {
            return Ok(());
        }

        if is_validate_step {
            return Ok(());
        }

        if state.repair_count < state.plan.limits.max_repairs {
            state.increment_repair()?;
            state.log(
                "repair_attempt",
                serde_json::json!({
                    "step_id": step.id,
                    "repair_count": state.repair_count,
                }),
            );
            Ok(())
        } else {
            Err(Error::validation(format!(
                "Check failures in step '{}': {:?}",
                step.id, failures
            )))
        }
    }

    fn resolve_path_arg(
        &self,
        arg: Option<&JsonValue>,
        state: &ExecutionState,
    ) -> Result<String> {
        let value = arg.ok_or_else(|| Error::runtime("Missing path argument"))?;
        Self::resolve_string_value(value, state)
    }

    fn resolve_content_var(
        &self,
        step: &IRStep,
        state: &ExecutionState,
    ) -> Result<String> {
        if let Some(content) = step.args.get("content").and_then(|v| v.as_str()) {
            return Ok(content.to_string());
        }

        for prev in state.plan.steps.iter().rev() {
            if prev.id == step.id {
                break;
            }
            if let Some(var_name) = &prev.produces {
                if state.variables.contains_key(var_name) {
                    return Ok(var_name.clone());
                }
            }
        }

        Err(Error::runtime("No content variable available for parse step"))
    }

    fn resolve_string_value(value: &JsonValue, state: &ExecutionState) -> Result<String> {
        if let Some(s) = value.as_str() {
            return Ok(s.to_string());
        }

        if let Some(obj) = value.as_object() {
            if let Some(var_name) = obj.get("var").and_then(|v| v.as_str()) {
                if let Some(Value::Json(json_val)) = state.get_variable(var_name) {
                    if let Some(s) = json_val.as_str() {
                        return Ok(s.to_string());
                    }
                }
            }
        }

        Err(Error::runtime(format!(
            "Could not resolve string value from {:?}",
            value
        )))
    }

    fn execute_read_file(&self, step: &IRStep, state: &mut ExecutionState) -> Result<Value> {
        let path = self.resolve_path_arg(step.args.get("path"), state)?;
        let resolved_path = self.capability_checker.resolve_fs_read_path(&path)?;

        let bytes = self.host.read_file(&resolved_path)?;

        state.log(
            "read_file",
            serde_json::json!({
                "path": resolved_path,
                "size": bytes.len(),
            }),
        );

        Ok(Value::Bytes(bytes))
    }

    fn execute_write_file(&self, step: &IRStep, state: &mut ExecutionState) -> Result<Value> {
        let path = self.resolve_path_arg(step.args.get("path"), state)?;
        let resolved_path = self.capability_checker.resolve_fs_write_path(&path)?;

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

        self.host.write_file(&resolved_path, &bytes)?;

        state.log(
            "write_file",
            serde_json::json!({
                "path": resolved_path,
                "size": bytes.len(),
            }),
        );

        Ok(Value::Bool(true))
    }

    fn execute_parse_openapi(&self, step: &IRStep, state: &mut ExecutionState) -> Result<Value> {
        let content_var = self.resolve_content_var(step, state)?;
        let content = state
            .get_variable(&content_var)
            .ok_or_else(|| Error::runtime(format!("Variable '{}' not found", content_var)))?;

        let bytes = match content {
            Value::Bytes(b) => b,
            _ => return Err(Error::runtime("ParseOpenApi content must be Bytes")),
        };

        let doc = self.host.parse_openapi(bytes)?;

        state.log(
            "parse_openapi",
            serde_json::json!({
                "content_var": content_var,
                "title": doc.content.get("info").and_then(|i| i.get("title")),
            }),
        );

        Ok(Value::OpenApiDoc(doc))
    }

    fn execute_parse_markdown(&self, step: &IRStep, state: &mut ExecutionState) -> Result<Value> {
        let content_var = self.resolve_content_var(step, state)?;
        let content = state
            .get_variable(&content_var)
            .ok_or_else(|| Error::runtime(format!("Variable '{}' not found", content_var)))?;

        let bytes = match content {
            Value::Bytes(b) => b,
            _ => return Err(Error::runtime("ParseMarkdown content must be Bytes")),
        };

        let doc = self.host.parse_markdown(bytes)?;

        state.log(
            "parse_markdown",
            serde_json::json!({
                "content_var": content_var,
                "length": doc.content.len(),
            }),
        );

        Ok(Value::MarkdownDoc(doc))
    }

    fn execute_render_template(&self, step: &IRStep, state: &mut ExecutionState) -> Result<Value> {
        self.capability_checker.check_templates_capability()?;

        let template_name = step
            .args
            .get("template")
            .or(step.args.get("name"))
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

    fn execute_export_xlsx(&self, step: &IRStep, state: &mut ExecutionState) -> Result<Value> {
        self.capability_checker.check_exports_capability()?;

        let sheet_name = step
            .args
            .get("sheet")
            .and_then(|v| v.as_str())
            .unwrap_or("Sheet1")
            .to_string();
        let headers = step
            .args
            .get("headers")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let spec = crate::host::XlsxSpec { sheet_name, headers };
        let bytes = self.host.export_xlsx(&spec, &[])?;

        state.log("export_xlsx", serde_json::json!({ "size": bytes.len() }));

        Ok(Value::Bytes(bytes))
    }

    fn execute_export_pdf(&self, step: &IRStep, state: &mut ExecutionState) -> Result<Value> {
        self.capability_checker.check_exports_capability()?;

        let title = step
            .args
            .get("title")
            .and_then(|v| v.as_str())
            .map(String::from);
        let content = step
            .args
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let spec = crate::host::PdfSpec {
            title,
            author: None,
        };
        let bytes = self.host.export_pdf(&spec, &content)?;

        state.log("export_pdf", serde_json::json!({ "size": bytes.len() }));

        Ok(Value::Bytes(bytes))
    }

    fn execute_validate(&self, step: &IRStep, state: &mut ExecutionState) -> Result<Value> {
        let artifact = state
            .latest_validatable_artifact()
            .ok_or_else(|| Error::runtime("No artifact available for validation"))?
            .clone();

        if !step.checks.is_empty() {
            self.run_step_checks(step, &artifact, state, true)?;
        }

        let summary = serde_json::json!({
            "task": state.plan.meta.task_name,
            "goal": state.plan.meta.task_version,
            "checks": state.validation_records,
            "passed": !state.had_validation_failures,
        });

        state.log("validate", summary.clone());

        Ok(Value::Json(summary))
    }

    fn execute_report(&self, step: &IRStep, state: &mut ExecutionState) -> Result<Value> {
        let format = step
            .args
            .get("format")
            .or(step.args.get("arg_0"))
            .and_then(|v| v.as_str())
            .unwrap_or("markdown");

        let report_content = match format {
            "json" => self.build_json_report(state),
            "text" => self.build_text_report(state),
            _ => self.build_markdown_report(state),
        };

        let artifact_path = format!(
            "./artifacts/{}.{}",
            state.plan.meta.task_name,
            match format {
                "json" => "json",
                "text" => "txt",
                _ => "md",
            }
        );

        let artifact = Artifact {
            path: artifact_path.clone(),
            content: Value::String(report_content.clone()),
            type_name: format.to_string(),
        };

        state.add_artifact(artifact);

        state.log(
            "report",
            serde_json::json!({
                "format": format,
                "path": artifact_path,
                "size": report_content.len(),
                "passed": !state.had_validation_failures,
            }),
        );

        Ok(Value::String(report_content))
    }

    fn build_markdown_report(&self, state: &ExecutionState) -> String {
        let mut lines = vec![
            format!("# Validation Report: {}", state.plan.meta.task_name),
            String::new(),
            format!("**Task version:** {}", state.plan.meta.task_version),
            format!("**Policy hash:** {}", state.plan.meta.policy_hash),
            format!(
                "**Result:** {}",
                if state.had_validation_failures {
                    "FAILED"
                } else {
                    "PASSED"
                }
            ),
            String::new(),
            "## Check Results".to_string(),
        ];

        for record in &state.validation_records {
            let status = if record.passed { "PASS" } else { "FAIL" };
            lines.push(format!(
                "- [{}] **{}**: {}",
                status, record.check_name, record.message
            ));
        }

        if state.validation_records.is_empty() {
            lines.push("- No checks recorded".to_string());
        }

        lines.join("\n")
    }

    fn build_json_report(&self, state: &ExecutionState) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "task_name": state.plan.meta.task_name,
            "task_version": state.plan.meta.task_version,
            "policy_hash": state.plan.meta.policy_hash,
            "passed": !state.had_validation_failures,
            "checks": state.validation_records,
        }))
        .unwrap_or_else(|_| "{}".to_string())
    }

    fn build_text_report(&self, state: &ExecutionState) -> String {
        let status = if state.had_validation_failures {
            "FAILED"
        } else {
            "PASSED"
        };
        format!(
            "Task: {} v{} - {}\nChecks: {}",
            state.plan.meta.task_name,
            state.plan.meta.task_version,
            status,
            state.validation_records.len()
        )
    }
}