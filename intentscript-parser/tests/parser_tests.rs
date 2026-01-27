use intentscript_parser::{Parser, Section};

#[test]
fn test_parse_simple_task() {
    let source = r#"
        task "example" v1.0 {
            goal: "Test task"
            input: name: text
            run: step1 -> step2
        }
    "#;

    let mut parser = Parser::new(source);
    let result = parser.parse_file();

    assert!(result.is_ok(), "Parser should succeed: {:?}", result.err());
    let file = result.unwrap();
    assert_eq!(file.tasks.len(), 1);
    
    let task = &file.tasks[0];
    assert_eq!(task.name, "example");
    assert!(task.version.is_some());
    assert_eq!(task.version.as_ref().unwrap().major, 1);
    assert_eq!(task.version.as_ref().unwrap().minor, 0);
}

#[test]
fn test_parse_task_with_sections() {
    let source = r#"
        task "full_example" v1.0 {
            goal: "Complete task"
            input: {
                name: text,
                age: int
            }
            constraints: {
                fs = on,
                net = off
            }
            output_schema: text
            checks: {
                validate(output)
            }
            run: read_file -> process -> write_file
        }
    "#;

    let mut parser = Parser::new(source);
    let result = parser.parse_file();

    assert!(result.is_ok(), "Parser should succeed: {:?}", result.err());
    let file = result.unwrap();
    assert_eq!(file.tasks.len(), 1);
    
    let task = &file.tasks[0];
    assert_eq!(task.name, "full_example");
    assert_eq!(task.sections.len(), 6);
}

#[test]
fn test_parse_error_missing_task_name() {
    let source = r#"
        task {
            goal: "Test"
        }
    "#;

    let mut parser = Parser::new(source);
    let result = parser.parse_file();

    assert!(result.is_err(), "Parser should fail for missing task name");
}

#[test]
fn test_parse_pipeline() {
    let source = r#"
        task "pipeline_test" v1.0 {
            goal: "Test pipeline"
            run: step1 -> step2(arg1, arg2) -> step3
        }
    "#;

    let mut parser = Parser::new(source);
    let result = parser.parse_file();

    assert!(result.is_ok(), "Parser should succeed: {:?}", result.err());
    let file = result.unwrap();
    let task = &file.tasks[0];
    
    // Find the run section
    let run_section = task.sections.iter().find(|s| matches!(s, Section::Run(_)));
    assert!(run_section.is_some());
    
    if let Section::Run(pipeline) = run_section.unwrap() {
        assert_eq!(pipeline.steps.len(), 3);
    }
}

#[test]
fn test_parse_type_expressions() {
    let source = r#"
        task "type_test" v1.0 {
            goal: "Test types"
            input: {
                simple: text,
                number: int,
                optional_val: optional[text],
                list_val: list[int],
                obj: object { field1: text, field2: int }
            }
            run: process
        }
    "#;

    let mut parser = Parser::new(source);
    let result = parser.parse_file();

    assert!(result.is_ok(), "Parser should succeed: {:?}", result.err());
}
