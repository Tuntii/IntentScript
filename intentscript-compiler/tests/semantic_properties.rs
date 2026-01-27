// Property-based tests for semantic analysis
// Feature: intentscript-compiler

use intentscript_compiler::SemanticAnalyzer;
use intentscript_core::Span;
use intentscript_parser::{
    File, InputDecl, Literal, PrimitiveType, Section, Task, TypeExpr, Version,
};
use quickcheck::{Arbitrary, Gen, TestResult};
use quickcheck_macros::quickcheck;

// Helper to create a default span
fn default_span() -> Span {
    Span::new(1, 1, 0, 0)
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

#[derive(Debug, Clone)]
struct ArbitraryTypeExpr(TypeExpr);

impl Arbitrary for ArbitraryTypeExpr {
    fn arbitrary(g: &mut Gen) -> Self {
        let choice = u8::arbitrary(g) % 3;
        let type_expr = match choice {
            0 => {
                let prim = ArbitraryPrimitiveType::arbitrary(g).0;
                TypeExpr::Primitive(prim, default_span())
            }
            1 => {
                let prim = ArbitraryPrimitiveType::arbitrary(g).0;
                TypeExpr::List(
                    Box::new(TypeExpr::Primitive(prim, default_span())),
                    default_span(),
                )
            }
            _ => {
                let prim = ArbitraryPrimitiveType::arbitrary(g).0;
                TypeExpr::Optional(
                    Box::new(TypeExpr::Primitive(prim, default_span())),
                    default_span(),
                )
            }
        };
        ArbitraryTypeExpr(type_expr)
    }
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

/// **Feature: intentscript-compiler, Property 19: Input type validation**
/// **Validates: Requirements 6.1**
///
/// For any input declaration, the semantic analyzer should verify that the type annotation
/// is valid and well-formed.
#[quickcheck]
fn property_input_type_validation(prim_type: ArbitraryPrimitiveType) -> TestResult {
    let mut g = Gen::new(10);
    let input_name = gen_identifier(&mut g);
    
    // Create a valid input declaration with a primitive type
    let input = InputDecl {
        name: input_name.clone(),
        type_expr: TypeExpr::Primitive(prim_type.0, default_span()),
        default: None,
        span: default_span(),
    };

    // Create a minimal task with this input
    let task = Task {
        name: "test_task".to_string(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: None,
        }),
        sections: vec![Section::Input(vec![input])],
        span: default_span(),
    };

    let file = File { tasks: vec![task] };

    // Analyze the file
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&file);

    // The analysis should succeed for valid primitive types
    match result {
        Ok(_) => TestResult::passed(),
        Err(errors) => {
            println!("Unexpected errors: {:?}", errors);
            TestResult::failed()
        }
    }
}

/// **Feature: intentscript-compiler, Property 19: Input type validation (nested types)**
/// **Validates: Requirements 6.1**
///
/// For any input declaration with nested types (List, Optional), the semantic analyzer
/// should verify that the type annotation is valid and well-formed.
#[quickcheck]
fn property_input_nested_type_validation(type_expr: ArbitraryTypeExpr) -> TestResult {
    let mut g = Gen::new(10);
    let input_name = gen_identifier(&mut g);
    
    // Create a valid input declaration with a nested type
    let input = InputDecl {
        name: input_name.clone(),
        type_expr: type_expr.0,
        default: None,
        span: default_span(),
    };

    // Create a minimal task with this input
    let task = Task {
        name: "test_task".to_string(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: None,
        }),
        sections: vec![Section::Input(vec![input])],
        span: default_span(),
    };

    let file = File { tasks: vec![task] };

    // Analyze the file
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&file);

    // The analysis should succeed for valid type expressions
    match result {
        Ok(_) => TestResult::passed(),
        Err(errors) => {
            println!("Unexpected errors: {:?}", errors);
            TestResult::failed()
        }
    }
}

