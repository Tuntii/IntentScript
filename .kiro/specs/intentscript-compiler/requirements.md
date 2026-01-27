# Requirements Document

## Introduction

IntentScript is a declarative task language designed to express intent, constraints, schemas, and validation checks, and to compile them into a deterministic execution plan that can run in CI and tooling. This requirements document defines the functional and non-functional requirements for implementing the IntentScript compiler and runtime system. The implementation will be language-first (not framework-first), with a clear separation between the language core, compiler pipeline, intermediate representation (IR), and runtime execution model.

## Glossary

- **IntentScript**: A declarative task language for expressing workflow tasks with validation and constraints
- **Compiler**: The system that transforms IntentScript source code into executable IR
- **IR (Intermediate Representation)**: The deterministic, serializable Execution Plan format
- **Runtime**: The execution engine that runs IR with a host adapter
- **Host**: The capability-based interface for effectful operations (file I/O, templates, etc.)
- **AST (Abstract Syntax Tree)**: The language-level representation of parsed IntentScript source
- **Task**: A unit of work defined in IntentScript with goal, inputs, constraints, checks, and execution pipeline
- **Lexer**: The component that converts source text into tokens
- **Parser**: The component that converts tokens into AST
- **Semantic Analyzer**: The component that performs type checking and constraint validation
- **Capability**: A permission gate for side effects (fs, net, exec, templates, exports)
- **Deterministic Execution**: Execution that produces identical results given identical inputs and configuration
- **Policy**: A set of rules governing task execution, validation, and capabilities

## Requirements

### Requirement 1

**User Story:** As a language designer, I want IntentScript to have its own syntax and semantics independent of Rust, so that the language can be portable across different runtime hosts.

#### Acceptance Criteria

1. WHEN the compiler processes IntentScript source THEN the system SHALL parse it using language-specific grammar rules that do not expose Rust types or concepts
2. WHEN defining the AST THEN the system SHALL use language-level constructs (tasks, expressions, types) rather than Rust-specific structures
3. WHEN generating IR THEN the system SHALL produce a host-agnostic representation that can execute on any compliant runtime
4. WHEN documenting language features THEN the system SHALL describe them in terms of IntentScript semantics, not implementation details

### Requirement 2

**User Story:** As a compiler engineer, I want a clear lexical structure for IntentScript, so that source code can be tokenized consistently and precisely.

#### Acceptance Criteria

1. WHEN the lexer encounters whitespace (spaces, tabs, newlines) THEN the system SHALL recognize it as token separators
2. WHEN the lexer encounters line comments starting with "//" THEN the system SHALL ignore all characters until end of line
3. WHEN the lexer encounters identifiers matching `[A-Za-z_][A-Za-z0-9_]*` THEN the system SHALL tokenize them as IDENT tokens
4. WHEN the lexer encounters double-quoted strings THEN the system SHALL parse them as UTF-8 string literals with escape sequences `\n`, `\t`, `\"`, `\\`
5. WHEN the lexer encounters numeric literals THEN the system SHALL distinguish between integers `[0-9]+` and floats `[0-9]+\.[0-9]+`

### Requirement 3

**User Story:** As a developer, I want to write IntentScript tasks with clear structure, so that I can express workflow intent declaratively.

#### Acceptance Criteria

1. WHEN parsing a task block THEN the system SHALL recognize the syntax `task "name" v1.0 { ... }` with optional version
2. WHEN parsing task sections THEN the system SHALL support goal, input, constraints, output_schema, checks, and run sections
3. WHEN parsing input declarations THEN the system SHALL support both inline `input: name: type` and block-style `input: { name: type }` formats
4. WHEN parsing pipelines THEN the system SHALL recognize step chaining with `->` operator
5. WHEN parsing type expressions THEN the system SHALL support primitives (bool, int, float, text, url, email, path, bytes, json), structured types (object, list, enum, optional), and domain types (openapi, markdown, xlsx, pdf)

