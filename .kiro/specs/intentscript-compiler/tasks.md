# Implementation Plan

- [x] 1. Set up project structure and core types





  - Create Rust workspace with crates: intentscript-core, intentscript-parser, intentscript-compiler, intentscript-runtime, intentscript-cli
  - Define core data structures in intentscript-core: Span, Position, Error types
  - Set up dependencies (serde, serde_json, quickcheck/proptest, clap, thiserror, sha2)
  - Configure workspace Cargo.toml with shared dependencies
  - _Requirements: 1.1, 1.2, 1.3_


- [x] 2. Implement lexer (tokenization)



  - [x] 2.1 Define Token and TokenKind types


    - Create Token struct with kind, lexeme, and span fields
    - Define TokenKind enum with all token types (keywords, identifiers, literals, symbols)
    - Implement Display trait for tokens for debugging
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

  - [x] 2.2 Implement Lexer struct with tokenization logic


    - Create Lexer struct with source text and position tracking
    - Implement next_token() method with character-by-character scanning
    - Handle whitespace, comments, identifiers, string literals, numeric literals
    - Track line and column numbers for span information
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

  - [x] 2.3 Write property test for whitespace token separation


    - **Property 1: Whitespace token separation**
    - **Validates: Requirements 2.1**

  - [x] 2.4 Write property test for comment handling


    - **Property 2: Comment content ignored**
    - **Validates: Requirements 2.2**

  - [x] 2.5 Write property test for identifier tokenization


    - **Property 3: Valid identifier tokenization**
    - **Validates: Requirements 2.3**

  - [x] 2.6 Write property test for string literal parsing


    - **Property 4: String literal parsing with escapes**
    - **Validates: Requirements 2.4**






  - [x] 2.7 Write property test for numeric type distinction



    - **Property 5: Numeric type distinction**
    - **Validates: Requirements 2.5**

- [ ] 3. Implement AST data structures



  - [ ] 3.1 Define AST node types
    - Create File, Task, Section, InputDecl, ConstraintDecl, CheckDecl structs
    - Create Pipeline, Step, Expr, CallExpr, Arg enums and structs





    - Create TypeExpr, PrimitiveType, DomainType enums

    - Create Literal enum for all literal types
    - Add Span field to all AST nodes for source location tracking


    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

  - [ ] 3.2 Write unit tests for AST construction
    - Test creating AST nodes programmatically
    - Verify span information is preserved


    - Test AST node equality and cloning
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_


- [x] 4. Implement parser (AST generation)

  - [ ] 4.1 Create Parser struct with token stream
    - Create Parser struct wrapping Lexer
    - Implement peek_token(), next_token(), expect() helper methods
    - Implement error recovery with synchronization points
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 4.1, 4.2, 4.3_



  - [ ] 4.2 Implement task and section parsing
    - Implement parse_file() to parse multiple tasks


    - Implement parse_task() for task declarations with name and version
    - Implement parse_section() dispatcher for all section types


    - Implement parse_goal(), parse_input(), parse_constraints(), parse_output_schema(), parse_checks(), parse_run()
    - _Requirements: 3.1, 3.2_



  - [x] 4.3 Implement expression and type parsing


    - Implement parse_expr() for literals, identifiers, and calls
    - Implement parse_call_expr() for function calls with arguments

    - Implement parse_type_expr() for all type forms

    - Implement parse_pipeline() for step chaining with ->

    - _Requirements: 3.3, 3.4, 3.5_



  - [x] 4.4 Implement error reporting with precise locations


    - Create ParseError type with span and message







    - Generate errors with line/column information

    - Include expected vs found information in errors
    - Report missing required sections

    - _Requirements: 4.1, 4.2, 4.3_

  - [ ] 4.5 Write property test for task declaration parsing
    - **Property 6: Task declaration parsing**
    - **Validates: Requirements 3.1**

  - [x] 4.6 Write property test for section parsing completeness


    - **Property 7: Section parsing completeness**
    - **Validates: Requirements 3.2**



  - [x] 4.7 Write property test for input format equivalence


    - **Property 8: Input format equivalence**
    - **Validates: Requirements 3.3**



  - [x] 4.8 Write property test for pipeline step chaining




    - **Property 9: Pipeline step chaining**



    - **Validates: Requirements 3.4**

  - [ ] 4.9 Write property test for type expression parsing
    - **Property 10: Type expression parsing**
    - **Validates: Requirements 3.5**



  - [ ] 4.10 Write property test for parse error position accuracy
    - **Property 11: Parse error position accuracy**


    - **Validates: Requirements 4.1**



  - [ ] 4.11 Write property test for unexpected token errors
    - **Property 12: Unexpected token error content**





    - **Validates: Requirements 4.2**




  - [ ] 4.12 Write property test for missing section errors
    - **Property 13: Missing section error reporting**
    - **Validates: Requirements 4.3**



