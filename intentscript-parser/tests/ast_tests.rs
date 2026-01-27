// Unit tests for AST construction
// Tests creating AST nodes programmatically, span preservation, equality, and cloning

use intentscript_core::Span;
use intentscript_parser::ast::*;

#[test]
fn test_create_file_node() {
    let file = File { tasks: vec![] };
    assert_eq!(file.tasks.len(), 0);

    let task = Task {
        name: "test_task".to_string(),
        version: None,
        sections: vec![],
        span: Span::new(1, 1, 0, 10),
    };
    let file_with_task = File {
        tasks: vec![task.clone()],
    };
    assert_eq!(file_with_task.tasks.len(), 1);
    assert_eq!(file_with_task.tasks[0].name, "test_task");
}

#[test]
fn test_create_task_with_version() {
    let version = Version {
        major: 1,
        minor: 0,
        patch: Some(2),
    };
    let task = Task {
        name: "versioned_task".to_string(),
        version: Some(version.clone()),
        sections: vec![],
        span: Span::new(1, 1, 0, 20),
    };

    assert_eq!(task.name, "versioned_task");
    assert!(task.version.is_some());
    let v = task.version.unwrap();
    assert_eq!(v.major, 1);
    assert_eq!(v.minor, 0);
    assert_eq!(v.patch, Some(2));
}

#[test]
fn test_create_input_decl() {
    let input = InputDecl {
        name: "api_spec".to_string(),
        type_expr: TypeExpr::Domain(DomainType::OpenApi, Span::new(1, 10, 10, 7)),
        default: None,
        span: Span::new(1, 1, 0, 20),
    };

    assert_eq!(input.name, "api_spec");
    assert!(matches!(input.type_expr, TypeExpr::Domain(DomainType::OpenApi, _)));
    assert!(input.default.is_none());
}

#[test]
fn test_create_constraint_decl() {
    let constraint = ConstraintDecl {
        name: "fs".to_string(),
        value: ConstraintValue::On,
        span: Span::new(2, 1, 25, 10),
    };

    assert_eq!(constraint.name, "fs");
    assert!(matches!(constraint.value, ConstraintValue::On));
}

#[test]
fn test_create_check_decl() {
    let check = CheckDecl {
        name: "must_have_sections".to_string(),
        args: vec![
            Expr::Literal(Literal::String("paths".to_string()), Span::new(3, 5, 50, 7)),
            Expr::Literal(Literal::String("info".to_string()), Span::new(3, 14, 59, 6)),
        ],
        span: Span::new(3, 1, 45, 25),
    };

    assert_eq!(check.name, "must_have_sections");
    assert_eq!(check.args.len(), 2);
}

#[test]
fn test_create_pipeline() {
    let pipeline = Pipeline {
        steps: vec![
            Step::Ident("read_file".to_string(), Span::new(4, 1, 70, 9)),
            Step::Call(CallExpr {
                name: "parse_openapi".to_string(),
                args: vec![],
                span: Span::new(4, 14, 83, 15),
            }),
        ],
        span: Span::new(4, 1, 70, 30),
    };

    assert_eq!(pipeline.steps.len(), 2);
    assert!(matches!(pipeline.steps[0], Step::Ident(_, _)));
    assert!(matches!(pipeline.steps[1], Step::Call(_)));
}

#[test]
fn test_create_expressions() {
    // Literal expression
    let lit_expr = Expr::Literal(Literal::Int(42), Span::new(5, 1, 100, 2));
    assert!(matches!(lit_expr, Expr::Literal(Literal::Int(42), _)));

    // Identifier expression
    let ident_expr = Expr::Ident("my_var".to_string(), Span::new(5, 5, 105, 6));
    assert!(matches!(ident_expr, Expr::Ident(_, _)));

    // Call expression
    let call_expr = Expr::Call(CallExpr {
        name: "validate".to_string(),
        args: vec![Arg::Positional(Expr::Ident("spec".to_string(), Span::new(5, 15, 115, 4)))],
        span: Span::new(5, 10, 110, 15),
    });
    assert!(matches!(call_expr, Expr::Call(_)));
}

