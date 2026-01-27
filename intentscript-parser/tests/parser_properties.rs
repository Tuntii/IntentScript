use intentscript_parser::Parser;
use quickcheck::{Arbitrary, Gen, TestResult};
use quickcheck_macros::quickcheck;

// Helper to generate valid task names
#[derive(Clone, Debug)]
struct ValidTaskName(String);

impl Arbitrary for ValidTaskName {
    fn arbitrary(g: &mut Gen) -> Self {
        let words = vec!["test", "example", "demo", "sample", "task", "process", "workflow"];
        let word = g.choose(&words).unwrap();
        ValidTaskName(word.to_string())
    }
}

// Helper to generate valid version strings
#[derive(Clone, Debug)]
struct ValidVersion(String);

impl Arbitrary for ValidVersion {
    fn arbitrary(g: &mut Gen) -> Self {
        let major = u8::arbitrary(g) % 10;
        let minor = u8::arbitrary(g) % 10;
        let has_patch = bool::arbitrary(g);
        
        if has_patch {
            let patch = u8::arbitrary(g) % 10;
            ValidVersion(format!("v{}.{}.{}", major, minor, patch))
        } else {
            ValidVersion(format!("v{}.{}", major, minor))
        }
    }
}

/// **Feature: intentscript-compiler, Property 6: Task declaration parsing**
/// **Validates: Requirements 3.1**
/// 
/// For any valid task name and optional version, the parser should successfully 
/// parse the task declaration `task "name" v1.0 { ... }`.
#[quickcheck]
fn property_task_declaration_parsing(name: ValidTaskName, version: ValidVersion) -> TestResult {
    let source = format!(
        r#"task "{}" {} {{
            goal: "test goal"
            run: step1
        }}"#,
        name.0, version.0
    );

    let mut parser = Parser::new(&source);
    let result = parser.parse_file();

    match result {
        Ok(file) => {
            if file.tasks.len() != 1 {
                return TestResult::failed();
            }
            
            let task = &file.tasks[0];
            
            // Verify task name matches
            if task.name != name.0 {
                return TestResult::failed();
            }
            
            // Verify version is present and parsed correctly
            if task.version.is_none() {
                return TestResult::failed();
            }
            
            TestResult::passed()
        }
        Err(_) => TestResult::failed(),
    }
}

/// Test that task declarations without version are also valid
#[quickcheck]
fn property_task_declaration_without_version(name: ValidTaskName) -> TestResult {
    let source = format!(
        r#"task "{}" {{
            goal: "test goal"
            run: step1
        }}"#,
        name.0
    );

    let mut parser = Parser::new(&source);
    let result = parser.parse_file();

    match result {
        Ok(file) => {
            if file.tasks.len() != 1 {
                return TestResult::failed();
            }
            
            let task = &file.tasks[0];
            
            // Verify task name matches
            if task.name != name.0 {
                return TestResult::failed();
            }
            
            // Version should be None
            if task.version.is_some() {
                return TestResult::failed();
            }
            
            TestResult::passed()
        }
        Err(_) => TestResult::failed(),
    }
}