- [ ] 5. Checkpoint - Ensure lexer and parser tests pass




  - Ensure all tests pass, ask the user if questions arise.

- [ ] 6. Implement semantic analysis (type checking)

  - [ ] 6.1 Create SymbolTable and type checking infrastructure
    - Create SymbolTable struct with inputs, constraints, pipeline_vars maps
    - Create TypeChecker struct with symbol table


    - Define SemanticError type for type errors
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_



  - [x] 6.2 Implement type checking for expressions and pipelines


    - Implement check_expr() to verify expression types
    - Implement check_call() to verify function call argument types


    - Implement check_pipeline() to verify step type compatibility
    - Implement check_optional_handling() for optional type policy


    - Generate type mismatch errors with expected and actual types
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_



  - [x] 6.3 Write property test for input type validation


    - **Property 19: Input type validation**
    - **Validates: Requirements 6.1**



  - [x] 6.4 Write property test for function call type checking



    - **Property 20: Function call type checking**



    - **Validates: Requirements 6.2**





  - [ ] 6.5 Write property test for pipeline type compatibility
    - **Property 21: Pipeline type compatibility**
    - **Validates: Requirements 6.3**

  - [x] 6.6 Write property test for optional type policy enforcement


    - **Property 22: Optional type policy enforcement**
    - **Validates: Requirements 6.4**

  - [ ] 6.7 Write property test for type mismatch error content
    - **Property 23: Type mismatch error content**
    - **Validates: Requirements 6.5**

- [ ] 7. Implement constraint solving

  - [ ] 7.1 Create constraint solver
    - Create ConstraintSolver struct
    - Implement detect_contradictions() to find mutually exclusive constraints
    - Implement check_policy_conflicts() to find policy-task conflicts
    - Implement resolve_ambiguities() with policy-driven resolution
    - Generate errors with both sources for conflicts
    - _Requirements: 7.1, 7.2, 7.3, 7.4_

  - [ ] 7.2 Write property test for constraint contradiction detection
    - **Property 24: Constraint contradiction detection**
    - **Validates: Requirements 7.1**

  - [ ] 7.3 Write property test for policy-task conflict reporting
    - **Property 25: Policy-task conflict reporting**
    - **Validates: Requirements 7.2**

  - [ ] 7.4 Write property test for ambiguity resolution policy
    - **Property 26: Ambiguity resolution policy**
    - **Validates: Requirements 7.3**

  - [ ] 7.5 Write property test for constraint set consistency
    - **Property 27: Constraint set consistency**
    - **Validates: Requirements 7.4**

- [ ] 8. Implement IR data structures

  - [ ] 8.1 Define ExecutionPlan and related types
    - Create ExecutionPlan, Metadata, InputSpec, Capabilities, Limits structs
    - Create IRStep, StepKind, IRCheck, ArtifactSpec structs
    - Implement serde Serialize/Deserialize for all IR types
    - Use canonical field ordering for deterministic serialization
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_

  - [ ] 8.2 Write unit tests for IR serialization
    - Test JSON serialization of ExecutionPlan
    - Verify canonical field ordering
    - Test round-trip serialization/deserialization
    - _Requirements: 9.5_


- [ ] 9. Implement IR lowering (AST to IR)
  - [ ] 9.1 Create IR lowering pass
    - Create Lowering struct with policy
    - Implement lower_task() to convert Task to ExecutionPlan
    - Implement lower_pipeline() to convert Pipeline to Vec<IRStep>
    - Implement lower_constraints() to convert constraints to Capabilities
    - Implement lower_checks() to embed checks in IR steps
    - Compute policy hash using SHA-256
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_

  - [ ] 9.2 Write property test for compilation determinism
    - **Property 28: Compilation determinism**
    - **Validates: Requirements 8.1**

  - [ ] 9.3 Write property test for IR serialization determinism
    - **Property 29: IR serialization determinism**
    - **Validates: Requirements 8.2**

  - [ ] 9.4 Write property test for policy hash stability
    - **Property 30: Policy hash stability**
    - **Validates: Requirements 8.3**

  - [ ] 9.5 Write property test for IR metadata completeness
    - **Property 31: IR metadata completeness**
    - **Validates: Requirements 8.4**

  - [ ] 9.6 Write property test for ExecutionPlan structure
    - **Property 32: ExecutionPlan structure completeness**
    - **Validates: Requirements 9.1**

  - [ ] 9.7 Write property test for pipeline step lowering
    - **Property 33: Pipeline step lowering**
    - **Validates: Requirements 9.2**

  - [ ] 9.8 Write property test for constraint to capability translation
    - **Property 34: Constraint to capability translation**
    - **Validates: Requirements 9.3**

  - [ ] 9.9 Write property test for check embedding
    - **Property 35: Check embedding in IR**
    - **Validates: Requirements 9.4**

  - [ ] 9.10 Write property test for IR JSON schema conformance
    - **Property 36: IR JSON schema conformance**
    - **Validates: Requirements 9.5**

