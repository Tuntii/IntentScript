// Property-based tests for IR lowering
// Feature: intentscript-compiler

use intentscript_compiler::ir::StepKind;
use intentscript_compiler::{Lowering, Policy};
use intentscript_core::Span;
use intentscript_parser::{
    InputDecl, Literal, PrimitiveType, Section, Task, TypeExpr, Version,
    ConstraintDecl, ConstraintValue, CheckDecl, Expr, Pipeline, Step, CallExpr, Arg,
};
use quickcheck::{Arbitrary, Gen, TestResult};
use quickcheck_macros::quickcheck;
use serde_json;

// Helper to create a default span
fn default_span() -> Span {
    Span::new(1, 1, 0, 0)
}

// Generate a valid identifier name
fn gen_identifier(g: &mut Gen) -> String {
    let first_chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";
    let rest_chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_";
    
    let mut name = String::new();
    name.push(*g.choose(first_chars.as_bytes()).unwrap() as char);
    
    let len = (usize::arbitrary(g) % 10) + 1;
    for _ in 0..len {
        name.push(*g.choose(rest_chars.as_bytes()).unwrap() as char);
    }
    
    name
}

// Wrapper types for implementing Arbitrary
#[derive(Debug, Clone)]
struct ArbitraryPrimitiveType(PrimitiveType);

impl Arbitrary for ArbitraryPrimitiveType {
    fn arbitrary(g: &mut Gen) -> Self {
        let types = vec![
            PrimitiveType::Bool,
            PrimitiveType::Int,
            PrimitiveType::Float,
            PrimitiveType::Text,
            PrimitiveType::Url,
            PrimitiveType::Email,
            PrimitiveType::Path,
            PrimitiveType::Bytes,
            PrimitiveType::Json,
        ];
        ArbitraryPrimitiveType(g.choose(&types).unwrap().clone())
    }
}

/// **Feature: intentscript-compiler, Property 28: Compilation determinism**
/// **Validates: Requirements 8.1**
///
/// For any IntentScript source, compiling it twice with identical compiler version,
/// policy hash, and inputs should produce byte-identical IR.
#[quickcheck]
fn property_compilation_determinism(prim_type: ArbitraryPrimitiveType) -> TestResult {
    let mut g = Gen::new(10);
    let task_name = gen_identifier(&mut g);
    let input_name = gen_identifier(&mut g);
    
    // Create a simple task
    let task = Task {
        name: task_name.clone(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: None,
        }),
        sections: vec![
            Section::Input(vec![InputDecl {
                name: input_name.clone(),
                type_expr: TypeExpr::Primitive(prim_type.0, default_span()),
                default: None,
                span: default_span(),
            }]),
            Section::Run(Pipeline {
                steps: vec![Step::Ident(input_name, default_span())],
                span: default_span(),
            }),
        ],
        span: default_span(),
    };
    
    // Create the same policy for both compilations
    let policy = Policy::new();
    
    // Lower the task twice
    let lowering1 = Lowering::new(policy.clone());
    let lowering2 = Lowering::new(policy.clone());
    
    let plan1 = match lowering1.lower_task(&task) {
        Ok(plan) => plan,
        Err(e) => {
            println!("Failed to lower task: {:?}", e);
            return TestResult::failed();
        }
    };
    
    let plan2 = match lowering2.lower_task(&task) {
        Ok(plan) => plan,
        Err(e) => {
            println!("Failed to lower task: {:?}", e);
            return TestResult::failed();
        }
    };
    
    // Serialize both plans to JSON
    let json1 = serde_json::to_string(&plan1).expect("Failed to serialize plan1");
    let json2 = serde_json::to_string(&plan2).expect("Failed to serialize plan2");
    
    // They should be byte-identical
    if json1 == json2 {
        TestResult::passed()
    } else {
        println!("Plans are not byte-identical:");
        println!("Plan 1: {}", json1);
        println!("Plan 2: {}", json2);
        TestResult::failed()
    }
}

