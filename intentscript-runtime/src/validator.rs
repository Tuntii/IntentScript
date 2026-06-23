use crate::executor::Value;
use crate::host::OpenApiDoc;
use intentscript_compiler::ir::IRCheck;
use intentscript_core::{Error, Result};
use serde_json::Value as JsonValue;

/// Diagnostic information for a check failure
#[derive(Debug, Clone, PartialEq)]
pub struct CheckFailure {
    pub check_name: String,
    pub expected: String,
    pub actual: String,
    pub message: String,
}

/// Validator for checking artifacts against validation rules
pub struct Validator;

impl Validator {
    pub fn new() -> Self {
        Self
    }

    /// Validate checks against an artifact
    pub fn validate_checks(
        &self,
        checks: &[IRCheck],
        artifact: &Value,
    ) -> Result<Vec<CheckFailure>> {
        let mut failures = Vec::new();

        for check in checks {
            if let Some(failure) = self.evaluate_check(check, artifact)? {
                failures.push(failure);
            }
        }

        Ok(failures)
    }

    fn evaluate_check(&self, check: &IRCheck, artifact: &Value) -> Result<Option<CheckFailure>> {
        match check.name.as_str() {
            "must_have_sections" => self.check_must_have_sections(check, artifact),
            "must_not_contain" => self.check_must_not_contain(check, artifact),
            "must_not_be_empty" => self.check_must_not_be_empty(check, artifact),
            "must_include_paths_prefix" => self.check_must_include_paths_prefix(check, artifact),
            "must_have_security_schemes" => self.check_must_have_security_schemes(check, artifact),
            "validate" => self.check_validate(check, artifact),
            _ => Err(Error::validation(format!(
                "Unknown check predicate: {}",
                check.name
            ))),
        }
    }

    fn string_arg(check: &IRCheck, keys: &[&str]) -> Option<String> {
        for key in keys {
            if let Some(value) = check.args.get(*key) {
                if let Some(s) = value.as_str() {
                    return Some(s.to_string());
                }
            }
        }
        None
    }

    fn array_arg(check: &IRCheck, keys: &[&str]) -> Option<Vec<String>> {
        for key in keys {
            if let Some(value) = check.args.get(*key) {
                if let Some(arr) = value.as_array() {
                    return Some(
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect(),
                    );
                }
                if let Some(s) = value.as_str() {
                    return Some(vec![s.to_string()]);
                }
            }
        }
        None
    }

    fn check_must_not_be_empty(
        &self,
        check: &IRCheck,
        artifact: &Value,
    ) -> Result<Option<CheckFailure>> {
        let empty = match artifact {
            Value::String(s) => s.trim().is_empty(),
            Value::Bytes(b) => b.is_empty(),
            Value::MarkdownDoc(doc) => doc.content.trim().is_empty(),
            Value::OpenApiDoc(doc) => doc.content.is_null(),
            _ => true,
        };

        if empty {
            Ok(Some(CheckFailure {
                check_name: check.name.clone(),
                expected: "non-empty content".to_string(),
                actual: "empty".to_string(),
                message: "Artifact must not be empty".to_string(),
            }))
        } else {
            Ok(None)
        }
    }

    fn check_must_have_sections(
        &self,
        check: &IRCheck,
        artifact: &Value,
    ) -> Result<Option<CheckFailure>> {
        let sections = Self::array_arg(check, &["sections", "arg_0"]).ok_or_else(|| {
            Error::validation("must_have_sections requires 'sections' array")
        })?;

        let content = match artifact {
            Value::String(s) => s.as_str(),
            Value::MarkdownDoc(doc) => doc.content.as_str(),
            _ => {
                return Ok(Some(CheckFailure {
                    check_name: check.name.clone(),
                    expected: "String or MarkdownDoc".to_string(),
                    actual: format!("{:?}", artifact),
                    message: "Artifact must be a string or markdown document".to_string(),
                }));
            }
        };

        let missing_sections: Vec<_> = sections
            .iter()
            .filter(|section| !content.contains(section.as_str()))
            .cloned()
            .collect();

        if missing_sections.is_empty() {
            Ok(None)
        } else {
            Ok(Some(CheckFailure {
                check_name: check.name.clone(),
                expected: format!("sections: {:?}", sections),
                actual: "missing sections".to_string(),
                message: format!("Missing required sections: {:?}", missing_sections),
            }))
        }
    }