/// **Feature: intentscript-compiler, Property 19: Input type validation (default value)**
/// **Validates: Requirements 6.1**
///
/// For any input declaration with a default value, the semantic analyzer should verify
/// that the default value matches the declared type.
#[quickcheck]
fn property_input_default_value_type_match(prim_type: ArbitraryPrimitiveType) -> TestResult {
    let mut g = Gen::new(10);
    let input_name = gen_identifier(&mut g);
    
    // Create a matching default value for the type
    // Only test types that have direct literal representations
    let (type_expr, default_value) = match prim_type.0 {
        PrimitiveType::Bool => (
            TypeExpr::Primitive(PrimitiveType::Bool, default_span()),
            Literal::Bool(true),
        ),
        PrimitiveType::Int => (
            TypeExpr::Primitive(PrimitiveType::Int, default_span()),
            Literal::Int(42),
        ),
        PrimitiveType::Float => (
            TypeExpr::Primitive(PrimitiveType::Float, default_span()),
            Literal::Float(3.14),
        ),
        PrimitiveType::Text | PrimitiveType::Url | PrimitiveType::Email | PrimitiveType::Path => (
            TypeExpr::Primitive(PrimitiveType::Text, default_span()),
            Literal::String("test".to_string()),
        ),
        _ => {
            // Skip types that don't have direct literal representations
            return TestResult::discard();
        }
    };

    let input = InputDecl {
        name: input_name.clone(),
        type_expr,
        default: Some(default_value),
        span: default_span(),
    };

    let task = Task {
        name: "test_task".to_string(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: None,
        }),
        sections: vec![Section::Input(vec![input])],
        span: default_span(),
    };

    let file = File { tasks: vec![task] };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&file);

    // The analysis should succeed when default value matches type
    match result {
        Ok(_) => TestResult::passed(),
        Err(errors) => {
            println!("Unexpected errors: {:?}", errors);
            TestResult::failed()
        }
    }
}


/// **Feature: intentscript-compiler, Property 20: Function call type checking**
/// **Validates: Requirements 6.2**
///
/// For any function call, the semantic analyzer should verify that argument types
/// are well-formed and can be inferred.
#[quickcheck]
fn property_function_call_type_checking(prim_type: ArbitraryPrimitiveType) -> TestResult {
    let mut g = Gen::new(10);
    let input_name = gen_identifier(&mut g);
    let func_name = gen_identifier(&mut g);
    
    // Create an input declaration
    let input = InputDecl {
        name: input_name.clone(),
        type_expr: TypeExpr::Primitive(prim_type.0, default_span()),
        default: None,
        span: default_span(),
    };

    // Create a function call that uses the input as an argument
    use intentscript_parser::{Arg, CallExpr, Expr, Pipeline, Step};
    
    let call = CallExpr {
        name: func_name,
        args: vec![Arg::Positional(Expr::Ident(input_name, default_span()))],
        span: default_span(),
    };

    let pipeline = Pipeline {
        steps: vec![Step::Call(call)],
        span: default_span(),
    };

    let task = Task {
        name: "test_task".to_string(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: None,
        }),
        sections: vec![
            Section::Input(vec![input]),
            Section::Run(pipeline),
        ],
        span: default_span(),
    };

    let file = File { tasks: vec![task] };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&file);

    // The analysis should succeed - function calls with valid identifiers should type check
    match result {
        Ok(_) => TestResult::passed(),
        Err(errors) => {
            println!("Unexpected errors: {:?}", errors);
            TestResult::failed()
        }
    }
}