/// **Feature: intentscript-compiler, Property 28: Compilation determinism with constraints**
/// **Validates: Requirements 8.1**
///
/// For any task with constraints, compiling it multiple times should produce
/// byte-identical IR.
#[quickcheck]
fn property_compilation_determinism_with_constraints() -> TestResult {
    let mut g = Gen::new(10);
    let task_name = gen_identifier(&mut g);
    let constraint_name = gen_identifier(&mut g);
    
    // Create a task with constraints
    let task = Task {
        name: task_name.clone(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: Some(0),
        }),
        sections: vec![
            Section::Constraints(vec![
                ConstraintDecl {
                    name: constraint_name.clone(),
                    value: ConstraintValue::On,
                    span: default_span(),
                },
            ]),
        ],
        span: default_span(),
    };
    
    let policy = Policy::new();
    
    // Lower the task twice
    let lowering1 = Lowering::new(policy.clone());
    let lowering2 = Lowering::new(policy.clone());
    
    let plan1 = match lowering1.lower_task(&task) {
        Ok(plan) => plan,
        Err(e) => {
            println!("Failed to lower task: {:?}", e);
            return TestResult::failed();
        }
    };
    
    let plan2 = match lowering2.lower_task(&task) {
        Ok(plan) => plan,
        Err(e) => {
            println!("Failed to lower task: {:?}", e);
            return TestResult::failed();
        }
    };
    
    // Serialize both plans to JSON
    let json1 = serde_json::to_string(&plan1).expect("Failed to serialize plan1");
    let json2 = serde_json::to_string(&plan2).expect("Failed to serialize plan2");
    
    // They should be byte-identical
    if json1 == json2 {
        TestResult::passed()
    } else {
        println!("Plans with constraints are not byte-identical");
        TestResult::failed()
    }
}


/// **Feature: intentscript-compiler, Property 29: IR serialization determinism**
/// **Validates: Requirements 8.2**
///
/// For any ExecutionPlan IR, serializing it to JSON multiple times should produce
/// byte-identical output.
#[quickcheck]
fn property_ir_serialization_determinism(prim_type: ArbitraryPrimitiveType) -> TestResult {
    let mut g = Gen::new(10);
    let task_name = gen_identifier(&mut g);
    let input_name = gen_identifier(&mut g);
    
    // Create a task and lower it to IR
    let task = Task {
        name: task_name.clone(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: None,
        }),
        sections: vec![
            Section::Input(vec![InputDecl {
                name: input_name.clone(),
                type_expr: TypeExpr::Primitive(prim_type.0, default_span()),
                default: None,
                span: default_span(),
            }]),
        ],
        span: default_span(),
    };
    
    let policy = Policy::new();
    let lowering = Lowering::new(policy);
    
    let plan = match lowering.lower_task(&task) {
        Ok(plan) => plan,
        Err(e) => {
            println!("Failed to lower task: {:?}", e);
            return TestResult::failed();
        }
    };
    
    // Serialize the same plan multiple times
    let json1 = serde_json::to_string(&plan).expect("Failed to serialize plan (1)");
    let json2 = serde_json::to_string(&plan).expect("Failed to serialize plan (2)");
    let json3 = serde_json::to_string(&plan).expect("Failed to serialize plan (3)");
    
    // All serializations should be byte-identical
    if json1 == json2 && json2 == json3 {
        TestResult::passed()
    } else {
        println!("Serializations are not byte-identical");
        TestResult::failed()
    }
}

/// **Feature: intentscript-compiler, Property 29: IR serialization determinism with complex plan**
/// **Validates: Requirements 8.2**
///
/// For any complex ExecutionPlan with multiple steps and checks, serializing it
/// multiple times should produce byte-identical output.
#[quickcheck]
fn property_ir_serialization_determinism_complex() -> TestResult {
    let mut g = Gen::new(10);
    let task_name = gen_identifier(&mut g);
    let func_name = gen_identifier(&mut g);
    
    // Create a task with a pipeline
    let task = Task {
        name: task_name.clone(),
        version: Some(Version {
            major: 2,
            minor: 1,
            patch: Some(3),
        }),
        sections: vec![
            Section::Constraints(vec![
                ConstraintDecl {
                    name: "fs".to_string(),
                    value: ConstraintValue::On,
                    span: default_span(),
                },
            ]),
            Section::Checks(vec![
                CheckDecl {
                    name: "validate".to_string(),
                    args: vec![Expr::Literal(Literal::String("test".to_string()), default_span())],
                    span: default_span(),
                },
            ]),
            Section::Run(Pipeline {
                steps: vec![
                    Step::Call(CallExpr {
                        name: func_name.clone(),
                        args: vec![Arg::Positional(Expr::Literal(Literal::Int(42), default_span()))],
                        span: default_span(),
                    }),
                ],
                span: default_span(),
            }),
        ],
        span: default_span(),
    };
    
    let policy = Policy::new();
    let lowering = Lowering::new(policy);
    
    let plan = match lowering.lower_task(&task) {
        Ok(plan) => plan,
        Err(e) => {
            println!("Failed to lower task: {:?}", e);
            return TestResult::failed();
        }
    };
    
    // Serialize the same plan multiple times
    let json1 = serde_json::to_string(&plan).expect("Failed to serialize plan (1)");
    let json2 = serde_json::to_string(&plan).expect("Failed to serialize plan (2)");
    let json3 = serde_json::to_string(&plan).expect("Failed to serialize plan (3)");
    
    // All serializations should be byte-identical
    if json1 == json2 && json2 == json3 {
        TestResult::passed()
    } else {
        println!("Complex plan serializations are not byte-identical");
        TestResult::failed()
    }
}