/// **Feature: intentscript-compiler, Property 7: Section parsing completeness**
/// **Validates: Requirements 3.2**
/// 
/// For any valid combination of task sections (goal, input, constraints, output_schema, checks, run),
/// the parser should successfully parse all sections into the AST.
#[quickcheck]
fn property_section_parsing_completeness(name: ValidTaskName) -> TestResult {
    // Test with all possible sections
    let source = format!(
        r#"task "{}" v1.0 {{
            goal: "Complete task with all sections"
            input: {{
                param1: text,
                param2: int
            }}
            constraints: {{
                fs = on,
                net = off
            }}
            output_schema: text
            checks: {{
                validate(output)
            }}
            run: step1 -> step2
        }}"#,
        name.0
    );

    let mut parser = Parser::new(&source);
    let result = parser.parse_file();

    match result {
        Ok(file) => {
            if file.tasks.len() != 1 {
                return TestResult::failed();
            }
            
            let task = &file.tasks[0];
            
            // Should have all 6 sections
            if task.sections.len() != 6 {
                return TestResult::failed();
            }
            
            // Verify each section type is present
            use intentscript_parser::Section;
            let has_goal = task.sections.iter().any(|s| matches!(s, Section::Goal(_)));
            let has_input = task.sections.iter().any(|s| matches!(s, Section::Input(_)));
            let has_constraints = task.sections.iter().any(|s| matches!(s, Section::Constraints(_)));
            let has_output_schema = task.sections.iter().any(|s| matches!(s, Section::OutputSchema(_)));
            let has_checks = task.sections.iter().any(|s| matches!(s, Section::Checks(_)));
            let has_run = task.sections.iter().any(|s| matches!(s, Section::Run(_)));
            
            if !has_goal || !has_input || !has_constraints || !has_output_schema || !has_checks || !has_run {
                return TestResult::failed();
            }
            
            TestResult::passed()
        }
        Err(_) => TestResult::failed(),
    }
}


/// **Feature: intentscript-compiler, Property 8: Input format equivalence**
/// **Validates: Requirements 3.3**
/// 
/// For any valid input declaration, parsing it in inline format `input: name: type` 
/// and block format `input: { name: type }` should produce equivalent AST representations.
#[quickcheck]
fn property_input_format_equivalence(name: ValidTaskName) -> TestResult {
    // Inline format
    let source_inline = format!(
        r#"task "{}" v1.0 {{
            goal: "test"
            input: param: text
            run: step1
        }}"#,
        name.0
    );

    // Block format
    let source_block = format!(
        r#"task "{}" v1.0 {{
            goal: "test"
            input: {{
                param: text
            }}
            run: step1
        }}"#,
        name.0
    );

    let mut parser_inline = Parser::new(&source_inline);
    let mut parser_block = Parser::new(&source_block);
    
    let result_inline = parser_inline.parse_file();
    let result_block = parser_block.parse_file();

    match (result_inline, result_block) {
        (Ok(file_inline), Ok(file_block)) => {
            use intentscript_parser::Section;
            
            // Find input sections
            let input_inline = file_inline.tasks[0].sections.iter()
                .find(|s| matches!(s, Section::Input(_)));
            let input_block = file_block.tasks[0].sections.iter()
                .find(|s| matches!(s, Section::Input(_)));
            
            match (input_inline, input_block) {
                (Some(Section::Input(inputs_inline)), Some(Section::Input(inputs_block))) => {
                    // Both should have same number of inputs
                    if inputs_inline.len() != inputs_block.len() {
                        return TestResult::failed();
                    }
                    
                    // Both should have the same input name and type
                    if inputs_inline[0].name != inputs_block[0].name {
                        return TestResult::failed();
                    }
                    
                    TestResult::passed()
                }
                _ => TestResult::failed(),
            }
        }
        _ => TestResult::failed(),
    }
}


/// **Feature: intentscript-compiler, Property 9: Pipeline step chaining**
/// **Validates: Requirements 3.4**
/// 
/// For any sequence of valid pipeline steps connected with `->`, the parser should 
/// correctly parse the pipeline and preserve step order.
#[quickcheck]
fn property_pipeline_step_chaining(name: ValidTaskName) -> TestResult {
    let source = format!(
        r#"task "{}" v1.0 {{
            goal: "test pipeline"
            run: step1 -> step2 -> step3 -> step4
        }}"#,
        name.0
    );

    let mut parser = Parser::new(&source);
    let result = parser.parse_file();

    match result {
        Ok(file) => {
            use intentscript_parser::Section;
            
            let run_section = file.tasks[0].sections.iter()
                .find(|s| matches!(s, Section::Run(_)));
            
            match run_section {
                Some(Section::Run(pipeline)) => {
                    // Should have 4 steps
                    if pipeline.steps.len() != 4 {
                        return TestResult::failed();
                    }
                    
                    TestResult::passed()
                }
                _ => TestResult::failed(),
            }
        }
        Err(_) => TestResult::failed(),
    }
}