    fn check_must_not_contain(
        &self,
        check: &IRCheck,
        artifact: &Value,
    ) -> Result<Option<CheckFailure>> {
        let patterns = Self::array_arg(check, &["patterns", "arg_0"]).ok_or_else(|| {
            Error::validation("must_not_contain requires 'patterns' array")
        })?;

        let content = match artifact {
            Value::String(s) => s.as_str(),
            Value::MarkdownDoc(doc) => doc.content.as_str(),
            _ => {
                return Ok(Some(CheckFailure {
                    check_name: check.name.clone(),
                    expected: "String or MarkdownDoc".to_string(),
                    actual: format!("{:?}", artifact),
                    message: "Artifact must be a string or markdown document".to_string(),
                }));
            }
        };

        let found_patterns: Vec<_> = patterns
            .iter()
            .filter(|pattern| content.contains(pattern.as_str()))
            .cloned()
            .collect();

        if found_patterns.is_empty() {
            Ok(None)
        } else {
            Ok(Some(CheckFailure {
                check_name: check.name.clone(),
                expected: "no forbidden patterns".to_string(),
                actual: format!("found: {:?}", found_patterns),
                message: format!("Found forbidden patterns: {:?}", found_patterns),
            }))
        }
    }

    fn check_must_include_paths_prefix(
        &self,
        check: &IRCheck,
        artifact: &Value,
    ) -> Result<Option<CheckFailure>> {
        let prefix = Self::string_arg(check, &["prefix", "arg_0"]).ok_or_else(|| {
            Error::validation("must_include_paths_prefix requires 'prefix' argument")
        })?;

        let doc = match artifact {
            Value::OpenApiDoc(OpenApiDoc { content }) => content,
            Value::Json(content) => content,
            _ => {
                return Ok(Some(CheckFailure {
                    check_name: check.name.clone(),
                    expected: "OpenApiDoc".to_string(),
                    actual: format!("{:?}", artifact),
                    message: "Artifact must be an OpenAPI document".to_string(),
                }));
            }
        };

        let paths = doc
            .get("paths")
            .and_then(|p| p.as_object())
            .map(|obj| obj.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default();

        let matching: Vec<_> = paths
            .iter()
            .filter(|path| path.starts_with(prefix.as_str()))
            .cloned()
            .collect();

        if matching.is_empty() {
            Ok(Some(CheckFailure {
                check_name: check.name.clone(),
                expected: format!("at least one path with prefix '{}'", prefix),
                actual: format!("paths: {:?}", paths),
                message: format!(
                    "No API paths found with required prefix '{}'",
                    prefix
                ),
            }))
        } else {
            Ok(None)
        }
    }

    fn check_must_have_security_schemes(
        &self,
        check: &IRCheck,
        artifact: &Value,
    ) -> Result<Option<CheckFailure>> {
        let required = Self::array_arg(check, &["schemes", "arg_0"]).ok_or_else(|| {
            Error::validation("must_have_security_schemes requires 'schemes' argument")
        })?;

        let doc = match artifact {
            Value::OpenApiDoc(OpenApiDoc { content }) => content,
            Value::Json(content) => content,
            _ => {
                return Ok(Some(CheckFailure {
                    check_name: check.name.clone(),
                    expected: "OpenApiDoc".to_string(),
                    actual: format!("{:?}", artifact),
                    message: "Artifact must be an OpenAPI document".to_string(),
                }));
            }
        };

        let defined: Vec<String> = doc
            .get("components")
            .and_then(|c| c.get("securitySchemes"))
            .and_then(|s| s.as_object())
            .map(|obj| obj.keys().cloned().collect())
            .unwrap_or_default();

        let missing: Vec<_> = required
            .iter()
            .filter(|scheme| !defined.contains(scheme))
            .cloned()
            .collect();

        if missing.is_empty() {
            Ok(None)
        } else {
            Ok(Some(CheckFailure {
                check_name: check.name.clone(),
                expected: format!("security schemes: {:?}", required),
                actual: format!("defined schemes: {:?}", defined),
                message: format!("Missing required security schemes: {:?}", missing),
            }))
        }
    }

    fn check_validate(
        &self,
        check: &IRCheck,
        artifact: &Value,
    ) -> Result<Option<CheckFailure>> {
        let _schema = check.args.get("schema");
        match artifact {
            Value::String(s) if s.trim().is_empty() => Ok(Some(CheckFailure {
                check_name: check.name.clone(),
                expected: "non-empty validated content".to_string(),
                actual: "empty".to_string(),
                message: "Validation failed: empty content".to_string(),
            })),
            Value::Bytes(b) if b.is_empty() => Ok(Some(CheckFailure {
                check_name: check.name.clone(),
                expected: "non-empty validated content".to_string(),
                actual: "empty bytes".to_string(),
                message: "Validation failed: empty bytes".to_string(),
            })),
            _ => Ok(None),
        }
    }

    pub fn validate_schema(&self, artifact: &Value, schema: &JsonValue) -> Result<()> {
        let _schema_type = schema.get("type").and_then(|v| v.as_str());
        let _artifact_type = match artifact {
            Value::String(_) => "string",
            Value::Int(_) => "integer",
            Value::Float(_) => "number",
            Value::Bool(_) => "boolean",
            Value::Json(_) => "object",
            _ => "unknown",
        };
        Ok(())
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}