/// **Feature: intentscript-compiler, Property 30: Policy hash stability**
/// **Validates: Requirements 8.3**
///
/// For any policy, computing its hash multiple times should produce identical results,
/// and any change to the policy should produce a different hash.
#[quickcheck]
fn property_policy_hash_stability() -> TestResult {
    let mut g = Gen::new(10);
    let constraint_name = gen_identifier(&mut g);
    
    // Create a policy
    let mut policy = Policy::new();
    policy.add_constraint(constraint_name.clone(), ConstraintValue::On);
    
    // Create a minimal task to trigger lowering
    let task = Task {
        name: "test".to_string(),
        version: None,
        sections: vec![],
        span: default_span(),
    };
    
    // Lower the task multiple times with the same policy
    let lowering1 = Lowering::new(policy.clone());
    let lowering2 = Lowering::new(policy.clone());
    let lowering3 = Lowering::new(policy.clone());
    
    let plan1 = lowering1.lower_task(&task).expect("Failed to lower task");
    let plan2 = lowering2.lower_task(&task).expect("Failed to lower task");
    let plan3 = lowering3.lower_task(&task).expect("Failed to lower task");
    
    // All policy hashes should be identical
    if plan1.meta.policy_hash == plan2.meta.policy_hash
        && plan2.meta.policy_hash == plan3.meta.policy_hash
    {
        TestResult::passed()
    } else {
        println!("Policy hashes are not stable:");
        println!("Hash 1: {}", plan1.meta.policy_hash);
        println!("Hash 2: {}", plan2.meta.policy_hash);
        println!("Hash 3: {}", plan3.meta.policy_hash);
        TestResult::failed()
    }
}

/// **Feature: intentscript-compiler, Property 30: Policy hash changes with policy**
/// **Validates: Requirements 8.3**
///
/// For any two different policies, their hashes should be different.
#[quickcheck]
fn property_policy_hash_changes_with_policy() -> TestResult {
    let mut g = Gen::new(10);
    let constraint_name = gen_identifier(&mut g);
    
    // Create two different policies
    let mut policy1 = Policy::new();
    policy1.add_constraint(constraint_name.clone(), ConstraintValue::On);
    
    let mut policy2 = Policy::new();
    policy2.add_constraint(constraint_name.clone(), ConstraintValue::Off);
    
    // Create a minimal task
    let task = Task {
        name: "test".to_string(),
        version: None,
        sections: vec![],
        span: default_span(),
    };
    
    // Lower with both policies
    let lowering1 = Lowering::new(policy1);
    let lowering2 = Lowering::new(policy2);
    
    let plan1 = lowering1.lower_task(&task).expect("Failed to lower task");
    let plan2 = lowering2.lower_task(&task).expect("Failed to lower task");
    
    // Policy hashes should be different
    if plan1.meta.policy_hash != plan2.meta.policy_hash {
        TestResult::passed()
    } else {
        println!("Policy hashes are the same for different policies:");
        println!("Hash 1: {}", plan1.meta.policy_hash);
        println!("Hash 2: {}", plan2.meta.policy_hash);
        TestResult::failed()
    }
}

/// **Feature: intentscript-compiler, Property 30: Policy hash changes with ambiguity flag**
/// **Validates: Requirements 8.3**
///
/// For any policy, changing the allow_ambiguity_resolution flag should change the hash.
#[quickcheck]
fn property_policy_hash_changes_with_ambiguity_flag() -> TestResult {
    // Create two policies that differ only in ambiguity resolution flag
    let mut policy1 = Policy::new();
    policy1.allow_ambiguity_resolution = false;
    
    let mut policy2 = Policy::new();
    policy2.allow_ambiguity_resolution = true;
    
    // Create a minimal task
    let task = Task {
        name: "test".to_string(),
        version: None,
        sections: vec![],
        span: default_span(),
    };
    
    // Lower with both policies
    let lowering1 = Lowering::new(policy1);
    let lowering2 = Lowering::new(policy2);
    
    let plan1 = lowering1.lower_task(&task).expect("Failed to lower task");
    let plan2 = lowering2.lower_task(&task).expect("Failed to lower task");
    
    // Policy hashes should be different
    if plan1.meta.policy_hash != plan2.meta.policy_hash {
        TestResult::passed()
    } else {
        println!("Policy hashes are the same despite different ambiguity flags:");
        println!("Hash 1: {}", plan1.meta.policy_hash);
        println!("Hash 2: {}", plan2.meta.policy_hash);
        TestResult::failed()
    }
}

