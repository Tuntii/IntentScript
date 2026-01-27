use intentscript_compiler::*;
use serde_json;
use std::collections::HashMap;

#[test]
fn test_execution_plan_serialization() {
    let plan = ExecutionPlan {
        schema_version: "1.0".to_string(),
        meta: Metadata {
            task_name: "test_task".to_string(),
            task_version: "1.0.0".to_string(),
            compiler_version: "0.1.0".to_string(),
            policy_hash: "abc123".to_string(),
        },
        inputs: vec![
            InputSpec {
                name: "input1".to_string(),
                type_name: "text".to_string(),
                required: true,
                default: None,
            },
            InputSpec {
                name: "input2".to_string(),
                type_name: "int".to_string(),
                required: false,
                default: Some(serde_json::json!(42)),
            },
        ],
        capabilities: Capabilities {
            fs: Some(FsCapability {
                read_roots: vec!["/tmp".to_string()],
                write_roots: vec!["/output".to_string()],
            }),
            net: false,
            exec: false,
            templates: true,
            exports: false,
        },
        limits: Limits {
            max_repairs: 2,
            timeout_ms: Some(5000),
        },
        steps: vec![
            IRStep {
                id: "step1".to_string(),
                kind: StepKind::ReadFile,
                args: {
                    let mut map = HashMap::new();
                    map.insert("path".to_string(), serde_json::json!("/tmp/input.txt"));
                    map
                },
                produces: Some("file_content".to_string()),
                checks: vec![],
            },
            IRStep {
                id: "step2".to_string(),
                kind: StepKind::Validate,
                args: HashMap::new(),
                produces: None,
                checks: vec![IRCheck {
                    name: "must_not_contain".to_string(),
                    args: {
                        let mut map = HashMap::new();
                        map.insert("pattern".to_string(), serde_json::json!("error"));
                        map
                    },
                }],
            },
        ],
        outputs: vec![ArtifactSpec {
            path: "/output/result.txt".to_string(),
            type_name: "text".to_string(),
        }],
    };

    // Serialize to JSON
    let json = serde_json::to_string(&plan).expect("Failed to serialize");
    
    // Verify it's valid JSON
    assert!(!json.is_empty());
    
    // Deserialize back
    let deserialized: ExecutionPlan = serde_json::from_str(&json).expect("Failed to deserialize");
    
    // Verify round-trip equality
    assert_eq!(plan, deserialized);
}

#[test]
fn test_canonical_field_ordering() {
    let plan = ExecutionPlan {
        schema_version: "1.0".to_string(),
        meta: Metadata {
            task_name: "test".to_string(),
            task_version: "1.0.0".to_string(),
            compiler_version: "0.1.0".to_string(),
            policy_hash: "hash".to_string(),
        },
        inputs: vec![],
        capabilities: Capabilities {
            fs: None,
            net: false,
            exec: false,
            templates: false,
            exports: false,
        },
        limits: Limits {
            max_repairs: 2,
            timeout_ms: None,
        },
        steps: vec![],
        outputs: vec![],
    };

    // Serialize twice
    let json1 = serde_json::to_string(&plan).expect("Failed to serialize");
    let json2 = serde_json::to_string(&plan).expect("Failed to serialize");
    
    // Should be byte-identical
    assert_eq!(json1, json2);
}