- [ ] 10. Checkpoint - Ensure compiler tests pass

  - Ensure all tests pass, ask the user if questions arise.


- [x] 11. Implement Host trait interface


  - [x] 11.1 Define Host trait

    - Create Host trait with methods: read_file, write_file, render_template
    - Add domain parser methods: parse_openapi, parse_markdown
    - Add export methods: export_xlsx, export_pdf
    - Add log_operation method for audit trail
    - Define error types for Host operations
    - _Requirements: 13.1, 13.2, 13.3, 13.4, 13.5_

  - [x] 11.2 Write unit tests for Host trait

    - Create mock Host implementation for testing
    - Test each Host method with valid inputs
    - Test error handling for Host operations
    - _Requirements: 13.1, 13.2, 13.3, 13.4, 13.5_



- [x] 12. Implement runtime executor


  - [x] 12.1 Create runtime state machine


    - Create ExecutionState struct with plan, variables, artifacts, audit_log
    - Create Executor struct with Host reference
    - Implement execute() method with lifecycle: plan -> generate -> validate -> repair -> finalize
    - Implement execute_step() for individual IR step execution
    - Track repair count and enforce max_repairs limit
    - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5_

  - [x] 12.2 Implement capability enforcement


    - Create CapabilityChecker struct
    - Implement check_fs_capability() with read/write root validation
    - Implement check_net_capability() (default false)
    - Implement check_exec_capability()
    - Implement check_templates_capability()
    - Generate policy violation errors for unauthorized operations
    - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5_

  - [x] 12.3 Implement validation and checking


    - Create Validator struct
    - Implement validate_checks() to evaluate checks against artifacts
    - Implement validate_schema() for output schema validation
    - Generate check failure diagnostics with name, expected, actual
    - Implement bounded repair mechanism
    - _Requirements: 15.1, 15.2, 15.3, 15.4, 15.5_

  - [x] 12.4 Implement audit logging


    - Create AuditLog struct with append-only log entries
    - Log all Host operations with timestamp and details
    - Log capability checks and violations
    - Log validation results and repair attempts
    - _Requirements: 10.3_

  - [x] 12.5 Write property test for execution determinism


    - **Property 37: Execution determinism**
    - **Validates: Requirements 10.1**

  - [x] 12.6 Write property test for effect delegation


    - **Property 38: Effect delegation to Host**
    - **Validates: Requirements 10.2**

  - [x] 12.7 Write property test for audit log completeness


    - **Property 39: Audit log completeness**
    - **Validates: Requirements 10.3**

  - [x] 12.8 Write property test for bounded repair enforcement


    - **Property 40: Bounded repair enforcement**
    - **Validates: Requirements 10.4**

  - [x] 12.9 Write property test for execution output completeness


    - **Property 41: Execution output completeness**
    - **Validates: Requirements 10.5**

  - [x] 12.10 Write property test for filesystem capability enforcement


    - **Property 42: Filesystem capability enforcement**
    - **Validates: Requirements 11.1**

  - [x] 12.11 Write property test for network capability enforcement


    - **Property 43: Network capability enforcement**
    - **Validates: Requirements 11.2**

  - [x] 12.12 Write property test for exec capability enforcement


    - **Property 44: Exec capability enforcement**
    - **Validates: Requirements 11.3**

  - [x] 12.13 Write property test for template capability enforcement


    - **Property 45: Template capability enforcement**
    - **Validates: Requirements 11.4**

  - [x] 12.14 Write property test for capability violation rejection


    - **Property 46: Capability violation rejection**
    - **Validates: Requirements 11.5**

  - [x] 12.15 Write property test for Host read delegation


    - **Property 49: Host read delegation**
    - **Validates: Requirements 13.1**

  - [x] 12.16 Write property test for Host write delegation


    - **Property 50: Host write delegation**
    - **Validates: Requirements 13.2**

  - [x] 12.17 Write property test for Host template delegation


    - **Property 51: Host template delegation**
    - **Validates: Requirements 13.3**

  - [x] 12.18 Write property test for Host export delegation


    - **Property 52: Host export delegation**
    - **Validates: Requirements 13.4**

  - [x] 12.19 Write property test for Host parse delegation


    - **Property 53: Host parse delegation**
    - **Validates: Requirements 13.5**

  - [x] 12.20 Write property test for unbounded loop rejection


    - **Property 54: Unbounded loop rejection**
    - **Validates: Requirements 14.1**


  - [x] 12.21 Write property test for bounded iteration enforcement

    - **Property 55: Bounded iteration enforcement**
    - **Validates: Requirements 14.2**

  - [x] 12.22 Write property test for repair limit enforcement


    - **Property 56: Repair limit enforcement**
    - **Validates: Requirements 14.3**

  - [x] 12.23 Write property test for resource limit violation handling


    - **Property 57: Resource limit violation handling**
    - **Validates: Requirements 14.5**

  - [x] 12.24 Write property test for check predicate support


    - **Property 58: Check predicate support**
    - **Validates: Requirements 15.1**

  - [x] 12.25 Write property test for check evaluation


    - **Property 59: Check evaluation against artifacts**
    - **Validates: Requirements 15.2**


  - [x] 12.26 Write property test for check failure diagnostics

    - **Property 60: Check failure diagnostic content**
    - **Validates: Requirements 15.3**

  - [x] 12.27 Write property test for output schema validation


    - **Property 61: Output schema validation**
    - **Validates: Requirements 15.4**

  - [x] 12.28 Write property test for successful execution marking


    - **Property 62: Successful execution marking**
    - **Validates: Requirements 15.5**