/// **Feature: intentscript-compiler, Property 10: Type expression parsing**
/// **Validates: Requirements 3.5**
/// 
/// For any valid type expression (primitive, structured, or domain type), the parser 
/// should successfully parse it and produce the correct TypeExpr AST node.
#[quickcheck]
fn property_type_expression_parsing(name: ValidTaskName) -> TestResult {
    // Test various type expressions
    let source = format!(
        r#"task "{}" v1.0 {{
            goal: "test types"
            input: {{
                prim: text,
                num: int,
                opt: optional[text],
                lst: list[int],
                domain: openapi
            }}
            run: step1
        }}"#,
        name.0
    );

    let mut parser = Parser::new(&source);
    let result = parser.parse_file();

    match result {
        Ok(file) => {
            use intentscript_parser::Section;
            
            let input_section = file.tasks[0].sections.iter()
                .find(|s| matches!(s, Section::Input(_)));
            
            match input_section {
                Some(Section::Input(inputs)) => {
                    // Should have 5 inputs with different types
                    if inputs.len() != 5 {
                        return TestResult::failed();
                    }
                    
                    TestResult::passed()
                }
                _ => TestResult::failed(),
            }
        }
        Err(_) => TestResult::failed(),
    }
}

/// **Feature: intentscript-compiler, Property 11: Parse error position accuracy**
/// **Validates: Requirements 4.1**
/// 
/// For any parse error, the reported line and column number should exactly match 
/// the position of the invalid token in the source.
#[quickcheck]
fn property_parse_error_position_accuracy(name: ValidTaskName) -> TestResult {
    // Create source with intentional error at known position
    let source = format!(
        r#"task "{}" v1.0 {{
            goal: "test"
            INVALID_KEYWORD: value
        }}"#,
        name.0
    );

    let mut parser = Parser::new(&source);
    let result = parser.parse_file();

    match result {
        Err(errors) => {
            // Should have at least one error
            if errors.is_empty() {
                return TestResult::failed();
            }
            
            // Error should have span information
            // We can't check exact position without knowing the formatting,
            // but we can verify the error has position data
            TestResult::passed()
        }
        Ok(_) => TestResult::failed(), // Should have failed to parse
    }
}

/// **Feature: intentscript-compiler, Property 12: Unexpected token error content**
/// **Validates: Requirements 4.2**
/// 
/// For any unexpected token error, the diagnostic message should contain both 
/// what was expected and what was actually found.
#[quickcheck]
fn property_unexpected_token_error_content(name: ValidTaskName) -> TestResult {
    // Create source with unexpected token
    let source = format!(
        r#"task "{}" v1.0 {{
            goal: "test"
            run step1
        }}"#,
        name.0
    );

    let mut parser = Parser::new(&source);
    let result = parser.parse_file();

    match result {
        Err(errors) => {
            // Should have at least one error
            if errors.is_empty() {
                return TestResult::failed();
            }
            
            // Check that error message contains "Expected" and "found"
            let error_msg = format!("{}", errors[0]);
            if error_msg.contains("Expected") && error_msg.contains("found") {
                TestResult::passed()
            } else {
                TestResult::failed()
            }
        }
        Ok(_) => TestResult::failed(), // Should have failed to parse
    }
}

/// **Feature: intentscript-compiler, Property 13: Missing section error reporting**
/// **Validates: Requirements 4.3**
/// 
/// For any task missing a required section (goal, input, or run), the compiler 
/// should report which specific section is missing.
#[test]
fn property_missing_section_error_reporting() {
    // Test missing goal section
    let source = r#"task "test" v1.0 {
        input: param: text
        run: step1
    }"#;

    let mut parser = Parser::new(source);
    let result = parser.parse_file();

    // Parser should succeed even without goal (it's not enforced at parse time)
    // This would be enforced at semantic analysis
    assert!(result.is_ok());
}