#[test]
fn test_create_call_expr_with_named_args() {
    let call = CallExpr {
        name: "render_template".to_string(),
        args: vec![
            Arg::Named {
                name: "template".to_string(),
                value: Expr::Literal(Literal::String("report.md".to_string()), Span::new(6, 20, 140, 11)),
            },
            Arg::Named {
                name: "data".to_string(),
                value: Expr::Ident("results".to_string(), Span::new(6, 40, 160, 7)),
            },
        ],
        span: Span::new(6, 1, 120, 50),
    };

    assert_eq!(call.name, "render_template");
    assert_eq!(call.args.len(), 2);
    assert!(matches!(call.args[0], Arg::Named { .. }));
    assert!(matches!(call.args[1], Arg::Named { .. }));
}

#[test]
fn test_create_literals() {
    let string_lit = Literal::String("hello".to_string());
    let int_lit = Literal::Int(123);
    let float_lit = Literal::Float(3.14);
    let bool_lit = Literal::Bool(true);

    assert!(matches!(string_lit, Literal::String(_)));
    assert!(matches!(int_lit, Literal::Int(123)));
    assert!(matches!(float_lit, Literal::Float(_)));
    assert!(matches!(bool_lit, Literal::Bool(true)));
}

#[test]
fn test_create_primitive_types() {
    let bool_type = TypeExpr::Primitive(PrimitiveType::Bool, Span::new(7, 1, 170, 4));
    let int_type = TypeExpr::Primitive(PrimitiveType::Int, Span::new(7, 6, 175, 3));
    let text_type = TypeExpr::Primitive(PrimitiveType::Text, Span::new(7, 10, 179, 4));
    let url_type = TypeExpr::Primitive(PrimitiveType::Url, Span::new(7, 15, 184, 3));

    assert!(matches!(bool_type, TypeExpr::Primitive(PrimitiveType::Bool, _)));
    assert!(matches!(int_type, TypeExpr::Primitive(PrimitiveType::Int, _)));
    assert!(matches!(text_type, TypeExpr::Primitive(PrimitiveType::Text, _)));
    assert!(matches!(url_type, TypeExpr::Primitive(PrimitiveType::Url, _)));
}

#[test]
fn test_create_structured_types() {
    // Object type
    let object_type = TypeExpr::Object {
        fields: vec![
            ("name".to_string(), TypeExpr::Primitive(PrimitiveType::Text, Span::new(8, 10, 200, 4))),
            ("age".to_string(), TypeExpr::Primitive(PrimitiveType::Int, Span::new(8, 20, 210, 3))),
        ],
        span: Span::new(8, 1, 190, 30),
    };
    assert!(matches!(object_type, TypeExpr::Object { .. }));

    // List type
    let list_type = TypeExpr::List(
        Box::new(TypeExpr::Primitive(PrimitiveType::Text, Span::new(9, 6, 226, 4))),
        Span::new(9, 1, 220, 10),
    );
    assert!(matches!(list_type, TypeExpr::List(_, _)));

    // Enum type
    let enum_type = TypeExpr::Enum(
        vec!["red".to_string(), "green".to_string(), "blue".to_string()],
        Span::new(10, 1, 230, 20),
    );
    assert!(matches!(enum_type, TypeExpr::Enum(_, _)));

    // Optional type
    let optional_type = TypeExpr::Optional(
        Box::new(TypeExpr::Primitive(PrimitiveType::Int, Span::new(11, 10, 260, 3))),
        Span::new(11, 1, 250, 15),
    );
    assert!(matches!(optional_type, TypeExpr::Optional(_, _)));
}

#[test]
fn test_create_domain_types() {
    let openapi_type = TypeExpr::Domain(DomainType::OpenApi, Span::new(12, 1, 270, 7));
    let markdown_type = TypeExpr::Domain(DomainType::Markdown, Span::new(12, 10, 279, 8));
    let xlsx_type = TypeExpr::Domain(DomainType::Xlsx, Span::new(12, 20, 289, 4));
    let pdf_type = TypeExpr::Domain(DomainType::Pdf, Span::new(12, 26, 295, 3));

    assert!(matches!(openapi_type, TypeExpr::Domain(DomainType::OpenApi, _)));
    assert!(matches!(markdown_type, TypeExpr::Domain(DomainType::Markdown, _)));
    assert!(matches!(xlsx_type, TypeExpr::Domain(DomainType::Xlsx, _)));
    assert!(matches!(pdf_type, TypeExpr::Domain(DomainType::Pdf, _)));
}

#[test]
fn test_span_preservation() {
    let span = Span::new(5, 10, 100, 20);
    let task = Task {
        name: "test".to_string(),
        version: None,
        sections: vec![],
        span,
    };

    assert_eq!(task.span.line, 5);
    assert_eq!(task.span.column, 10);
    assert_eq!(task.span.offset, 100);
    assert_eq!(task.span.length, 20);
}