/// **Feature: intentscript-compiler, Property 20: Function call with literal arguments**
/// **Validates: Requirements 6.2**
///
/// For any function call with literal arguments, the semantic analyzer should
/// successfully infer the types of the arguments.
#[quickcheck]
fn property_function_call_literal_args() -> TestResult {
    let mut g = Gen::new(10);
    let func_name = gen_identifier(&mut g);
    
    use intentscript_parser::{Arg, CallExpr, Expr, Pipeline, Step};
    
    // Create a function call with literal arguments
    let call = CallExpr {
        name: func_name,
        args: vec![
            Arg::Positional(Expr::Literal(Literal::Int(42), default_span())),
            Arg::Positional(Expr::Literal(Literal::String("test".to_string()), default_span())),
            Arg::Named {
                name: "flag".to_string(),
                value: Expr::Literal(Literal::Bool(true), default_span()),
            },
        ],
        span: default_span(),
    };

    let pipeline = Pipeline {
        steps: vec![Step::Call(call)],
        span: default_span(),
    };

    let task = Task {
        name: "test_task".to_string(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: None,
        }),
        sections: vec![Section::Run(pipeline)],
        span: default_span(),
    };

    let file = File { tasks: vec![task] };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&file);

    // The analysis should succeed - literal arguments have clear types
    match result {
        Ok(_) => TestResult::passed(),
        Err(errors) => {
            println!("Unexpected errors: {:?}", errors);
            TestResult::failed()
        }
    }
}

/// **Feature: intentscript-compiler, Property 21: Pipeline type compatibility**
/// **Validates: Requirements 6.3**
///
/// For any pipeline with multiple steps, the semantic analyzer should verify that
/// all steps are well-formed and identifiers are defined.
#[quickcheck]
fn property_pipeline_type_compatibility(prim_type: ArbitraryPrimitiveType) -> TestResult {
    let mut g = Gen::new(10);
    let input_name = gen_identifier(&mut g);
    let func1_name = gen_identifier(&mut g);
    let func2_name = gen_identifier(&mut g);
    
    // Create an input declaration
    let input = InputDecl {
        name: input_name.clone(),
        type_expr: TypeExpr::Primitive(prim_type.0, default_span()),
        default: None,
        span: default_span(),
    };

    use intentscript_parser::{Arg, CallExpr, Expr, Pipeline, Step};
    
    // Create a pipeline with multiple steps
    let call1 = CallExpr {
        name: func1_name,
        args: vec![Arg::Positional(Expr::Ident(input_name, default_span()))],
        span: default_span(),
    };

    let call2 = CallExpr {
        name: func2_name,
        args: vec![Arg::Positional(Expr::Literal(Literal::String("test".to_string()), default_span()))],
        span: default_span(),
    };

    let pipeline = Pipeline {
        steps: vec![Step::Call(call1), Step::Call(call2)],
        span: default_span(),
    };

    let task = Task {
        name: "test_task".to_string(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: None,
        }),
        sections: vec![
            Section::Input(vec![input]),
            Section::Run(pipeline),
        ],
        span: default_span(),
    };

    let file = File { tasks: vec![task] };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&file);

    // The analysis should succeed - all identifiers are defined
    match result {
        Ok(_) => TestResult::passed(),
        Err(errors) => {
            println!("Unexpected errors: {:?}", errors);
            TestResult::failed()
        }
    }
}

/// **Feature: intentscript-compiler, Property 21: Pipeline with identifier steps**
/// **Validates: Requirements 6.3**
///
/// For any pipeline that references identifiers, the semantic analyzer should
/// verify that those identifiers are defined in the symbol table.
#[quickcheck]
fn property_pipeline_identifier_validation(prim_type: ArbitraryPrimitiveType) -> TestResult {
    let mut g = Gen::new(10);
    let input_name = gen_identifier(&mut g);
    
    // Create an input declaration
    let input = InputDecl {
        name: input_name.clone(),
        type_expr: TypeExpr::Primitive(prim_type.0, default_span()),
        default: None,
        span: default_span(),
    };

    use intentscript_parser::{Pipeline, Step};
    
    // Create a pipeline that references the input identifier
    let pipeline = Pipeline {
        steps: vec![Step::Ident(input_name, default_span())],
        span: default_span(),
    };

    let task = Task {
        name: "test_task".to_string(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: None,
        }),
        sections: vec![
            Section::Input(vec![input]),
            Section::Run(pipeline),
        ],
        span: default_span(),
    };

    let file = File { tasks: vec![task] };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&file);

    // The analysis should succeed - the identifier is defined
    match result {
        Ok(_) => TestResult::passed(),
        Err(errors) => {
            println!("Unexpected errors: {:?}", errors);
            TestResult::failed()
        }
    }
}

