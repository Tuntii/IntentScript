// Integration tests for IR lowering
// Verifies end-to-end lowering from AST to ExecutionPlan

use intentscript_compiler::{Lowering, Policy};
use intentscript_core::Span;
use intentscript_parser::{
    Arg, CallExpr, CheckDecl, ConstraintDecl, ConstraintValue, Expr, InputDecl, Literal,
    Pipeline, PrimitiveType, Section, Step, Task, TypeExpr, Version,
};

fn default_span() -> Span {
    Span::new(1, 1, 0, 0)
}

#[test]
fn test_lower_simple_task() {
    // Create a simple task with an input
    let task = Task {
        name: "simple_task".to_string(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: None,
        }),
        sections: vec![Section::Input(vec![InputDecl {
            name: "input1".to_string(),
            type_expr: TypeExpr::Primitive(PrimitiveType::Text, default_span()),
            default: None,
            span: default_span(),
        }])],
        span: default_span(),
    };

    let policy = Policy::new();
    let lowering = Lowering::new(policy);

    let plan = lowering.lower_task(&task).expect("Failed to lower task");

    // Verify basic structure
    assert_eq!(plan.schema_version, "1.0");
    assert_eq!(plan.meta.task_name, "simple_task");
    assert_eq!(plan.meta.task_version, "1.0");
    assert_eq!(plan.inputs.len(), 1);
    assert_eq!(plan.inputs[0].name, "input1");
    assert_eq!(plan.inputs[0].type_name, "text");
    assert!(plan.inputs[0].required);
}

#[test]
fn test_lower_task_with_constraints() {
    // Create a task with constraints
    let task = Task {
        name: "constrained_task".to_string(),
        version: None,
        sections: vec![Section::Constraints(vec![
            ConstraintDecl {
                name: "fs".to_string(),
                value: ConstraintValue::On,
                span: default_span(),
            },
            ConstraintDecl {
                name: "net".to_string(),
                value: ConstraintValue::On,
                span: default_span(),
            },
        ])],
        span: default_span(),
    };

    let policy = Policy::new();
    let lowering = Lowering::new(policy);

    let plan = lowering.lower_task(&task).expect("Failed to lower task");

    // Verify capabilities
    assert!(plan.capabilities.fs.is_some());
    assert!(plan.capabilities.net);
    assert!(!plan.capabilities.exec);
    assert!(!plan.capabilities.templates);
    assert!(!plan.capabilities.exports);
}

#[test]
fn test_lower_task_with_pipeline() {
    // Create a task with a pipeline
    let task = Task {
        name: "pipeline_task".to_string(),
        version: None,
        sections: vec![Section::Run(Pipeline {
            steps: vec![
                Step::Call(CallExpr {
                    name: "read_file".to_string(),
                    args: vec![Arg::Named {
                        name: "path".to_string(),
                        value: Expr::Literal(Literal::String("/input.txt".to_string()), default_span()),
                    }],
                    span: default_span(),
                }),
                Step::Call(CallExpr {
                    name: "validate".to_string(),
                    args: vec![],
                    span: default_span(),
                }),
            ],
            span: default_span(),
        })],
        span: default_span(),
    };

    let policy = Policy::new();
    let lowering = Lowering::new(policy);

    let plan = lowering.lower_task(&task).expect("Failed to lower task");

    // Verify steps
    assert_eq!(plan.steps.len(), 2);
    assert_eq!(plan.steps[0].id, "step_1");
    assert_eq!(plan.steps[1].id, "step_2");
}

#[test]
fn test_lower_task_with_checks() {
    // Create a task with checks
    let task = Task {
        name: "checked_task".to_string(),
        version: None,
        sections: vec![
            Section::Checks(vec![CheckDecl {
                name: "must_not_contain".to_string(),
                args: vec![Expr::Literal(Literal::String("error".to_string()), default_span())],
                span: default_span(),
            }]),
            Section::Run(Pipeline {
                steps: vec![
                    Step::Call(CallExpr {
                        name: "read_file".to_string(),
                        args: vec![Arg::Positional(Expr::Literal(
                            Literal::String("input.txt".to_string()),
                            default_span(),
                        ))],
                        span: default_span(),
                    }),
                    Step::Ident("validate".to_string(), default_span()),
                ],
                span: default_span(),
            }),
        ],
        span: default_span(),
    };

    let policy = Policy::new();
    let lowering = Lowering::new(policy);

    let plan = lowering.lower_task(&task).expect("Failed to lower task");

    // Verify checks are embedded in steps
    assert!(!plan.steps.is_empty());
    let has_checks = plan.steps.iter().any(|step| !step.checks.is_empty());
    assert!(has_checks, "Expected at least one step to have checks");
}

#[test]
fn test_lower_task_with_version() {
    // Create a task with full version
    let task = Task {
        name: "versioned_task".to_string(),
        version: Some(Version {
            major: 2,
            minor: 3,
            patch: Some(4),
        }),
        sections: vec![],
        span: default_span(),
    };

    let policy = Policy::new();
    let lowering = Lowering::new(policy);

    let plan = lowering.lower_task(&task).expect("Failed to lower task");

    // Verify version
    assert_eq!(plan.meta.task_version, "2.3.4");
}

#[test]
fn test_lower_task_with_default_version() {
    // Create a task without version
    let task = Task {
        name: "unversioned_task".to_string(),
        version: None,
        sections: vec![],
        span: default_span(),
    };

    let policy = Policy::new();
    let lowering = Lowering::new(policy);

    let plan = lowering.lower_task(&task).expect("Failed to lower task");

    // Verify default version
    assert_eq!(plan.meta.task_version, "1.0");
}

#[test]
fn test_lower_task_serialization() {
    // Create a comprehensive task
    let task = Task {
        name: "comprehensive_task".to_string(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: None,
        }),
        sections: vec![
            Section::Input(vec![InputDecl {
                name: "api_spec".to_string(),
                type_expr: TypeExpr::Primitive(PrimitiveType::Text, default_span()),
                default: Some(Literal::String("default.yaml".to_string())),
                span: default_span(),
            }]),
            Section::Constraints(vec![ConstraintDecl {
                name: "fs".to_string(),
                value: ConstraintValue::On,
                span: default_span(),
            }]),
            Section::Run(Pipeline {
                steps: vec![Step::Call(CallExpr {
                    name: "read_file".to_string(),
                    args: vec![],
                    span: default_span(),
                })],
                span: default_span(),
            }),
        ],
        span: default_span(),
    };

    let policy = Policy::new();
    let lowering = Lowering::new(policy);

    let plan = lowering.lower_task(&task).expect("Failed to lower task");

    // Serialize to JSON
    let json = serde_json::to_string(&plan).expect("Failed to serialize");

    // Deserialize back
    let deserialized: intentscript_compiler::ExecutionPlan =
        serde_json::from_str(&json).expect("Failed to deserialize");

    // Verify round-trip
    assert_eq!(plan, deserialized);
}