/// **Feature: intentscript-compiler, Property 31: IR metadata completeness**
/// **Validates: Requirements 8.4**
///
/// For any generated ExecutionPlan, the metadata should contain schema_version,
/// task_name, task_version, compiler_version, and policy_hash.
#[quickcheck]
fn property_ir_metadata_completeness(prim_type: ArbitraryPrimitiveType) -> TestResult {
    let mut g = Gen::new(10);
    let task_name = gen_identifier(&mut g);
    let input_name = gen_identifier(&mut g);
    
    // Create a task with version
    let task = Task {
        name: task_name.clone(),
        version: Some(Version {
            major: 2,
            minor: 3,
            patch: Some(4),
        }),
        sections: vec![
            Section::Input(vec![InputDecl {
                name: input_name.clone(),
                type_expr: TypeExpr::Primitive(prim_type.0, default_span()),
                default: None,
                span: default_span(),
            }]),
        ],
        span: default_span(),
    };
    
    let policy = Policy::new();
    let lowering = Lowering::new(policy);
    
    let plan = match lowering.lower_task(&task) {
        Ok(plan) => plan,
        Err(e) => {
            println!("Failed to lower task: {:?}", e);
            return TestResult::failed();
        }
    };
    
    // Check that all metadata fields are present and non-empty
    let has_schema_version = !plan.schema_version.is_empty();
    let has_task_name = !plan.meta.task_name.is_empty();
    let has_task_version = !plan.meta.task_version.is_empty();
    let has_compiler_version = !plan.meta.compiler_version.is_empty();
    let has_policy_hash = !plan.meta.policy_hash.is_empty();
    
    if has_schema_version && has_task_name && has_task_version && has_compiler_version && has_policy_hash {
        // Verify specific values
        if plan.schema_version == "1.0"
            && plan.meta.task_name == task_name
            && plan.meta.task_version == "2.3.4"
        {
            TestResult::passed()
        } else {
            println!("Metadata values are incorrect:");
            println!("  schema_version: {}", plan.schema_version);
            println!("  task_name: {}", plan.meta.task_name);
            println!("  task_version: {}", plan.meta.task_version);
            TestResult::failed()
        }
    } else {
        println!("Missing metadata fields:");
        println!("  has_schema_version: {}", has_schema_version);
        println!("  has_task_name: {}", has_task_name);
        println!("  has_task_version: {}", has_task_version);
        println!("  has_compiler_version: {}", has_compiler_version);
        println!("  has_policy_hash: {}", has_policy_hash);
        TestResult::failed()
    }
}

/// **Feature: intentscript-compiler, Property 31: IR metadata with no version**
/// **Validates: Requirements 8.4**
///
/// For any task without an explicit version, the metadata should still contain
/// a default task_version.
#[quickcheck]
fn property_ir_metadata_default_version() -> TestResult {
    let mut g = Gen::new(10);
    let task_name = gen_identifier(&mut g);
    
    // Create a task without version
    let task = Task {
        name: task_name.clone(),
        version: None,
        sections: vec![],
        span: default_span(),
    };
    
    let policy = Policy::new();
    let lowering = Lowering::new(policy);
    
    let plan = match lowering.lower_task(&task) {
        Ok(plan) => plan,
        Err(e) => {
            println!("Failed to lower task: {:?}", e);
            return TestResult::failed();
        }
    };
    
    // Check that metadata is complete
    if !plan.meta.task_version.is_empty() {
        // Should have a default version
        if plan.meta.task_version == "1.0" {
            TestResult::passed()
        } else {
            println!("Unexpected default version: {}", plan.meta.task_version);
            TestResult::failed()
        }
    } else {
        println!("Missing task_version in metadata");
        TestResult::failed()
    }
}