### Requirement 4

**User Story:** As a compiler developer, I want precise error reporting, so that users can quickly identify and fix syntax errors.

#### Acceptance Criteria

1. WHEN a parse error occurs THEN the system SHALL report the exact line and column number
2. WHEN an unexpected token is encountered THEN the system SHALL indicate what was expected and what was found
3. WHEN a required section is missing THEN the system SHALL report which section is missing and where it should appear
4. WHEN diagnostics are generated THEN the system SHALL provide actionable messages that explain how to fix the issue

### Requirement 5

**User Story:** As a language implementer, I want a well-defined AST structure, so that the compiler can represent IntentScript programs accurately.

#### Acceptance Criteria

1. WHEN constructing the AST THEN the system SHALL represent files as collections of task definitions
2. WHEN representing tasks THEN the system SHALL include name, version, goal, inputs, constraints, output schema, checks, and run pipeline
3. WHEN representing expressions THEN the system SHALL support literals, identifiers, and function calls with named or positional arguments
4. WHEN representing types THEN the system SHALL distinguish between primitive types, structured types, and domain-specific types
5. WHEN preserving source information THEN the system SHALL attach line and column metadata to AST nodes for diagnostic purposes

### Requirement 6

**User Story:** As a type system designer, I want static type checking for IntentScript, so that type errors are caught at compile time.

#### Acceptance Criteria

1. WHEN analyzing input declarations THEN the system SHALL verify that each input has a valid type annotation
2. WHEN analyzing function calls THEN the system SHALL verify that argument types match expected parameter types
3. WHEN analyzing pipeline steps THEN the system SHALL verify that data flows between steps have compatible types
4. WHEN an optional type is used THEN the system SHALL enforce that it is explicitly handled according to policy
5. WHEN a type mismatch is detected THEN the system SHALL report a compile-time error with the expected and actual types

### Requirement 7

**User Story:** As a constraint solver implementer, I want to detect contradictions in task definitions, so that ambiguous or impossible tasks are rejected at compile time.

#### Acceptance Criteria

1. WHEN analyzing constraints THEN the system SHALL detect mutually exclusive constraint declarations
2. WHEN policy rules conflict with task constraints THEN the system SHALL report the contradiction with both sources
3. WHEN ambiguous constructs are detected THEN the system SHALL report them as errors unless policy allows resolution
4. WHEN constraint solving completes THEN the system SHALL produce a consistent set of constraints or fail compilation

### Requirement 8

**User Story:** As a CI engineer, I want deterministic compilation, so that identical source produces identical IR across builds.

#### Acceptance Criteria

1. WHEN compiling IntentScript source with identical compiler version, policy hash, and inputs THEN the system SHALL produce byte-identical IR after canonical serialization
2. WHEN serializing IR to JSON THEN the system SHALL use a canonical ordering of fields and stable formatting
3. WHEN hashing policy THEN the system SHALL include all policy rules that affect compilation
4. WHEN versioning IR THEN the system SHALL embed the schema version, compiler version, and policy hash in the IR metadata

### Requirement 9

**User Story:** As a compiler developer, I want to lower typed AST to IR, so that tasks can be executed without re-parsing source.

#### Acceptance Criteria

1. WHEN lowering a task THEN the system SHALL generate an ExecutionPlan with meta, inputs, steps, outputs, limits, and capabilities
2. WHEN lowering pipeline steps THEN the system SHALL convert each step into an IR step with id, kind, args, produces, and checks
3. WHEN lowering constraints THEN the system SHALL translate them into capability gates (fs, net, exec, templates, exports)
4. WHEN lowering checks THEN the system SHALL embed them in the appropriate IR steps
5. WHEN serializing IR THEN the system SHALL produce valid JSON conforming to the IR schema version 1.0

### Requirement 10

**User Story:** As a runtime implementer, I want a deterministic execution model, so that IR execution is predictable and auditable.