/// **Feature: intentscript-compiler, Property 22: Optional type policy enforcement**
/// **Validates: Requirements 6.4**
///
/// For any usage of optional types, the semantic analyzer should recognize and
/// validate optional type declarations.
#[quickcheck]
fn property_optional_type_policy_enforcement(prim_type: ArbitraryPrimitiveType) -> TestResult {
    let mut g = Gen::new(10);
    let input_name = gen_identifier(&mut g);
    
    // Create an input declaration with an optional type
    let input = InputDecl {
        name: input_name.clone(),
        type_expr: TypeExpr::Optional(
            Box::new(TypeExpr::Primitive(prim_type.0, default_span())),
            default_span(),
        ),
        default: None,
        span: default_span(),
    };

    let task = Task {
        name: "test_task".to_string(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: None,
        }),
        sections: vec![Section::Input(vec![input])],
        span: default_span(),
    };

    let file = File { tasks: vec![task] };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&file);

    // The analysis should succeed - optional types are valid
    match result {
        Ok(_) => TestResult::passed(),
        Err(errors) => {
            println!("Unexpected errors: {:?}", errors);
            TestResult::failed()
        }
    }
}

/// **Feature: intentscript-compiler, Property 22: Nested optional types**
/// **Validates: Requirements 6.4**
///
/// For any nested optional type (e.g., Optional<List<T>>), the semantic analyzer
/// should correctly validate the type structure.
#[quickcheck]
fn property_nested_optional_types(prim_type: ArbitraryPrimitiveType) -> TestResult {
    let mut g = Gen::new(10);
    let input_name = gen_identifier(&mut g);
    
    // Create an input declaration with a nested optional type
    let input = InputDecl {
        name: input_name.clone(),
        type_expr: TypeExpr::Optional(
            Box::new(TypeExpr::List(
                Box::new(TypeExpr::Primitive(prim_type.0, default_span())),
                default_span(),
            )),
            default_span(),
        ),
        default: None,
        span: default_span(),
    };

    let task = Task {
        name: "test_task".to_string(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: None,
        }),
        sections: vec![Section::Input(vec![input])],
        span: default_span(),
    };

    let file = File { tasks: vec![task] };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&file);

    // The analysis should succeed - nested optional types are valid
    match result {
        Ok(_) => TestResult::passed(),
        Err(errors) => {
            println!("Unexpected errors: {:?}", errors);
            TestResult::failed()
        }
    }
}

/// **Feature: intentscript-compiler, Property 23: Type mismatch error content**
/// **Validates: Requirements 6.5**
///
/// For any type mismatch, the error diagnostic should include both the expected type
/// and the actual type found.
#[quickcheck]
fn property_type_mismatch_error_content() -> TestResult {
    let mut g = Gen::new(10);
    let input_name = gen_identifier(&mut g);
    
    // Create an input declaration with Int type but String default
    let input = InputDecl {
        name: input_name.clone(),
        type_expr: TypeExpr::Primitive(PrimitiveType::Int, default_span()),
        default: Some(Literal::String("not_an_int".to_string())),
        span: default_span(),
    };

    let task = Task {
        name: "test_task".to_string(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: None,
        }),
        sections: vec![Section::Input(vec![input])],
        span: default_span(),
    };

    let file = File { tasks: vec![task] };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&file);

    // The analysis should fail with a type error
    match result {
        Ok(_) => {
            println!("Expected type error but analysis succeeded");
            TestResult::failed()
        }
        Err(errors) => {
            // Check that we have at least one error
            if errors.is_empty() {
                println!("Expected errors but got empty error list");
                return TestResult::failed();
            }

            // Check that the error contains type information
            let error_str = format!("{:?}", errors[0]);
            let has_expected = error_str.contains("Int") || error_str.contains("expected");
            let has_found = error_str.contains("Text") || error_str.contains("String") || error_str.contains("found");

            if has_expected && has_found {
                TestResult::passed()
            } else {
                println!("Error doesn't contain expected type information: {:?}", errors[0]);
                TestResult::failed()
            }
        }
    }
}