/// **Feature: intentscript-compiler, Property 32: ExecutionPlan structure completeness**
/// **Validates: Requirements 9.1**
///
/// For any lowered task, the ExecutionPlan should contain all required fields:
/// meta, inputs, capabilities, limits, steps, and outputs.
#[quickcheck]
fn property_execution_plan_structure_completeness(prim_type: ArbitraryPrimitiveType) -> TestResult {
    let mut g = Gen::new(10);
    let task_name = gen_identifier(&mut g);
    let input_name = gen_identifier(&mut g);
    
    // Create a task with various sections
    let task = Task {
        name: task_name.clone(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: None,
        }),
        sections: vec![
            Section::Input(vec![InputDecl {
                name: input_name.clone(),
                type_expr: TypeExpr::Primitive(prim_type.0, default_span()),
                default: None,
                span: default_span(),
            }]),
            Section::Constraints(vec![
                ConstraintDecl {
                    name: "fs".to_string(),
                    value: ConstraintValue::On,
                    span: default_span(),
                },
            ]),
            Section::Run(Pipeline {
                steps: vec![Step::Ident(input_name, default_span())],
                span: default_span(),
            }),
        ],
        span: default_span(),
    };
    
    let policy = Policy::new();
    let lowering = Lowering::new(policy);
    
    let plan = match lowering.lower_task(&task) {
        Ok(plan) => plan,
        Err(e) => {
            println!("Failed to lower task: {:?}", e);
            return TestResult::failed();
        }
    };
    
    // Verify all required fields are present
    // meta is always present (checked in previous property)
    // inputs should have at least one entry
    let has_inputs = !plan.inputs.is_empty();
    // capabilities should be present (even if all false)
    let has_capabilities = true; // capabilities is not Option
    // limits should be present
    let has_limits = plan.limits.max_repairs > 0;
    // steps should have at least one entry (from the pipeline)
    let has_steps = !plan.steps.is_empty();
    // outputs may be empty for tasks without output_schema
    
    if has_inputs && has_capabilities && has_limits && has_steps {
        TestResult::passed()
    } else {
        println!("Missing required fields in ExecutionPlan:");
        println!("  has_inputs: {}", has_inputs);
        println!("  has_capabilities: {}", has_capabilities);
        println!("  has_limits: {}", has_limits);
        println!("  has_steps: {}", has_steps);
        TestResult::failed()
    }
}

/// **Feature: intentscript-compiler, Property 33: Pipeline step lowering**
/// **Validates: Requirements 9.2**
///
/// For any pipeline with N steps, the lowered IR should contain N IRStep entries
/// with correct id, kind, args, produces, and checks fields.
#[quickcheck]
fn property_pipeline_step_lowering() -> TestResult {
    let mut g = Gen::new(10);
    let func1_name = gen_identifier(&mut g);
    let func2_name = gen_identifier(&mut g);
    let func3_name = gen_identifier(&mut g);
    
    // Create a pipeline with 3 steps
    let pipeline = Pipeline {
        steps: vec![
            Step::Call(CallExpr {
                name: func1_name.clone(),
                args: vec![Arg::Positional(Expr::Literal(Literal::Int(1), default_span()))],
                span: default_span(),
            }),
            Step::Call(CallExpr {
                name: func2_name.clone(),
                args: vec![Arg::Positional(Expr::Literal(Literal::Int(2), default_span()))],
                span: default_span(),
            }),
            Step::Call(CallExpr {
                name: func3_name.clone(),
                args: vec![Arg::Positional(Expr::Literal(Literal::Int(3), default_span()))],
                span: default_span(),
            }),
        ],
        span: default_span(),
    };
    
    let task = Task {
        name: "test_task".to_string(),
        version: None,
        sections: vec![Section::Run(pipeline)],
        span: default_span(),
    };
    
    let policy = Policy::new();
    let lowering = Lowering::new(policy);
    
    let plan = match lowering.lower_task(&task) {
        Ok(plan) => plan,
        Err(e) => {
            println!("Failed to lower task: {:?}", e);
            return TestResult::failed();
        }
    };
    
    // Should have 3 steps in the IR
    if plan.steps.len() != 3 {
        println!("Expected 3 steps, got {}", plan.steps.len());
        return TestResult::failed();
    }
    
    // Each step should have an id, kind, args, and produces
    for (i, step) in plan.steps.iter().enumerate() {
        if step.id.is_empty() {
            println!("Step {} has empty id", i);
            return TestResult::failed();
        }
        // produces should be Some for call steps
        if step.produces.is_none() {
            println!("Step {} has no produces field", i);
            return TestResult::failed();
        }
    }
    
    TestResult::passed()
}