- [x] 13. Checkpoint - Ensure runtime tests pass



  - Ensure all tests pass, ask the user if questions arise.



- [x] 14. Implement CLI commands




  - [x] 14.1 Create CLI structure with clap

    - Create intentscript-cli crate
    - Define CLI commands: build, run, lint, fmt, explain
    - Parse command-line arguments with clap
    - Set up error handling and exit codes
    - _Requirements: 12.1, 12.2, 12.3, 12.4, 12.5, 12.6_


  - [x] 14.2 Implement build command

    - Read IntentScript source file
    - Invoke compiler pipeline (lex, parse, analyze, lower)
    - Write ExecutionPlan IR to output file
    - Report errors with source locations
    - Support --output flag for IR destination
    - _Requirements: 12.1_



  - [ ] 14.3 Implement run command
    - Read ExecutionPlan IR from file
    - Create Host adapter (default or specified)
    - Execute IR with runtime
    - Write artifacts to output locations
    - Display audit log summary
    - Support --host flag for Host selection
    - _Requirements: 12.2_

  - [x] 14.4 Implement lint command

    - Read IntentScript source file
    - Run compiler pipeline up to semantic analysis
    - Report all errors and warnings
    - Do not generate IR or execute
    - Support --json flag for machine-readable output
    - _Requirements: 12.3_

  - [x] 14.5 Implement fmt command

    - Read IntentScript source file
    - Parse to AST
    - Pretty-print AST back to source
    - Write formatted source to file or stdout
    - Support --check flag to verify formatting without writing
    - _Requirements: 12.4_

  - [x] 14.6 Implement explain command

    - Read ExecutionPlan IR from file
    - Display human-readable explanation of execution plan
    - Show steps, capabilities, checks, and limits
    - Explain why checks passed or failed (if execution log provided)
    - _Requirements: 12.5_

  - [x] 14.7 Implement CI mode with JSON diagnostics

    - Add --json flag to all commands
    - Output structured JSON diagnostics
    - Use stable exit codes (0 = success, 1 = error, 2 = warning)
    - Include error codes, locations, and messages in JSON
    - _Requirements: 12.6_

  - [x] 14.8 Write property test for format idempotence


    - **Property 47: Format idempotence**
    - **Validates: Requirements 12.4**

  - [x] 14.9 Write property test for CI JSON diagnostic validity


    - **Property 48: CI JSON diagnostic validity**
    - **Validates: Requirements 12.6**

  - [x] 14.10 Write integration tests for CLI commands



    - Test build command with valid and invalid source
    - Test run command with sample IR and mock Host
    - Test lint command with various error conditions


    - Test fmt command with unformatted source
    - Test explain command with sample IR
    - _Requirements: 12.1, 12.2, 12.3, 12.4, 12.5_


- [-] 15. Implement example tasks and documentation



  - [ ] 15.1 Create example IntentScript tasks

    - Implement OpenAPI lint task from spec


    - Implement cookbook validation task from spec
    - Create additional example tasks demonstrating features
    - Add comments explaining task structure
    - _Requirements: All_

  - [x] 15.2 Write user documentation


    - Create README with project overview and quick start
    - Document IntentScript syntax and semantics
    - Document CLI commands with examples
    - Document Host trait for custom adapters



    - Create tutorial for writing first task
    - _Requirements: All_

  - [ ] 15.3 Write integration tests with example tasks
    - Test compiling and running OpenAPI lint task
    - Test compiling and running cookbook validation task
    - Verify end-to-end workflow: source -> IR -> execution -> artifacts
    - _Requirements: All_



- [ ] 16. Final checkpoint - Ensure all tests pass

  - Ensure all tests pass, ask the user if questions arise.