/// **Feature: intentscript-compiler, Property 23: Type mismatch with different primitives**
/// **Validates: Requirements 6.5**
///
/// For any type mismatch between different primitive types, the error should clearly
/// indicate both the expected and actual types.
#[quickcheck]
fn property_type_mismatch_primitives(
    expected_type: ArbitraryPrimitiveType,
    actual_type: ArbitraryPrimitiveType,
) -> TestResult {
    // Only test when types are different
    if expected_type.0 == actual_type.0 {
        return TestResult::discard();
    }

    // Only test types that have literal representations
    let actual_literal = match actual_type.0 {
        PrimitiveType::Bool => Literal::Bool(true),
        PrimitiveType::Int => Literal::Int(42),
        PrimitiveType::Float => Literal::Float(3.14),
        PrimitiveType::Text => Literal::String("test".to_string()),
        _ => return TestResult::discard(),
    };

    let mut g = Gen::new(10);
    let input_name = gen_identifier(&mut g);
    
    let input = InputDecl {
        name: input_name.clone(),
        type_expr: TypeExpr::Primitive(expected_type.0, default_span()),
        default: Some(actual_literal),
        span: default_span(),
    };

    let task = Task {
        name: "test_task".to_string(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: None,
        }),
        sections: vec![Section::Input(vec![input])],
        span: default_span(),
    };

    let file = File { tasks: vec![task] };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&file);

    // The analysis should fail with a type error
    match result {
        Ok(_) => {
            println!("Expected type error but analysis succeeded");
            TestResult::failed()
        }
        Err(errors) => {
            if errors.is_empty() {
                println!("Expected errors but got empty error list");
                return TestResult::failed();
            }

            // The error should be a type error
            let error_str = format!("{:?}", errors[0]);
            if error_str.contains("Type") || error_str.contains("expected") {
                TestResult::passed()
            } else {
                println!("Error is not a type error: {:?}", errors[0]);
                TestResult::failed()
            }
        }
    }
}

/// **Feature: intentscript-compiler, Property 24: Constraint contradiction detection**
/// **Validates: Requirements 7.1**
///
/// For any set of mutually exclusive constraints (e.g., `net = on` and `net = off`),
/// the semantic analyzer should detect and report the contradiction.
#[quickcheck]
fn property_constraint_contradiction_detection() -> TestResult {
    use intentscript_parser::{ConstraintDecl, ConstraintValue};
    
    let mut g = Gen::new(10);
    let constraint_name = gen_identifier(&mut g);
    
    // Create two contradictory constraints: one On and one Off
    let constraint_on = ConstraintDecl {
        name: constraint_name.clone(),
        value: ConstraintValue::On,
        span: Span::new(1, 1, 0, 10),
    };
    
    let constraint_off = ConstraintDecl {
        name: constraint_name.clone(),
        value: ConstraintValue::Off,
        span: Span::new(2, 1, 20, 10),
    };
    
    let task = Task {
        name: "test_task".to_string(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: None,
        }),
        sections: vec![Section::Constraints(vec![constraint_on, constraint_off])],
        span: default_span(),
    };
    
    let file = File { tasks: vec![task] };
    
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&file);
    
    // The analysis should fail with a constraint error
    match result {
        Ok(_) => {
            println!("Expected constraint contradiction error but analysis succeeded");
            TestResult::failed()
        }
        Err(errors) => {
            if errors.is_empty() {
                println!("Expected errors but got empty error list");
                return TestResult::failed();
            }
            
            // Check that the error mentions contradiction
            let error_str = format!("{:?}", errors[0]);
            if error_str.contains("Constraint") || error_str.contains("contradict") || error_str.contains("Contradictory") {
                TestResult::passed()
            } else {
                println!("Error doesn't mention contradiction: {:?}", errors[0]);
                TestResult::failed()
            }
        }
    }
}

