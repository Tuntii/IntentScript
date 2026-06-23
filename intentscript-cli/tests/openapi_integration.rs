use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn cargo_intentscript(args: &[&str]) -> std::process::Output {
    Command::new("cargo")
        .args(["run", "--package", "intentscript-cli", "--bin", "intentscript", "--"])
        .args(args)
        .current_dir(workspace_root())
        .output()
        .expect("run intentscript cli")
}

#[test]
fn build_openapi_lint_is_deterministic() {
    let temp = TempDir::new().unwrap();
    let out1 = temp.path().join("lint1.ir.json");
    let out2 = temp.path().join("lint2.ir.json");

    let build1 = cargo_intentscript(&[
        "build",
        "examples/openapi_lint.intent",
        "--output",
        out1.to_str().unwrap(),
    ]);
    assert_eq!(build1.status.code().unwrap(), 0, "build1 failed: {:?}", build1.stderr);

    let build2 = cargo_intentscript(&[
        "build",
        "examples/openapi_lint.intent",
        "--output",
        out2.to_str().unwrap(),
    ]);
    assert_eq!(build2.status.code().unwrap(), 0);

    let ir1 = fs::read_to_string(&out1).unwrap();
    let ir2 = fs::read_to_string(&out2).unwrap();
    assert_eq!(ir1, ir2, "IR must be byte-identical across builds");

    let json: serde_json::Value = serde_json::from_str(&ir1).unwrap();
    assert!(json["meta"]["task_name"]
        .as_str()
        .unwrap()
        .contains("RustApi"));
    assert!(!json["meta"]["policy_hash"].as_str().unwrap().is_empty());
    assert!(json["steps"].as_array().unwrap().len() >= 4);
}

#[test]
fn run_openapi_lint_produces_report_artifact() {
    let temp = TempDir::new().unwrap();
    let ir_path = temp.path().join("openapi_lint.ir.json");

    let build = cargo_intentscript(&[
        "build",
        "examples/openapi_lint.intent",
        "--output",
        ir_path.to_str().unwrap(),
    ]);
    assert_eq!(build.status.code().unwrap(), 0);

    let run = cargo_intentscript(&[
        "run",
        ir_path.to_str().unwrap(),
        "--input",
        "openapi_file=./testdata/good_openapi.json",
    ]);
    assert_eq!(run.status.code().unwrap(), 0, "run failed: {:?}", run.stderr);

    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(stdout.contains("successfully") || stdout.contains("PASSED"));
    assert!(stdout.contains("Artifacts"));
    assert!(stdout.contains("Audit Log"));
}