#### Acceptance Criteria

1. WHEN executing an ExecutionPlan THEN the system SHALL process steps as a deterministic state machine
2. WHEN a step requires side effects THEN the system SHALL delegate to the Host interface rather than performing effects directly
3. WHEN recording effects THEN the system SHALL append all operations to an audit log
4. WHEN validation checks fail THEN the system SHALL trigger bounded repair passes according to policy (max_repairs limit)
5. WHEN execution completes THEN the system SHALL produce artifacts and a complete audit trail

### Requirement 11

**User Story:** As a security engineer, I want capability-based gating, so that tasks cannot perform unauthorized side effects.

#### Acceptance Criteria

1. WHEN a task attempts file system access THEN the system SHALL verify that fs capability is enabled with appropriate read/write roots
2. WHEN a task attempts network access THEN the system SHALL verify that net capability is enabled (default is off)
3. WHEN a task attempts external command execution THEN the system SHALL verify that exec capability is enabled
4. WHEN a task attempts template rendering THEN the system SHALL verify that templates capability is enabled
5. WHEN a capability check fails THEN the system SHALL reject the operation and report a policy violation

### Requirement 12

**User Story:** As a developer, I want a CLI for working with IntentScript, so that I can build, run, lint, and debug tasks.

#### Acceptance Criteria

1. WHEN running `intentscript build` THEN the system SHALL compile source to IR and report any errors
2. WHEN running `intentscript run` THEN the system SHALL execute IR with the configured Host adapter
3. WHEN running `intentscript lint` THEN the system SHALL perform static checks and policy validation without execution
4. WHEN running `intentscript fmt` THEN the system SHALL format source code according to style rules
5. WHEN running `intentscript explain` THEN the system SHALL describe the execution plan and validation results
6. WHEN running in CI mode THEN the system SHALL output machine-readable JSON diagnostics with stable exit codes

### Requirement 13

**User Story:** As a runtime host implementer, I want a clear Host trait interface, so that I can provide effectful operations to the runtime.

#### Acceptance Criteria

1. WHEN the runtime needs to read a file THEN the system SHALL call the Host read method with the file path
2. WHEN the runtime needs to write a file THEN the system SHALL call the Host write method with the path and bytes
3. WHEN the runtime needs to render a template THEN the system SHALL call the Host render_template method with template name and variables
4. WHEN the runtime needs to export structured data THEN the system SHALL call the appropriate Host export method (e.g., export_xlsx)
5. WHEN the runtime needs to parse domain-specific formats THEN the system SHALL call the appropriate Host parse method (e.g., parse_openapi)

### Requirement 14

**User Story:** As a language user, I want bounded execution guarantees, so that tasks cannot run indefinitely or consume unbounded resources.

#### Acceptance Criteria

1. WHEN analyzing loops THEN the system SHALL reject unbounded loop constructs at compile time
2. WHEN iteration is required THEN the system SHALL only allow bounded mapping over explicit collections
3. WHEN repair passes are triggered THEN the system SHALL enforce the max_repairs limit (default 2)
4. WHEN a timeout is configured THEN the system SHALL terminate execution after timeout_ms milliseconds
5. WHEN resource limits are exceeded THEN the system SHALL report the violation and halt execution

### Requirement 15

**User Story:** As a validation engineer, I want declarative checks, so that I can validate artifacts against schemas and policies.

#### Acceptance Criteria

1. WHEN checks are declared THEN the system SHALL support predicates like must_have_sections, must_not_contain, and validate
2. WHEN executing checks THEN the system SHALL evaluate them against intermediate artifacts or final outputs
3. WHEN a check fails THEN the system SHALL produce a diagnostic with the check name, expected condition, and actual result
4. WHEN checks reference output_schema THEN the system SHALL validate outputs conform to the declared schema
5. WHEN all checks pass THEN the system SHALL mark the task as successful
