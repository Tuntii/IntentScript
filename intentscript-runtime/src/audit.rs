use serde_json::Value as JsonValue;
use std::time::{SystemTime, UNIX_EPOCH};

/// A log entry in the audit trail
#[derive(Debug, Clone, PartialEq)]
pub struct LogEntry {
    pub timestamp: u64,
    pub operation: String,
    pub details: JsonValue,
}

/// Append-only audit log for tracking all operations during execution
#[derive(Debug, Clone, Default)]
pub struct AuditLog {
    entries: Vec<LogEntry>,
}

impl AuditLog {
    /// Create a new empty audit log
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Append a log entry to the audit trail
    pub fn log(&mut self, operation: impl Into<String>, details: JsonValue) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.entries.push(LogEntry {
            timestamp,
            operation: operation.into(),
            details,
        });
    }

    /// Log a Host operation
    pub fn log_host_operation(&mut self, operation: &str, path: Option<&str>, details: JsonValue) {
        let mut log_details = details;
        if let Some(p) = path {
            if let Some(obj) = log_details.as_object_mut() {
                obj.insert("path".to_string(), JsonValue::String(p.to_string()));
            }
        }

        self.log(format!("host_{}", operation), log_details);
    }

    /// Log a capability check
    pub fn log_capability_check(&mut self, capability: &str, allowed: bool, reason: Option<&str>) {
        self.log(
            "capability_check",
            serde_json::json!({
                "capability": capability,
                "allowed": allowed,
                "reason": reason,
            }),
        );
    }

    /// Log a capability violation
    pub fn log_capability_violation(&mut self, capability: &str, reason: &str) {
        self.log(
            "capability_violation",
            serde_json::json!({
                "capability": capability,
                "reason": reason,
            }),
        );
    }

    /// Log a validation result
    pub fn log_validation_result(&mut self, step_id: &str, passed: bool, failures: Option<JsonValue>) {
        self.log(
            "validation_result",
            serde_json::json!({
                "step_id": step_id,
                "passed": passed,
                "failures": failures,
            }),
        );
    }

    /// Log a repair attempt
    pub fn log_repair_attempt(&mut self, step_id: &str, repair_count: u32) {
        self.log(
            "repair_attempt",
            serde_json::json!({
                "step_id": step_id,
                "repair_count": repair_count,
            }),
        );
    }

    /// Get all log entries
    pub fn entries(&self) -> &[LogEntry] {
        &self.entries
    }

    /// Get the number of entries in the log
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the log is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all entries (for testing purposes)
    #[cfg(test)]
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_creation() {
        let log = AuditLog::new();
        assert_eq!(log.len(), 0);
        assert!(log.is_empty());
    }

    #[test]
    fn test_audit_log_append() {
        let mut log = AuditLog::new();
        log.log("test_operation", serde_json::json!({"key": "value"}));
        
        assert_eq!(log.len(), 1);
        assert!(!log.is_empty());
        
        let entry = &log.entries()[0];
        assert_eq!(entry.operation, "test_operation");
        assert_eq!(entry.details["key"], "value");
    }

    #[test]
    fn test_audit_log_host_operation() {
        let mut log = AuditLog::new();
        log.log_host_operation("read_file", Some("/tmp/test.txt"), serde_json::json!({"size": 1024}));
        
        assert_eq!(log.len(), 1);
        let entry = &log.entries()[0];
        assert_eq!(entry.operation, "host_read_file");
        assert_eq!(entry.details["path"], "/tmp/test.txt");
        assert_eq!(entry.details["size"], 1024);
    }

    #[test]
    fn test_audit_log_capability_check() {
        let mut log = AuditLog::new();
        log.log_capability_check("fs", true, Some("read access allowed"));
        
        assert_eq!(log.len(), 1);
        let entry = &log.entries()[0];
        assert_eq!(entry.operation, "capability_check");
        assert_eq!(entry.details["capability"], "fs");
        assert_eq!(entry.details["allowed"], true);
    }

    #[test]
    fn test_audit_log_capability_violation() {
        let mut log = AuditLog::new();
        log.log_capability_violation("net", "Network capability not enabled");
        
        assert_eq!(log.len(), 1);
        let entry = &log.entries()[0];
        assert_eq!(entry.operation, "capability_violation");
        assert_eq!(entry.details["capability"], "net");
    }

    #[test]
    fn test_audit_log_validation_result() {
        let mut log = AuditLog::new();
        log.log_validation_result("step_1", false, Some(serde_json::json!(["check1", "check2"])));
        
        assert_eq!(log.len(), 1);
        let entry = &log.entries()[0];
        assert_eq!(entry.operation, "validation_result");
        assert_eq!(entry.details["step_id"], "step_1");
        assert_eq!(entry.details["passed"], false);
    }

    #[test]
    fn test_audit_log_repair_attempt() {
        let mut log = AuditLog::new();
        log.log_repair_attempt("step_1", 1);
        
        assert_eq!(log.len(), 1);
        let entry = &log.entries()[0];
        assert_eq!(entry.operation, "repair_attempt");
        assert_eq!(entry.details["step_id"], "step_1");
        assert_eq!(entry.details["repair_count"], 1);
    }
}