/// **Feature: intentscript-compiler, Property 33: Pipeline step lowering with identifier steps**
/// **Validates: Requirements 9.2**
///
/// For any pipeline containing identifier steps, the lowered IR should correctly
/// represent them.
#[quickcheck]
fn property_pipeline_identifier_step_lowering() -> TestResult {
    let mut g = Gen::new(10);
    let builtin_steps = ["validate", "parse_openapi", "parse_markdown", "report"];
    let ident_name = builtin_steps[(gen_identifier(&mut g).len()) % builtin_steps.len()].to_string();

    let pipeline = Pipeline {
        steps: vec![Step::Ident(ident_name.clone(), default_span())],
        span: default_span(),
    };

    let task = Task {
        name: "test_task".to_string(),
        version: None,
        sections: vec![Section::Run(pipeline)],
        span: default_span(),
    };

    let policy = Policy::new();
    let lowering = Lowering::new(policy);

    let plan = match lowering.lower_task(&task) {
        Ok(plan) => plan,
        Err(e) => {
            println!("Failed to lower task: {:?}", e);
            return TestResult::failed();
        }
    };

    if plan.steps.len() != 1 {
        println!("Expected 1 step, got {}", plan.steps.len());
        return TestResult::failed();
    }

    let step = &plan.steps[0];
    let expected_kind = match ident_name.as_str() {
        "validate" => StepKind::Validate,
        "parse_openapi" => StepKind::ParseOpenApi,
        "parse_markdown" => StepKind::ParseMarkdown,
        "report" => StepKind::Report,
        _ => return TestResult::failed(),
    };

    if step.kind != expected_kind {
        println!("Expected kind {:?}, got {:?}", expected_kind, step.kind);
        return TestResult::failed();
    }

    if step.produces.as_deref() == Some("step_1_result") {
        TestResult::passed()
    } else {
        println!("Expected produces 'step_1_result', got {:?}", step.produces);
        TestResult::failed()
    }
}

/// **Feature: intentscript-compiler, Property 34: Constraint to capability translation**
/// **Validates: Requirements 9.3**
///
/// For any constraint set, the lowering process should correctly translate constraints
/// into the Capabilities structure (fs, net, exec, templates, exports).
#[quickcheck]
fn property_constraint_to_capability_translation() -> TestResult {
    let mut g = Gen::new(10);
    let task_name = gen_identifier(&mut g);
    
    // Create a task with various constraints
    let task = Task {
        name: task_name.clone(),
        version: None,
        sections: vec![
            Section::Constraints(vec![
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
                ConstraintDecl {
                    name: "templates".to_string(),
                    value: ConstraintValue::On,
                    span: default_span(),
                },
            ]),
        ],
        span: default_span(),
    };
    
    let policy = Policy::new();
    let lowering = Lowering::new(policy);
    
    let plan = match lowering.lower_task(&task) {
        Ok(plan) => plan,
        Err(e) => {
            println!("Failed to lower task: {:?}", e);
            return TestResult::failed();
        }
    };
    
    // Check that capabilities are correctly set
    if plan.capabilities.fs.is_none() {
        println!("Expected fs capability to be Some, got None");
        return TestResult::failed();
    }
    
    if !plan.capabilities.net {
        println!("Expected net capability to be true, got false");
        return TestResult::failed();
    }
    
    if !plan.capabilities.templates {
        println!("Expected templates capability to be true, got false");
        return TestResult::failed();
    }
    
    // exec and exports should be false (not set)
    if plan.capabilities.exec {
        println!("Expected exec capability to be false, got true");
        return TestResult::failed();
    }
    
    if plan.capabilities.exports {
        println!("Expected exports capability to be false, got true");
        return TestResult::failed();
    }
    
    TestResult::passed()
}

/// **Feature: intentscript-compiler, Property 34: Filesystem capability with paths**
/// **Validates: Requirements 9.3**
///
/// For any constraint specifying fs_read or fs_write paths, the lowering process
/// should correctly populate the FsCapability structure.
#[quickcheck]
fn property_fs_capability_with_paths() -> TestResult {
    let mut g = Gen::new(10);
    let task_name = gen_identifier(&mut g);
    let read_path = format!("/{}", gen_identifier(&mut g));
    let write_path = format!("/{}", gen_identifier(&mut g));
    
    // Create a task with fs path constraints
    let task = Task {
        name: task_name.clone(),
        version: None,
        sections: vec![
            Section::Constraints(vec![
                ConstraintDecl {
                    name: "fs_read".to_string(),
                    value: ConstraintValue::Literal(Literal::String(read_path.clone())),
                    span: default_span(),
                },
                ConstraintDecl {
                    name: "fs_write".to_string(),
                    value: ConstraintValue::Literal(Literal::String(write_path.clone())),
                    span: default_span(),
                },
            ]),
        ],
        span: default_span(),
    };
    
    let policy = Policy::new();
    let lowering = Lowering::new(policy);
    
    let plan = match lowering.lower_task(&task) {
        Ok(plan) => plan,
        Err(e) => {
            println!("Failed to lower task: {:?}", e);
            return TestResult::failed();
        }
    };
    
    // Check that fs capability is set with correct paths
    if let Some(fs_cap) = &plan.capabilities.fs {
        if !fs_cap.read_roots.contains(&read_path) {
            println!("Expected read_roots to contain '{}', got {:?}", read_path, fs_cap.read_roots);
            return TestResult::failed();
        }
        
        if !fs_cap.write_roots.contains(&write_path) {
            println!("Expected write_roots to contain '{}', got {:?}", write_path, fs_cap.write_roots);
            return TestResult::failed();
        }
        
        TestResult::passed()
    } else {
        println!("Expected fs capability to be Some, got None");
        TestResult::failed()
    }
}