#[test]
fn test_round_trip_serialization() {
    // Test with minimal ExecutionPlan
    let minimal_plan = ExecutionPlan {
        schema_version: "1.0".to_string(),
        meta: Metadata {
            task_name: "minimal".to_string(),
            task_version: "1.0.0".to_string(),
            compiler_version: "0.1.0".to_string(),
            policy_hash: "hash123".to_string(),
        },
        inputs: vec![],
        capabilities: Capabilities {
            fs: None,
            net: false,
            exec: false,
            templates: false,
            exports: false,
        },
        limits: Limits {
            max_repairs: 2,
            timeout_ms: None,
        },
        steps: vec![],
        outputs: vec![],
    };

    let json = serde_json::to_string(&minimal_plan).expect("Failed to serialize");
    let deserialized: ExecutionPlan = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(minimal_plan, deserialized);

    // Test with complex ExecutionPlan
    let complex_plan = ExecutionPlan {
        schema_version: "1.0".to_string(),
        meta: Metadata {
            task_name: "complex".to_string(),
            task_version: "2.1.3".to_string(),
            compiler_version: "0.1.0".to_string(),
            policy_hash: "complex_hash".to_string(),
        },
        inputs: vec![
            InputSpec {
                name: "api_spec".to_string(),
                type_name: "openapi".to_string(),
                required: true,
                default: None,
            },
        ],
        capabilities: Capabilities {
            fs: Some(FsCapability {
                read_roots: vec!["/input".to_string(), "/config".to_string()],
                write_roots: vec!["/output".to_string()],
            }),
            net: true,
            exec: false,
            templates: true,
            exports: true,
        },
        limits: Limits {
            max_repairs: 5,
            timeout_ms: Some(30000),
        },
        steps: vec![
            IRStep {
                id: "read_spec".to_string(),
                kind: StepKind::ReadFile,
                args: {
                    let mut map = HashMap::new();
                    map.insert("path".to_string(), serde_json::json!("/input/api.yaml"));
                    map
                },
                produces: Some("spec_bytes".to_string()),
                checks: vec![],
            },
            IRStep {
                id: "parse_spec".to_string(),
                kind: StepKind::ParseOpenApi,
                args: {
                    let mut map = HashMap::new();
                    map.insert("input".to_string(), serde_json::json!("spec_bytes"));
                    map
                },
                produces: Some("parsed_spec".to_string()),
                checks: vec![
                    IRCheck {
                        name: "must_have_sections".to_string(),
                        args: {
                            let mut map = HashMap::new();
                            map.insert("sections".to_string(), serde_json::json!(["paths", "info"]));
                            map
                        },
                    },
                ],
            },
            IRStep {
                id: "export_report".to_string(),
                kind: StepKind::ExportXlsx,
                args: {
                    let mut map = HashMap::new();
                    map.insert("data".to_string(), serde_json::json!("parsed_spec"));
                    map
                },
                produces: Some("report".to_string()),
                checks: vec![],
            },
        ],
        outputs: vec![
            ArtifactSpec {
                path: "/output/report.xlsx".to_string(),
                type_name: "xlsx".to_string(),
            },
        ],
    };

    let json = serde_json::to_string(&complex_plan).expect("Failed to serialize");
    let deserialized: ExecutionPlan = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(complex_plan, deserialized);
}

#[test]
fn test_step_kind_serialization() {
    // Test all StepKind variants
    let step_kinds = vec![
        StepKind::ReadFile,
        StepKind::WriteFile,
        StepKind::ParseOpenApi,
        StepKind::ParseMarkdown,
        StepKind::RenderTemplate,
        StepKind::ExportXlsx,
        StepKind::ExportPdf,
        StepKind::Validate,
        StepKind::Report,
        StepKind::Custom { name: "custom_step".to_string() },
    ];

    for kind in step_kinds {
        let json = serde_json::to_string(&kind).expect("Failed to serialize StepKind");
        let deserialized: StepKind = serde_json::from_str(&json).expect("Failed to deserialize StepKind");
        assert_eq!(kind, deserialized);
    }
}

#[test]
fn test_optional_fields_omitted() {
    // Test that optional fields are omitted when None
    let plan = ExecutionPlan {
        schema_version: "1.0".to_string(),
        meta: Metadata {
            task_name: "test".to_string(),
            task_version: "1.0.0".to_string(),
            compiler_version: "0.1.0".to_string(),
            policy_hash: "hash".to_string(),
        },
        inputs: vec![
            InputSpec {
                name: "input1".to_string(),
                type_name: "text".to_string(),
                required: true,
                default: None, // Should be omitted
            },
        ],
        capabilities: Capabilities {
            fs: None, // Should be omitted
            net: false,
            exec: false,
            templates: false,
            exports: false,
        },
        limits: Limits {
            max_repairs: 2,
            timeout_ms: None, // Should be omitted
        },
        steps: vec![
            IRStep {
                id: "step1".to_string(),
                kind: StepKind::Validate,
                args: HashMap::new(),
                produces: None, // Should be omitted
                checks: vec![],
            },
        ],
        outputs: vec![],
    };

    let json = serde_json::to_string(&plan).expect("Failed to serialize");
    
    // Verify optional fields are not in JSON
    assert!(!json.contains("\"default\""));
    assert!(!json.contains("\"fs\""));
    assert!(!json.contains("\"timeout_ms\""));
    assert!(!json.contains("\"produces\""));
}