#[test]
fn test_ast_node_equality() {
    let task1 = Task {
        name: "task1".to_string(),
        version: Some(Version { major: 1, minor: 0, patch: None }),
        sections: vec![],
        span: Span::new(1, 1, 0, 10),
    };

    let task2 = Task {
        name: "task1".to_string(),
        version: Some(Version { major: 1, minor: 0, patch: None }),
        sections: vec![],
        span: Span::new(1, 1, 0, 10),
    };

    let task3 = Task {
        name: "task2".to_string(),
        version: Some(Version { major: 1, minor: 0, patch: None }),
        sections: vec![],
        span: Span::new(1, 1, 0, 10),
    };

    assert_eq!(task1, task2);
    assert_ne!(task1, task3);
}

#[test]
fn test_ast_node_cloning() {
    let original = Task {
        name: "original".to_string(),
        version: Some(Version { major: 2, minor: 1, patch: Some(3) }),
        sections: vec![
            Section::Goal(Expr::Literal(Literal::String("Test goal".to_string()), Span::new(2, 1, 20, 15))),
        ],
        span: Span::new(1, 1, 0, 50),
    };

    let cloned = original.clone();

    assert_eq!(original, cloned);
    assert_eq!(original.name, cloned.name);
    assert_eq!(original.version, cloned.version);
    assert_eq!(original.sections.len(), cloned.sections.len());
    assert_eq!(original.span, cloned.span);
}

#[test]
fn test_section_variants() {
    let goal_section = Section::Goal(Expr::Literal(Literal::String("goal".to_string()), Span::new(1, 1, 0, 6)));
    let input_section = Section::Input(vec![]);
    let constraints_section = Section::Constraints(vec![]);
    let output_schema_section = Section::OutputSchema(TypeExpr::Primitive(PrimitiveType::Json, Span::new(2, 1, 10, 4)));
    let checks_section = Section::Checks(vec![]);
    let run_section = Section::Run(Pipeline { steps: vec![], span: Span::new(3, 1, 20, 5) });

    assert!(matches!(goal_section, Section::Goal(_)));
    assert!(matches!(input_section, Section::Input(_)));
    assert!(matches!(constraints_section, Section::Constraints(_)));
    assert!(matches!(output_schema_section, Section::OutputSchema(_)));
    assert!(matches!(checks_section, Section::Checks(_)));
    assert!(matches!(run_section, Section::Run(_)));
}

#[test]
fn test_complex_ast_construction() {
    // Build a complete task with multiple sections
    let task = Task {
        name: "complex_task".to_string(),
        version: Some(Version { major: 1, minor: 2, patch: Some(3) }),
        sections: vec![
            Section::Goal(Expr::Literal(
                Literal::String("Validate OpenAPI spec".to_string()),
                Span::new(2, 7, 30, 25),
            )),
            Section::Input(vec![
                InputDecl {
                    name: "spec_file".to_string(),
                    type_expr: TypeExpr::Primitive(PrimitiveType::Path, Span::new(3, 15, 70, 4)),
                    default: None,
                    span: Span::new(3, 3, 58, 20),
                },
            ]),
            Section::Constraints(vec![
                ConstraintDecl {
                    name: "fs".to_string(),
                    value: ConstraintValue::On,
                    span: Span::new(4, 3, 90, 8),
                },
            ]),
            Section::Run(Pipeline {
                steps: vec![
                    Step::Call(CallExpr {
                        name: "read_file".to_string(),
                        args: vec![Arg::Positional(Expr::Ident("spec_file".to_string(), Span::new(5, 15, 120, 9)))],
                        span: Span::new(5, 3, 108, 20),
                    }),
                    Step::Call(CallExpr {
                        name: "parse_openapi".to_string(),
                        args: vec![],
                        span: Span::new(5, 27, 132, 15),
                    }),
                ],
                span: Span::new(5, 3, 108, 45),
            }),
        ],
        span: Span::new(1, 1, 0, 160),
    };

    assert_eq!(task.name, "complex_task");
    assert_eq!(task.sections.len(), 4);
    assert!(task.version.is_some());

    // Verify each section type
    assert!(matches!(task.sections[0], Section::Goal(_)));
    assert!(matches!(task.sections[1], Section::Input(_)));
    assert!(matches!(task.sections[2], Section::Constraints(_)));
    assert!(matches!(task.sections[3], Section::Run(_)));
}