/// **Feature: intentscript-compiler, Property 35: Check embedding in IR**
/// **Validates: Requirements 9.4**
///
/// For any check declaration, it should appear in the checks field of the appropriate
/// IRStep in the lowered IR.
#[quickcheck]
fn property_check_embedding() -> TestResult {
    let mut g = Gen::new(10);
    let task_name = gen_identifier(&mut g);
    let check_name = gen_identifier(&mut g);
    let func_name = gen_identifier(&mut g);
    
    // Create a task with checks
    let task = Task {
        name: task_name.clone(),
        version: None,
        sections: vec![
            Section::Checks(vec![
                CheckDecl {
                    name: check_name.clone(),
                    args: vec![Expr::Literal(Literal::String("test".to_string()), default_span())],
                    span: default_span(),
                },
            ]),
            Section::Run(Pipeline {
                steps: vec![
                    Step::Call(CallExpr {
                        name: func_name.clone(),
                        args: vec![],
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

    let plan = match lowering.lower_task(&task) {
        Ok(plan) => plan,
        Err(e) => {
            println!("Failed to lower task: {:?}", e);
            return TestResult::failed();
        }
    };

    let has_checks = plan.steps.iter().any(|step| !step.checks.is_empty());
    
    if !has_checks {
        println!("No checks found in any IR step");
        return TestResult::failed();
    }
    
    // Check that the check name appears in at least one step
    let has_check_name = plan.steps.iter().any(|step| {
        step.checks.iter().any(|check| check.name == check_name)
    });
    
    if has_check_name {
        TestResult::passed()
    } else {
        println!("Check '{}' not found in any IR step", check_name);
        TestResult::failed()
    }
}

/// **Feature: intentscript-compiler, Property 35: Multiple checks embedding**
/// **Validates: Requirements 9.4**
///
/// For any task with multiple check declarations, all checks should appear in the
/// lowered IR steps.
#[quickcheck]
fn property_multiple_checks_embedding() -> TestResult {
    let mut g = Gen::new(10);
    let task_name = gen_identifier(&mut g);
    let check1_name = gen_identifier(&mut g);
    let check2_name = gen_identifier(&mut g);
    let func_name = gen_identifier(&mut g);
    
    // Create a task with multiple checks
    let task = Task {
        name: task_name.clone(),
        version: None,
        sections: vec![
            Section::Checks(vec![
                CheckDecl {
                    name: check1_name.clone(),
                    args: vec![],
                    span: default_span(),
                },
                CheckDecl {
                    name: check2_name.clone(),
                    args: vec![],
                    span: default_span(),
                },
            ]),
            Section::Run(Pipeline {
                steps: vec![
                    Step::Call(CallExpr {
                        name: func_name.clone(),
                        args: vec![],
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

    let plan = match lowering.lower_task(&task) {
        Ok(plan) => plan,
        Err(e) => {
            println!("Failed to lower task: {:?}", e);
            return TestResult::failed();
        }
    };

    let total_checks: usize = plan.steps.iter().map(|step| step.checks.len()).sum();
    
    if total_checks >= 2 {
        TestResult::passed()
    } else {
        println!("Expected at least 2 checks in IR, found {}", total_checks);
        TestResult::failed()
    }
}

/// **Feature: intentscript-compiler, Property 36: IR JSON schema conformance**
/// **Validates: Requirements 9.5**
///
/// For any serialized ExecutionPlan, the JSON should be valid and conform to the
/// IR schema version 1.0 specification.
#[quickcheck]
fn property_ir_json_schema_conformance(prim_type: ArbitraryPrimitiveType) -> TestResult {
    let mut g = Gen::new(10);
    let task_name = gen_identifier(&mut g);
    let input_name = gen_identifier(&mut g);
    
    // Create a comprehensive task
    let task = Task {
        name: task_name.clone(),
        version: Some(Version {
            major: 1,
            minor: 2,
            patch: Some(3),
        }),
        sections: vec![
            Section::Input(vec![InputDecl {
                name: input_name.clone(),
                type_expr: TypeExpr::Primitive(prim_type.0, default_span()),
                default: None,
                span: default_span(),
            }]),
            Section::Constraints(vec![
                ConstraintDecl {
                    name: "fs".to_string(),
                    value: ConstraintValue::On,
                    span: default_span(),
                },
            ]),
            Section::Run(Pipeline {
                steps: vec![Step::Ident(input_name, default_span())],
                span: default_span(),
            }),
        ],
        span: default_span(),
    };
    
    let policy = Policy::new();
    let lowering = Lowering::new(policy);
    
    let plan = match lowering.lower_task(&task) {
        Ok(plan) => plan,
        Err(e) => {
            println!("Failed to lower task: {:?}", e);
            return TestResult::failed();
        }
    };
    
    // Serialize to JSON
    let json_str = match serde_json::to_string(&plan) {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to serialize plan to JSON: {:?}", e);
            return TestResult::failed();
        }
    };
    
    // Parse back to verify it's valid JSON
    let json_value: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(e) => {
            println!("Failed to parse JSON: {:?}", e);
            return TestResult::failed();
        }
    };
    
    // Verify required top-level fields exist
    if !json_value.is_object() {
        println!("JSON is not an object");
        return TestResult::failed();
    }
    
    let obj = json_value.as_object().unwrap();
    
    let required_fields = vec![
        "schema_version",
        "meta",
        "inputs",
        "capabilities",
        "limits",
        "steps",
        "outputs",
    ];
    
    for field in required_fields {
        if !obj.contains_key(field) {
            println!("Missing required field: {}", field);
            return TestResult::failed();
        }
    }
    
    // Verify schema_version is "1.0"
    if let Some(schema_version) = obj.get("schema_version") {
        if schema_version.as_str() != Some("1.0") {
            println!("Invalid schema_version: {:?}", schema_version);
            return TestResult::failed();
        }
    }
    
    // Verify meta has required fields
    if let Some(meta) = obj.get("meta") {
        if !meta.is_object() {
            println!("meta is not an object");
            return TestResult::failed();
        }
        
        let meta_obj = meta.as_object().unwrap();
        let meta_fields = vec!["task_name", "task_version", "compiler_version", "policy_hash"];
        
        for field in meta_fields {
            if !meta_obj.contains_key(field) {
                println!("Missing required meta field: {}", field);
                return TestResult::failed();
            }
        }
    }
    
    TestResult::passed()
}

/// **Feature: intentscript-compiler, Property 36: IR JSON round-trip conformance**
/// **Validates: Requirements 9.5**
///
/// For any ExecutionPlan, serializing to JSON and deserializing back should produce
/// an equivalent ExecutionPlan.
#[quickcheck]
fn property_ir_json_roundtrip_conformance(prim_type: ArbitraryPrimitiveType) -> TestResult {
    let mut g = Gen::new(10);
    let task_name = gen_identifier(&mut g);
    let input_name = gen_identifier(&mut g);
    
    // Create a task
    let task = Task {
        name: task_name.clone(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: None,
        }),
        sections: vec![
            Section::Input(vec![InputDecl {
                name: input_name.clone(),
                type_expr: TypeExpr::Primitive(prim_type.0, default_span()),
                default: None,
                span: default_span(),
            }]),
        ],
        span: default_span(),
    };
    
    let policy = Policy::new();
    let lowering = Lowering::new(policy);
    
    let plan = match lowering.lower_task(&task) {
        Ok(plan) => plan,
        Err(e) => {
            println!("Failed to lower task: {:?}", e);
            return TestResult::failed();
        }
    };
    
    // Serialize to JSON
    let json_str = match serde_json::to_string(&plan) {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to serialize plan: {:?}", e);
            return TestResult::failed();
        }
    };
    
    // Deserialize back
    let deserialized_plan: intentscript_compiler::ExecutionPlan = match serde_json::from_str(&json_str) {
        Ok(p) => p,
        Err(e) => {
            println!("Failed to deserialize plan: {:?}", e);
            return TestResult::failed();
        }
    };
    
    // Verify round-trip equality
    if plan == deserialized_plan {
        TestResult::passed()
    } else {
        println!("Round-trip failed: plans are not equal");
        TestResult::failed()
    }
}
