use intentscript_compiler::ir::*;
use intentscript_compiler::{Lowering, Policy, SemanticAnalyzer};
use intentscript_parser::Parser;
use intentscript_runtime::{Executor, Host, RealHost, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn compile_example(name: &str) -> ExecutionPlan {
    let root = workspace_root();
    let path = root.join("examples").join(name);
    let source = fs::read_to_string(&path).expect("example source");
    let mut parser = Parser::new(&source);
    let file = parser.parse_file().expect("parse example");
    let policy = Policy::new();
    let mut analyzer = SemanticAnalyzer::with_policy(policy.clone());
    analyzer.analyze(&file).expect("analyze example");
    let mut plan = Lowering::new(policy)
        .lower_task(&file.tasks[0])
        .expect("lower example");
    plan.capabilities.fs = Some(FsCapability {
        read_roots: vec![root.to_string_lossy().to_string()],
        write_roots: vec![],
    });
    plan
}

#[test]
fn openapi_lint_passes_with_good_spec() {
    let plan = compile_example("openapi_lint.intent");
    assert!(plan.meta.task_name.contains("RustApi"));
    assert!(!plan.meta.policy_hash.is_empty());
    assert!(!plan.steps.is_empty());

    let host = RealHost::new();
    let mut executor = Executor::new(&host);
    let mut inputs = HashMap::new();
    inputs.insert(
        "openapi_file".to_string(),
        serde_json::json!(workspace_root()
            .join("testdata/good_openapi.json")
            .to_string_lossy()
            .to_string()),
    );

    let result = executor
        .execute(plan, inputs)
        .expect("execute good openapi");

    assert!(result.success);
    assert!(!result.audit_log.is_empty());
    assert!(!result.artifacts.is_empty());

    let report = &result.artifacts[0];
    match &report.content {
        Value::String(content) => {
            assert!(content.contains("Validation Report"));
            assert!(content.contains("PASSED"));
            assert!(content.len() > 50);
        }
        other => panic!("expected string report, got {:?}", other),
    }
}

#[test]
fn openapi_lint_fails_with_bad_spec() {
    let plan = compile_example("openapi_lint.intent");
    let host = RealHost::new();
    let mut executor = Executor::new(&host);
    let mut inputs = HashMap::new();
    inputs.insert(
        "openapi_file".to_string(),
        serde_json::json!(workspace_root()
            .join("testdata/bad_openapi.json")
            .to_string_lossy()
            .to_string()),
    );

    let result = executor
        .execute(plan, inputs)
        .expect("execute bad openapi");

    assert!(!result.success);
    assert!(!result.artifacts.is_empty());

    let report = match &result.artifacts[0].content {
        Value::String(s) => s.clone(),
        other => panic!("expected string report, got {:?}", other),
    };
    assert!(report.contains("FAILED"));
    assert!(report.contains("must_include_paths_prefix") || report.contains("must_have_security_schemes"));
}

#[test]
fn real_host_parses_openapi_from_bytes() {
    let bytes = fs::read(workspace_root().join("testdata/good_openapi.json")).unwrap();
    let host = RealHost::new();
    let doc = host.parse_openapi(&bytes).expect("parse openapi");
    assert_eq!(doc.content["info"]["title"], "RustAPI Sample");
    assert!(doc.content["paths"].as_object().unwrap().contains_key("/api/users"));
}