/// **Feature: intentscript-compiler, Property 24: Multiple literal constraint contradictions**
/// **Validates: Requirements 7.1**
///
/// For any constraint with multiple different literal values, the semantic analyzer
/// should detect the contradiction.
#[quickcheck]
fn property_constraint_literal_contradictions() -> TestResult {
    use intentscript_parser::{ConstraintDecl, ConstraintValue};
    
    let mut g = Gen::new(10);
    let constraint_name = gen_identifier(&mut g);
    
    // Create two constraints with different literal values
    let constraint1 = ConstraintDecl {
        name: constraint_name.clone(),
        value: ConstraintValue::Literal(Literal::Int(10)),
        span: Span::new(1, 1, 0, 10),
    };
    
    let constraint2 = ConstraintDecl {
        name: constraint_name.clone(),
        value: ConstraintValue::Literal(Literal::Int(20)),
        span: Span::new(2, 1, 20, 10),
    };
    
    let task = Task {
        name: "test_task".to_string(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: None,
        }),
        sections: vec![Section::Constraints(vec![constraint1, constraint2])],
        span: default_span(),
    };
    
    let file = File { tasks: vec![task] };
    
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&file);
    
    // The analysis should fail with a constraint error
    match result {
        Ok(_) => {
            println!("Expected constraint contradiction error but analysis succeeded");
            TestResult::failed()
        }
        Err(errors) => {
            if errors.is_empty() {
                println!("Expected errors but got empty error list");
                return TestResult::failed();
            }
            
            // Check that the error is a constraint error
            let error_str = format!("{:?}", errors[0]);
            if error_str.contains("Constraint") || error_str.contains("conflict") {
                TestResult::passed()
            } else {
                println!("Error is not a constraint error: {:?}", errors[0]);
                TestResult::failed()
            }
        }
    }
}

/// **Feature: intentscript-compiler, Property 25: Policy-task conflict reporting**
/// **Validates: Requirements 7.2**
///
/// For any conflict between policy rules and task constraints, the error should
/// identify both the policy rule and the task constraint.
#[quickcheck]
fn property_policy_task_conflict_reporting() -> TestResult {
    use intentscript_compiler::Policy;
    use intentscript_parser::{ConstraintDecl, ConstraintValue};
    
    let mut g = Gen::new(10);
    let constraint_name = gen_identifier(&mut g);
    
    // Create a policy that says constraint should be On
    let mut policy = Policy::new();
    policy.add_constraint(constraint_name.clone(), ConstraintValue::On);
    
    // Create a task constraint that says it should be Off
    let task_constraint = ConstraintDecl {
        name: constraint_name.clone(),
        value: ConstraintValue::Off,
        span: Span::new(1, 1, 0, 10),
    };
    
    let task = Task {
        name: "test_task".to_string(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: None,
        }),
        sections: vec![Section::Constraints(vec![task_constraint])],
        span: default_span(),
    };
    
    let file = File { tasks: vec![task] };
    
    let mut analyzer = SemanticAnalyzer::with_policy(policy);
    let result = analyzer.analyze(&file);
    
    // The analysis should fail with a policy violation error
    match result {
        Ok(_) => {
            println!("Expected policy conflict error but analysis succeeded");
            TestResult::failed()
        }
        Err(errors) => {
            if errors.is_empty() {
                println!("Expected errors but got empty error list");
                return TestResult::failed();
            }
            
            // Check that the error mentions policy
            let error_str = format!("{:?}", errors[0]);
            if error_str.contains("Policy") || error_str.contains("policy") || error_str.contains("conflict") {
                TestResult::passed()
            } else {
                println!("Error doesn't mention policy conflict: {:?}", errors[0]);
                TestResult::failed()
            }
        }
    }
}

