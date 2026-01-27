use crate::executor::Value;
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
pub struct Validator {
    // Placeholder - will be expanded as needed
}

impl Validator {
    pub fn new() -> Self {
        Self {}
    }

    /// Validate checks against an artifact
    /// 
    /// Returns Ok(()) if all checks pass, or Err with diagnostic information
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

    /// Evaluate a single check against an artifact
    fn evaluate_check(&self, check: &IRCheck, artifact: &Value) -> Result<Option<CheckFailure>> {
        match check.name.as_str() {
            "must_have_sections" => self.check_must_have_sections(check, artifact),
            "must_not_contain" => self.check_must_not_contain(check, artifact),
            "validate" => self.check_validate(check, artifact),
            _ => Err(Error::validation(format!(
                "Unknown check predicate: {}",
                check.name
            ))),
        }
    }

    /// Check that a document has required sections
    fn check_must_have_sections(
        &self,
        check: &IRCheck,
        artifact: &Value,
    ) -> Result<Option<CheckFailure>> {
        let sections = check
            .args
            .get("sections")
            .and_then(|v| v.as_array())
            .ok_or_else(|| Error::validation("must_have_sections requires 'sections' array"))?;

        let content = match artifact {
            Value::String(s) => s,
            Value::MarkdownDoc(doc) => &doc.content,
            _ => {
                return Ok(Some(CheckFailure {
                    check_name: check.name.clone(),
                    expected: "String or MarkdownDoc".to_string(),
                    actual: format!("{:?}", artifact),
                    message: "Artifact must be a string or markdown document".to_string(),
                }));
            }
        };

        let mut missing_sections = Vec::new();
        for section in sections {
            if let Some(section_name) = section.as_str() {
                if !content.contains(section_name) {
                    missing_sections.push(section_name);
                }
            }
        }

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

    /// Check that a document does not contain forbidden content
    fn check_must_not_contain(
        &self,
        check: &IRCheck,
        artifact: &Value,
    ) -> Result<Option<CheckFailure>> {
        let patterns = check
            .args
            .get("patterns")
            .and_then(|v| v.as_array())
            .ok_or_else(|| Error::validation("must_not_contain requires 'patterns' array"))?;

        let content = match artifact {
            Value::String(s) => s,
            Value::MarkdownDoc(doc) => &doc.content,
            _ => {
                return Ok(Some(CheckFailure {
                    check_name: check.name.clone(),
                    expected: "String or MarkdownDoc".to_string(),
                    actual: format!("{:?}", artifact),
                    message: "Artifact must be a string or markdown document".to_string(),
                }));
            }
        };

        let mut found_patterns = Vec::new();
        for pattern in patterns {
            if let Some(pattern_str) = pattern.as_str() {
                if content.contains(pattern_str) {
                    found_patterns.push(pattern_str);
                }
            }
        }

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

    /// Generic validation check (placeholder for schema validation)
    fn check_validate(
        &self,
        check: &IRCheck,
        _artifact: &Value,
    ) -> Result<Option<CheckFailure>> {
        // Placeholder for schema validation
        // In a full implementation, this would validate against a JSON schema or similar
        let _schema = check.args.get("schema");
        
        // For now, always pass
        Ok(None)
    }

    /// Validate that an artifact conforms to a schema
    /// 
    /// This is a placeholder for full schema validation
    pub fn validate_schema(&self, artifact: &Value, schema: &JsonValue) -> Result<()> {
        // Placeholder implementation
        // In a full implementation, this would use a JSON schema validator
        // or similar mechanism to validate the artifact structure
        
        let _schema_type = schema.get("type").and_then(|v| v.as_str());
        let _artifact_type = match artifact {
            Value::String(_) => "string",
            Value::Int(_) => "integer",
            Value::Float(_) => "number",
            Value::Bool(_) => "boolean",
            Value::Json(_) => "object",
            _ => "unknown",
        };

        // For now, always pass
        Ok(())
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}
