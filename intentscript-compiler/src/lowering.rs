// IR lowering pass: converts typed AST to ExecutionPlan IR

use crate::ir::*;
use crate::semantic::Policy;
use intentscript_core::Error;
use intentscript_parser::{
    Arg, CallExpr, CheckDecl, ConstraintValue, Expr, Literal, Pipeline, Section, Step, Task,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Compiler version for metadata
const COMPILER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// IR schema version
const IR_SCHEMA_VERSION: &str = "1.0";

/// Lowering pass that converts AST to IR
pub struct Lowering {
    policy: Policy,
}

impl Lowering {
    /// Create a new lowering pass with a policy
    pub fn new(policy: Policy) -> Self {
        Self { policy }
    }

    /// Lower a task to an ExecutionPlan
    pub fn lower_task(&self, task: &Task) -> Result<ExecutionPlan, Error> {
        // Extract sections
        let mut _goal_expr = None;
        let mut inputs = Vec::new();
        let mut constraints = Vec::new();
        let mut output_schema = None;
        let mut checks = Vec::new();
        let mut pipeline = None;

        for section in &task.sections {
            match section {
                Section::Goal(expr) => _goal_expr = Some(expr),
                Section::Input(input_decls) => inputs = input_decls.clone(),
                Section::Constraints(constraint_decls) => constraints = constraint_decls.clone(),
                Section::OutputSchema(type_expr) => output_schema = Some(type_expr),
                Section::Checks(check_decls) => checks = check_decls.clone(),
                Section::Run(pipe) => pipeline = Some(pipe),
            }
        }

        // Compute policy hash
        let policy_hash = self.compute_policy_hash();

        // Build metadata
        let meta = Metadata {
            task_name: task.name.clone(),
            task_version: task
                .version
                .as_ref()
                .map(|v| {
                    if let Some(patch) = v.patch {
                        format!("{}.{}.{}", v.major, v.minor, patch)
                    } else {
                        format!("{}.{}", v.major, v.minor)
                    }
                })
                .unwrap_or_else(|| "1.0".to_string()),
            compiler_version: COMPILER_VERSION.to_string(),
            policy_hash,
        };

        // Lower inputs
        let input_specs = self.lower_inputs(&inputs);

        // Lower constraints to capabilities
        let capabilities = self.lower_constraints(&constraints);

        // Build limits
        let limits = Limits {
            max_repairs: 2, // Default from spec
            timeout_ms: None,
        };

        // Lower pipeline to steps
        let steps = if let Some(pipe) = pipeline {
            self.lower_pipeline(pipe, &checks)?
        } else {
            Vec::new()
        };

        // Build output artifacts
        let outputs = if let Some(_schema) = output_schema {
            vec![ArtifactSpec {
                path: "output.json".to_string(),
                type_name: "json".to_string(),
            }]
        } else {
            Vec::new()
        };

        Ok(ExecutionPlan {
            schema_version: IR_SCHEMA_VERSION.to_string(),
            meta,
            inputs: input_specs,
            capabilities,
            limits,
            steps,
            outputs,
        })
    }

    /// Lower input declarations to InputSpec
    fn lower_inputs(&self, inputs: &[intentscript_parser::InputDecl]) -> Vec<InputSpec> {
        inputs
            .iter()
            .map(|input| {
                let type_name = self.type_expr_to_string(&input.type_expr);
                let default = input.default.as_ref().map(|lit| self.literal_to_json(lit));
                let required = input.default.is_none();

                InputSpec {
                    name: input.name.clone(),
                    type_name,
                    required,
                    default,
                }
            })
            .collect()
    }

    /// Convert a type expression to a string representation
    fn type_expr_to_string(&self, type_expr: &intentscript_parser::TypeExpr) -> String {
        use intentscript_parser::{DomainType, PrimitiveType, TypeExpr};

        match type_expr {
            TypeExpr::Primitive(prim, _) => match prim {
                PrimitiveType::Bool => "bool".to_string(),
                PrimitiveType::Int => "int".to_string(),
                PrimitiveType::Float => "float".to_string(),
                PrimitiveType::Text => "text".to_string(),
                PrimitiveType::Url => "url".to_string(),
                PrimitiveType::Email => "email".to_string(),
                PrimitiveType::Path => "path".to_string(),
                PrimitiveType::Bytes => "bytes".to_string(),
                PrimitiveType::Json => "json".to_string(),
            },
            TypeExpr::Domain(domain, _) => match domain {
                DomainType::OpenApi => "openapi".to_string(),
                DomainType::Markdown => "markdown".to_string(),
                DomainType::Xlsx => "xlsx".to_string(),
                DomainType::Pdf => "pdf".to_string(),
            },
            TypeExpr::List(inner, _) => format!("list<{}>", self.type_expr_to_string(inner)),
            TypeExpr::Optional(inner, _) => format!("optional<{}>", self.type_expr_to_string(inner)),
            TypeExpr::Enum(variants, _) => format!("enum<{}>", variants.join("|")),
            TypeExpr::Object { fields, .. } => {
                let field_strs: Vec<_> = fields
                    .iter()
                    .map(|(name, ty)| format!("{}:{}", name, self.type_expr_to_string(ty)))
                    .collect();
                format!("object<{}>", field_strs.join(","))
            }
        }
    }

    /// Convert a literal to JSON value
    fn literal_to_json(&self, literal: &Literal) -> serde_json::Value {
        match literal {
            Literal::String(s) => json!(s),
            Literal::Int(i) => json!(i),
            Literal::Float(f) => json!(f),
            Literal::Bool(b) => json!(b),
        }
    }

    /// Lower constraints to capabilities
    fn lower_constraints(&self, constraints: &[intentscript_parser::ConstraintDecl]) -> Capabilities {
        let mut fs_enabled = false;
        let mut net_enabled = false;
        let mut exec_enabled = false;
        let mut templates_enabled = false;
        let mut exports_enabled = false;

        let mut read_roots = Vec::new();
        let mut write_roots = Vec::new();

        for constraint in constraints {
            match constraint.name.as_str() {
                "fs" => {
                    if matches!(constraint.value, ConstraintValue::On) {
                        fs_enabled = true;
                    }
                }
                "fs_read" => {
                    if let ConstraintValue::Literal(Literal::String(path)) = &constraint.value {
                        read_roots.push(path.clone());
                        fs_enabled = true;
                    }
                }
                "fs_write" | "fs_write_roots" => {
                    if let ConstraintValue::Literal(Literal::String(path)) = &constraint.value {
                        write_roots.push(path.clone());
                        fs_enabled = true;
                    }
                }
                "fs_read_roots" => {
                    fs_enabled = true;
                    if let ConstraintValue::Literal(Literal::String(path)) = &constraint.value {
                        read_roots.push(path.clone());
                    } else if let ConstraintValue::Expr(Expr::Literal(Literal::String(path), _)) =
                        &constraint.value
                    {
                        read_roots.push(path.clone());
                    }
                }
                "net" => {
                    if matches!(constraint.value, ConstraintValue::On) {
                        net_enabled = true;
                    }
                }
                "exec" => {
                    if matches!(constraint.value, ConstraintValue::On) {
                        exec_enabled = true;
                    }
                }
                "templates" => {
                    if matches!(constraint.value, ConstraintValue::On) {
                        templates_enabled = true;
                    }
                }
                "exports" => {
                    if matches!(constraint.value, ConstraintValue::On) {
                        exports_enabled = true;
                    }
                }
                _ => {}
            }
        }

        let fs = if fs_enabled {
            let mut roots = read_roots;
            if roots.is_empty() {
                roots.push(".".to_string());
            }
            Some(FsCapability {
                read_roots: roots,
                write_roots,
            })
        } else {
            None
        };

        Capabilities {
            fs,
            net: net_enabled,
            exec: exec_enabled,
            templates: templates_enabled,
            exports: exports_enabled,
        }
    }

    /// Lower a pipeline to IR steps
    fn lower_pipeline(&self, pipeline: &Pipeline, checks: &[CheckDecl]) -> Result<Vec<IRStep>, Error> {
        let mut ir_steps = Vec::new();
        let mut step_counter = 0;
        let mut previous_produces: Option<String> = None;

        for step in &pipeline.steps {
            step_counter += 1;
            let step_id = format!("step_{}", step_counter);

            match step {
                Step::Call(call) => {
                    let ir_step =
                        self.lower_call_to_step(&step_id, call, checks, previous_produces.as_deref())?;
                    previous_produces = ir_step.produces.clone();
                    ir_steps.push(ir_step);
                }
                Step::Ident(name, _) => {
                    let ir_step =
                        self.lower_ident_to_step(&step_id, name, checks, previous_produces.as_deref())?;
                    previous_produces = ir_step.produces.clone();
                    ir_steps.push(ir_step);
                }
            }
        }

        Ok(ir_steps)
    }

    fn lower_ident_to_step(
        &self,
        step_id: &str,
        name: &str,
        checks: &[CheckDecl],
        previous_produces: Option<&str>,
    ) -> Result<IRStep, Error> {
        let kind = match name {
            "parse_openapi" => StepKind::ParseOpenApi,
            "parse_markdown" => StepKind::ParseMarkdown,
            "validate" => StepKind::Validate,
            "report" => StepKind::Report,
            other => StepKind::Custom {
                name: other.to_string(),
            },
        };

        let mut args = HashMap::new();
        if let Some(prev) = previous_produces {
            match kind {
                StepKind::ParseOpenApi | StepKind::ParseMarkdown => {
                    args.insert("content".to_string(), json!(prev));
                }
                _ => {}
            }
        }

        let step_checks = if matches!(kind, StepKind::Validate) {
            self.lower_checks(checks)
        } else {
            Vec::new()
        };

        Ok(IRStep {
            id: step_id.to_string(),
            kind,
            args,
            produces: Some(format!("{}_result", step_id)),
            checks: step_checks,
        })
    }

    /// Lower a call expression to an IR step
    fn lower_call_to_step(
        &self,
        step_id: &str,
        call: &CallExpr,
        checks: &[CheckDecl],
        previous_produces: Option<&str>,
    ) -> Result<IRStep, Error> {
        // Determine step kind based on function name
        let kind = match call.name.as_str() {
            "read_file" => StepKind::ReadFile,
            "write_file" => StepKind::WriteFile,
            "parse_openapi" => StepKind::ParseOpenApi,
            "parse_markdown" => StepKind::ParseMarkdown,
            "render_template" => StepKind::RenderTemplate,
            "export_xlsx" => StepKind::ExportXlsx,
            "export_pdf" => StepKind::ExportPdf,
            "validate" => StepKind::Validate,
            "report" => StepKind::Report,
            _ => StepKind::Custom {
                name: call.name.clone(),
            },
        };

        // Convert arguments with proper names for known functions
        let mut args = HashMap::new();
        
        // Handle known functions with specific parameter names
        match call.name.as_str() {
            "read_file" => {
                // read_file(path)
                if let Some(Arg::Positional(expr)) = call.args.first() {
                    let json_value = self.expr_to_json(expr)?;
                    args.insert("path".to_string(), json_value);
                }
                // Also handle named arguments
                for arg in &call.args {
                    if let Arg::Named { name, value } = arg {
                        let json_value = self.expr_to_json(value)?;
                        args.insert(name.clone(), json_value);
                    }
                }
            }
            "write_file" => {
                // write_file(path, content)
                for (i, arg) in call.args.iter().enumerate() {
                    match arg {
                        Arg::Named { name, value } => {
                            let json_value = self.expr_to_json(value)?;
                            args.insert(name.clone(), json_value);
                        }
                        Arg::Positional(expr) => {
                            let param_name = match i {
                                0 => "path",
                                1 => "content",
                                _ => "arg",
                            };
                            let json_value = self.expr_to_json(expr)?;
                            args.insert(param_name.to_string(), json_value);
                        }
                    }
                }
            }
            "render_template" => {
                // render_template(name, vars)
                for (i, arg) in call.args.iter().enumerate() {
                    match arg {
                        Arg::Named { name, value } => {
                            let json_value = self.expr_to_json(value)?;
                            args.insert(name.clone(), json_value);
                        }
                        Arg::Positional(expr) => {
                            let param_name = match i {
                                0 => "name",
                                1 => "vars",
                                _ => "arg",
                            };
                            let json_value = self.expr_to_json(expr)?;
                            args.insert(param_name.to_string(), json_value);
                        }
                    }
                }
            }
            "parse_openapi" | "parse_markdown" => {
                if let Some(prev) = previous_produces {
                    args.insert("content".to_string(), json!(prev));
                }
                for arg in &call.args {
                    if let Arg::Named { name, value } = arg {
                        let json_value = self.expr_to_json(value)?;
                        args.insert(name.clone(), json_value);
                    }
                }
            }
            "report" => {
                for (i, arg) in call.args.iter().enumerate() {
                    match arg {
                        Arg::Named { name, value } => {
                            let json_value = self.expr_to_json(value)?;
                            args.insert(name.clone(), json_value);
                        }
                        Arg::Positional(expr) => {
                            let param_name = if i == 0 { "format" } else { "arg" };
                            let json_value = self.expr_to_json(expr)?;
                            args.insert(param_name.to_string(), json_value);
                        }
                    }
                }
            }
            _ => {
                for arg in &call.args {
                    match arg {
                        Arg::Named { name, value } => {
                            let json_value = self.expr_to_json(value)?;
                            args.insert(name.clone(), json_value);
                        }
                        Arg::Positional(expr) => {
                            let index = args.len();
                            let json_value = self.expr_to_json(expr)?;
                            args.insert(format!("arg_{}", index), json_value);
                        }
                    }
                }
            }
        }

        let step_checks = if matches!(kind, StepKind::Validate) {
            self.lower_checks(checks)
        } else {
            Vec::new()
        };

        Ok(IRStep {
            id: step_id.to_string(),
            kind,
            args,
            produces: Some(format!("{}_result", step_id)),
            checks: step_checks,
        })
    }

    /// Lower check declarations to IR checks
    fn lower_checks(&self, checks: &[CheckDecl]) -> Vec<IRCheck> {
        checks
            .iter()
            .map(|check| {
                let mut args = HashMap::new();

                match check.name.as_str() {
                    "must_include_paths_prefix" | "must_not_be_empty" => {
                        if let Some(arg) = check.args.first() {
                            if let Ok(json_value) = self.expr_to_json(arg) {
                                let key = if check.name == "must_include_paths_prefix" {
                                    "prefix"
                                } else {
                                    "arg_0"
                                };
                                args.insert(key.to_string(), json_value);
                            }
                        }
                    }
                    "must_have_security_schemes" => {
                        if let Some(Expr::Literal(Literal::String(scheme), _)) = check.args.first()
                        {
                            args.insert("schemes".to_string(), json!(vec![scheme]));
                        } else if let Some(arg) = check.args.first() {
                            if let Ok(json_value) = self.expr_to_json(arg) {
                                args.insert("schemes".to_string(), json!(vec![json_value]));
                            }
                        }
                    }
                    "must_have_sections" | "must_not_contain" => {
                        let key = if check.name == "must_have_sections" {
                            "sections"
                        } else {
                            "patterns"
                        };
                        if let Some(arg) = check.args.first() {
                            if let Ok(json_value) = self.expr_to_json(arg) {
                                args.insert(key.to_string(), json_value);
                            }
                        }
                    }
                    _ => {
                        for (i, arg) in check.args.iter().enumerate() {
                            if let Ok(json_value) = self.expr_to_json(arg) {
                                args.insert(format!("arg_{}", i), json_value);
                            }
                        }
                    }
                }

                IRCheck {
                    name: check.name.clone(),
                    args,
                }
            })
            .collect()
    }

    /// Convert an expression to JSON value
    fn expr_to_json(&self, expr: &Expr) -> Result<serde_json::Value, Error> {
        match expr {
            Expr::Literal(lit, _) => Ok(self.literal_to_json(lit)),
            Expr::Ident(name, _) => Ok(json!({ "var": name })),
            Expr::Call(call) => {
                let mut call_obj = serde_json::Map::new();
                call_obj.insert("function".to_string(), json!(call.name));

                let mut args_array = Vec::new();
                for arg in &call.args {
                    match arg {
                        Arg::Named { name, value } => {
                            let val = self.expr_to_json(value)?;
                            args_array.push(json!({ "name": name, "value": val }));
                        }
                        Arg::Positional(value) => {
                            let val = self.expr_to_json(value)?;
                            args_array.push(val);
                        }
                    }
                }
                call_obj.insert("args".to_string(), json!(args_array));

                Ok(serde_json::Value::Object(call_obj))
            }
        }
    }

    /// Compute policy hash using SHA-256
    fn compute_policy_hash(&self) -> String {
        let mut hasher = Sha256::new();

        // Serialize policy constraints in a deterministic order
        let mut constraint_keys: Vec<_> = self.policy.constraints.keys().collect();
        constraint_keys.sort();

        for key in constraint_keys {
            if let Some(value) = self.policy.constraints.get(key) {
                hasher.update(key.as_bytes());
                hasher.update(b":");
                let value_str = format!("{:?}", value);
                hasher.update(value_str.as_bytes());
                hasher.update(b";");
            }
        }

        // Include ambiguity resolution flag
        let ambiguity_flag = if self.policy.allow_ambiguity_resolution {
            "allow_ambiguity:true"
        } else {
            "allow_ambiguity:false"
        };
        hasher.update(ambiguity_flag.as_bytes());

        // Return hex-encoded hash
        format!("{:x}", hasher.finalize())
    }
}

impl Default for Lowering {
    fn default() -> Self {
        Self::new(Policy::new())
    }
}