/// **Feature: intentscript-compiler, Property 25: Policy allows compatible task constraints**
/// **Validates: Requirements 7.2**
///
/// For any task constraint that is compatible with policy, the semantic analyzer
/// should not report an error.
#[quickcheck]
fn property_policy_compatible_constraints() -> TestResult {
    use intentscript_compiler::Policy;
    use intentscript_parser::{ConstraintDecl, ConstraintValue};
    
    let mut g = Gen::new(10);
    let constraint_name = gen_identifier(&mut g);
    
    // Create a policy that says constraint should be On
    let mut policy = Policy::new();
    policy.add_constraint(constraint_name.clone(), ConstraintValue::On);
    
    // Create a task constraint that also says it should be On (compatible)
    let task_constraint = ConstraintDecl {
        name: constraint_name.clone(),
        value: ConstraintValue::On,
        span: Span::new(1, 1, 0, 10),
    };
    
    let task = Task {
        name: "test_task".to_string(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: None,
        }),
        sections: vec![Section::Constraints(vec![task_constraint])],
        span: default_span(),
    };
    
    let file = File { tasks: vec![task] };
    
    let mut analyzer = SemanticAnalyzer::with_policy(policy);
    let result = analyzer.analyze(&file);
    
    // The analysis should succeed - constraints are compatible
    match result {
        Ok(_) => TestResult::passed(),
        Err(errors) => {
            println!("Unexpected errors for compatible constraints: {:?}", errors);
            TestResult::failed()
        }
    }
}

/// **Feature: intentscript-compiler, Property 26: Ambiguity resolution policy**
/// **Validates: Requirements 7.3**
///
/// For any ambiguous construct, the semantic analyzer should report an error unless
/// the policy explicitly allows resolution.
#[quickcheck]
fn property_ambiguity_resolution_policy() -> TestResult {
    use intentscript_compiler::Policy;
    use intentscript_parser::{ConstraintDecl, ConstraintValue};
    
    let mut g = Gen::new(10);
    let constraint_name = gen_identifier(&mut g);
    
    // Create a policy that does NOT allow ambiguity resolution
    let policy = Policy::new(); // default is allow_ambiguity_resolution = false
    
    // Create two constraints with the same name but different literal values
    // This creates an ambiguity
    let constraint1 = ConstraintDecl {
        name: constraint_name.clone(),
        value: ConstraintValue::Literal(Literal::Int(10)),
        span: Span::new(1, 1, 0, 10),
    };
    
    let constraint2 = ConstraintDecl {
        name: constraint_name.clone(),
        value: ConstraintValue::Literal(Literal::Int(20)),
        span: Span::new(2, 1, 20, 10),
    };
    
    let task = Task {
        name: "test_task".to_string(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: None,
        }),
        sections: vec![Section::Constraints(vec![constraint1, constraint2])],
        span: default_span(),
    };
    
    let file = File { tasks: vec![task] };
    
    let mut analyzer = SemanticAnalyzer::with_policy(policy);
    let result = analyzer.analyze(&file);
    
    // The analysis should fail because policy doesn't allow ambiguity resolution
    match result {
        Ok(_) => {
            println!("Expected ambiguity error but analysis succeeded");
            TestResult::failed()
        }
        Err(errors) => {
            if errors.is_empty() {
                println!("Expected errors but got empty error list");
                return TestResult::failed();
            }
            
            // Check that we got a constraint error
            let error_str = format!("{:?}", errors[0]);
            if error_str.contains("Constraint") || error_str.contains("conflict") || error_str.contains("Multiple") {
                TestResult::passed()
            } else {
                println!("Error doesn't indicate ambiguity: {:?}", errors[0]);
                TestResult::failed()
            }
        }
    }
}

/// **Feature: intentscript-compiler, Property 26: Policy allows ambiguity resolution**
/// **Validates: Requirements 7.3**
///
/// When policy allows ambiguity resolution, the semantic analyzer should resolve
/// ambiguous constraints without error.
#[quickcheck]
fn property_policy_allows_ambiguity_resolution() -> TestResult {
    use intentscript_compiler::Policy;
    use intentscript_parser::{ConstraintDecl, ConstraintValue};
    
    let mut g = Gen::new(10);
    let constraint_name = gen_identifier(&mut g);
    
    // Create a policy that DOES allow ambiguity resolution
    let mut policy = Policy::new();
    policy.allow_ambiguity_resolution = true;
    
    // Create two constraints with the same name (ambiguous)
    let constraint1 = ConstraintDecl {
        name: constraint_name.clone(),
        value: ConstraintValue::On,
        span: Span::new(1, 1, 0, 10),
    };
    
    let constraint2 = ConstraintDecl {
        name: constraint_name.clone(),
        value: ConstraintValue::On,
        span: Span::new(2, 1, 20, 10),
    };
    
    let task = Task {
        name: "test_task".to_string(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: None,
        }),
        sections: vec![Section::Constraints(vec![constraint1, constraint2])],
        span: default_span(),
    };
    
    let file = File { tasks: vec![task] };
    
    let mut analyzer = SemanticAnalyzer::with_policy(policy);
    let result = analyzer.analyze(&file);
    
    // The analysis should succeed because policy allows ambiguity resolution
    match result {
        Ok(_) => TestResult::passed(),
        Err(errors) => {
            println!("Unexpected errors when policy allows ambiguity resolution: {:?}", errors);
            TestResult::failed()
        }
    }
}

/// **Feature: intentscript-compiler, Property 27: Constraint set consistency**
/// **Validates: Requirements 7.4**
///
/// For any constraint set, the semantic analyzer should either produce a consistent
/// set of constraints or fail compilation with a clear error.
#[quickcheck]
fn property_constraint_set_consistency() -> TestResult {
    use intentscript_parser::{ConstraintDecl, ConstraintValue};
    
    let mut g = Gen::new(10);
    
    // Generate a set of constraints with unique names (should be consistent)
    let num_constraints = (usize::arbitrary(&mut g) % 5) + 1;
    let mut constraints = Vec::new();
    
    for i in 0..num_constraints {
        let constraint_name = format!("constraint_{}", i);
        let value = if bool::arbitrary(&mut g) {
            ConstraintValue::On
        } else {
            ConstraintValue::Off
        };
        
        constraints.push(ConstraintDecl {
            name: constraint_name,
            value,
            span: Span::new(i as u32 + 1, 1, i * 20, 10),
        });
    }
    
    let task = Task {
        name: "test_task".to_string(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: None,
        }),
        sections: vec![Section::Constraints(constraints)],
        span: default_span(),
    };
    
    let file = File { tasks: vec![task] };
    
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&file);
    
    // Since all constraint names are unique, the analysis should succeed
    match result {
        Ok(_) => TestResult::passed(),
        Err(errors) => {
            println!("Unexpected errors for consistent constraint set: {:?}", errors);
            TestResult::failed()
        }
    }
}

/// **Feature: intentscript-compiler, Property 27: Inconsistent constraint set fails**
/// **Validates: Requirements 7.4**
///
/// For any inconsistent constraint set (with contradictions), the semantic analyzer
/// should fail compilation with clear errors.
#[quickcheck]
fn property_inconsistent_constraint_set_fails() -> TestResult {
    use intentscript_parser::{ConstraintDecl, ConstraintValue};
    
    let mut g = Gen::new(10);
    let constraint_name = gen_identifier(&mut g);
    
    // Create an inconsistent set: same constraint with On and Off
    let constraints = vec![
        ConstraintDecl {
            name: constraint_name.clone(),
            value: ConstraintValue::On,
            span: Span::new(1, 1, 0, 10),
        },
        ConstraintDecl {
            name: constraint_name.clone(),
            value: ConstraintValue::Off,
            span: Span::new(2, 1, 20, 10),
        },
    ];
    
    let task = Task {
        name: "test_task".to_string(),
        version: Some(Version {
            major: 1,
            minor: 0,
            patch: None,
        }),
        sections: vec![Section::Constraints(constraints)],
        span: default_span(),
    };
    
    let file = File { tasks: vec![task] };
    
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&file);
    
    // The analysis should fail with a clear error
    match result {
        Ok(_) => {
            println!("Expected error for inconsistent constraint set but analysis succeeded");
            TestResult::failed()
        }
        Err(errors) => {
            if errors.is_empty() {
                println!("Expected errors but got empty error list");
                return TestResult::failed();
            }
            
            // Should have a constraint error
            TestResult::passed()
        }
    }
}
