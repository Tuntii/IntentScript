# IntentScript Compiler and Runtime Design

## Overview

IntentScript is a standalone declarative task language with its own syntax, semantics, and execution model. This design document describes the architecture of the IntentScript compiler and runtime system, emphasizing language-first principles and clear separation between the language core, compiler pipeline, intermediate representation (IR), and runtime execution.

The system is designed around three key artifacts:
1. **Language Core**: Syntax and semantics independent of any implementation language
2. **Compiler Pipeline**: Lexer → Parser → AST → Semantic Analysis → IR
3. **Runtime System**: Host-agnostic execution engine with capability-based side effects

The reference implementation is in Rust, but the language design ensures portability to other runtimes (WASM, other hosts) through a stable, versioned IR format.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    IntentScript Source                       │
│                    (.intent files)                           │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                   Compiler Pipeline                          │
│  ┌──────┐   ┌────────┐   ┌─────────┐   ┌──────────────┐   │
│  │Lexer │──▶│Parser  │──▶│Semantic │──▶│IR Lowering   │   │
│  │      │   │        │   │Analysis │   │              │   │
│  └──────┘   └────────┘   └─────────┘   └──────────────┘   │
│      │           │             │               │            │
│   Tokens       AST       Typed AST            IR            │
└─────────────────────────────────────────────┬───────────────┘
                                              │
                                              ▼
                                    ┌──────────────────┐
                                    │  Execution Plan  │
                                    │  (JSON IR)       │
                                    └────────┬─────────┘
                                             │
                                             ▼
┌─────────────────────────────────────────────────────────────┐
│                      Runtime System                          │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐   │
│  │State Machine │──▶│Validator     │──▶│Artifact      │   │
│  │Executor      │   │& Checker     │   │Generator     │   │
│  └──────┬───────┘   └──────────────┘   └──────────────┘   │
│         │                                                    │
│         ▼                                                    │
│  ┌──────────────────────────────────────────────────────┐  │
│  │            Host Interface (Trait)                     │  │
│  │  • File I/O (read/write with capability gates)       │  │
│  │  • Template rendering                                 │  │
│  │  • Domain parsers (OpenAPI, etc.)                    │  │
│  │  • Export functions (XLSX, PDF, etc.)                │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                         │
                         ▼
                  ┌─────────────┐
                  │ Audit Log   │
                  └─────────────┘
```

### Separation of Concerns

The architecture maintains strict boundaries:

1. **Language Layer**: AST and type system are language concepts, not Rust concepts
2. **Compiler Layer**: Transforms language constructs to IR without runtime dependencies
3. **IR Layer**: Stable, versioned, serializable contract between compiler and runtime
4. **Runtime Layer**: Executes IR through capability-gated Host interface
5. **Host Adapter Layer**: Rust-specific implementations (RustAPI bridge)

## Components and Interfaces

### 1. Lexer

**Responsibility**: Convert source text into tokens

**Input**: IntentScript source string  
**Output**: Token stream with position information

**Token Types**:
```
Token {
  kind: TokenKind,
  lexeme: String,
  span: Span { line, column, offset, length }
}

TokenKind:
  - Keyword(task, goal, input, constraints, output_schema, checks, run, etc.)
  - Identifier(String)
  - StringLiteral(String)
  - IntLiteral(i64)
  - FloatLiteral(f64)
  - Symbol('{', '}', ':', ',', '(', ')', '[', ']', '=', '->', '|>')
  - Comment(String)
  - Whitespace
  - EOF
```

**Key Operations**:
- `next_token() -> Token`: Advance to next token
- `peek_token() -> Token`: Look ahead without consuming
- `skip_whitespace()`: Consume whitespace and comments

### 2. Parser

**Responsibility**: Convert token stream into AST

**Input**: Token stream from Lexer  
**Output**: AST (Abstract Syntax Tree)

**Parsing Strategy**: Recursive descent with operator precedence for expressions

**Error Recovery**: 
- Synchronize on section boundaries (goal, input, constraints, etc.)
- Report all errors in a single pass when possible
- Preserve partial AST for better diagnostics

**Key Operations**:
- `parse_file() -> Result<File, Vec<ParseError>>`
- `parse_task() -> Result<Task, ParseError>`
- `parse_section() -> Result<Section, ParseError>`
- `parse_expr() -> Result<Expr, ParseError>`
- `parse_type() -> Result<TypeExpr, ParseError>`

### 3. Semantic Analyzer

**Responsibility**: Type checking, constraint validation, symbol resolution

**Input**: AST  
**Output**: Typed AST + Symbol Table + Diagnostics

**Analysis Phases**:
1. **Symbol Collection**: Build symbol table of inputs, constraints, identifiers
2. **Type Checking**: Verify type consistency across expressions and pipelines
3. **Constraint Solving**: Detect contradictions and ambiguities
4. **Policy Validation**: Check against policy rules

**Symbol Table Structure**:
```
SymbolTable {
  inputs: HashMap<String, TypeExpr>,
  constraints: HashMap<String, ConstraintValue>,
  pipeline_vars: HashMap<String, TypeExpr>,
  checks: Vec<CheckDecl>
}
```

**Type Checking Rules**:
- Input declarations must have valid types
- Function call arguments must match expected types
- Pipeline steps must have compatible input/output types
- Optional types must be explicitly handled

**Key Operations**:
- `analyze(ast: AST) -> Result<TypedAST, Vec<SemanticError>>`
- `check_types(expr: Expr, expected: TypeExpr) -> Result<(), TypeError>`
- `solve_constraints(constraints: Vec<Constraint>) -> Result<ConstraintSet, ContradictionError>`
- `validate_policy(task: Task, policy: Policy) -> Result<(), PolicyViolation>`

### 4. IR Lowering

**Responsibility**: Transform Typed AST into Execution Plan IR

**Input**: Typed AST + Policy  
**Output**: ExecutionPlan (serializable IR)

**Lowering Strategy**:
- Convert task sections to IR metadata and steps
- Translate pipeline to sequential step graph
- Embed checks into step validation
- Generate capability gates from constraints
- Compute policy hash for determinism

**Key Operations**:
- `lower(typed_ast: TypedAST, policy: Policy) -> Result<ExecutionPlan, LoweringError>`
- `lower_pipeline(pipeline: Pipeline) -> Vec<IRStep>`
- `lower_constraints(constraints: ConstraintSet) -> Capabilities`
- `compute_policy_hash(policy: Policy) -> String`

### 5. Runtime Executor

**Responsibility**: Execute IR as deterministic state machine

**Input**: ExecutionPlan + Inputs + Host  
**Output**: Artifacts + Audit Log

**Execution Lifecycle**:
```
Plan → Generate → Validate → Repair (bounded) → Finalize
```

**State Machine**:
- Each step transitions state deterministically
- Side effects only through Host interface
- All operations logged to audit trail
- Bounded repair on validation failures

**Key Operations**:
- `execute(plan: ExecutionPlan, inputs: Inputs, host: &dyn Host) -> Result<Artifacts, RuntimeError>`
- `execute_step(step: IRStep, state: &mut State) -> Result<StepOutput, StepError>`
- `validate_checks(checks: Vec<Check>, artifact: &Artifact) -> Result<(), CheckFailure>`
- `repair(state: &mut State, failure: CheckFailure) -> Result<(), RepairError>`

### 6. Host Interface

**Responsibility**: Provide capability-gated effectful operations

**Interface Definition** (language-agnostic contract):
```
trait Host {
  // File system operations (gated by fs capability)
  fn read_file(path: &str) -> Result<Bytes>
  fn write_file(path: &str, content: Bytes) -> Result<()>
  
  // Template operations (gated by templates capability)
  fn render_template(name: &str, vars: JsonValue) -> Result<String>
  
  // Domain parsers
  fn parse_openapi(bytes: Bytes) -> Result<OpenApiDoc>
  fn parse_markdown(bytes: Bytes) -> Result<MarkdownDoc>
  
  // Export operations (gated by exports capability)
  fn export_xlsx(spec: XlsxSpec, rows: Vec<Row>) -> Result<Bytes>
  fn export_pdf(spec: PdfSpec, content: String) -> Result<Bytes>
  
  // Audit logging
  fn log_operation(op: Operation) -> Result<()>
}
```

**Capability Enforcement**:
- Runtime checks capabilities before delegating to Host
- Host implementations may add additional restrictions
- All operations are logged for audit trail

## Data Models

### AST (Abstract Syntax Tree)

The AST represents the language-level structure of IntentScript programs:

```
File {
  tasks: Vec<Task>
}

Task {
  name: String,
  version: Option<Version>,
  sections: Vec<Section>,
  span: Span
}

Version {
  major: u32,
  minor: u32,
  patch: Option<u32>
}

Section:
  | Goal(Expr)
  | Input(Vec<InputDecl>)
  | Constraints(Vec<ConstraintDecl>)
  | OutputSchema(TypeExpr)
  | Checks(Vec<CheckDecl>)
  | Run(Pipeline)

InputDecl {
  name: String,
  type_expr: TypeExpr,
  default: Option<Literal>,
  span: Span
}

ConstraintDecl {
  name: String,
  value: ConstraintValue,
  span: Span
}

ConstraintValue:
  | On
  | Off
  | Literal(Literal)
  | Expr(Expr)

CheckDecl {
  name: String,
  args: Vec<Expr>,
  span: Span
}

Pipeline {
  steps: Vec<Step>,
  span: Span
}

Step:
  | Call(CallExpr)
  | Ident(String)

Expr:
  | Literal(Literal)
  | Ident(String)
  | Call(CallExpr)

CallExpr {
  name: String,
  args: Vec<Arg>,
  span: Span
}

Arg:
  | Named { name: String, value: Expr }
  | Positional(Expr)

Literal:
  | String(String)
  | Int(i64)
  | Float(f64)
  | Bool(bool)

TypeExpr:
  | Primitive(PrimitiveType)
  | Object { fields: Vec<(String, TypeExpr)> }
  | List(Box<TypeExpr>)
  | Enum(Vec<String>)
  | Optional(Box<TypeExpr>)
  | Domain(DomainType)

PrimitiveType:
  | Bool | Int | Float | Text | Url | Email | Path | Bytes | Json

DomainType:
  | OpenApi | Markdown | Xlsx | Pdf

Span {
  line: u32,
  column: u32,
  offset: usize,
  length: usize
}
```

### IR (Execution Plan)

The IR is the stable contract between compiler and runtime:

```
ExecutionPlan {
  schema_version: String,  // "1.0"
  meta: Metadata,
  inputs: Vec<InputSpec>,
  capabilities: Capabilities,
  limits: Limits,
  steps: Vec<IRStep>,
  outputs: Vec<ArtifactSpec>
}

Metadata {
  task_name: String,
  task_version: String,
  compiler_version: String,
  policy_hash: String
}

InputSpec {
  name: String,
  type_name: String,
  required: bool,
  default: Option<JsonValue>
}

Capabilities {
  fs: Option<FsCapability>,
  net: bool,
  exec: bool,
  templates: bool,
  exports: bool
}

FsCapability {
  read_roots: Vec<String>,
  write_roots: Vec<String>
}

Limits {
  max_repairs: u32,
  timeout_ms: Option<u64>
}

IRStep {
  id: String,
  kind: StepKind,
  args: HashMap<String, JsonValue>,
  produces: Option<String>,  // variable name
  checks: Vec<IRCheck>
}

StepKind:
  | ReadFile
  | WriteFile
  | ParseOpenApi
  | ParseMarkdown
  | RenderTemplate
  | ExportXlsx
  | ExportPdf
  | Validate
  | Report
  | Custom(String)

IRCheck {
  name: String,
  args: HashMap<String, JsonValue>
}

ArtifactSpec {
  path: String,
  type_name: String
}
```

### Runtime State

```
ExecutionState {
  plan: ExecutionPlan,
  variables: HashMap<String, Value>,
  artifacts: Vec<Artifact>,
  audit_log: Vec<LogEntry>,
  repair_count: u32
}

Value:
  | Bytes(Vec<u8>)
  | String(String)
  | Int(i64)
  | Float(f64)
  | Bool(bool)
  | Json(JsonValue)
  | OpenApiDoc(OpenApiDoc)
  | MarkdownDoc(MarkdownDoc)

Artifact {
  path: String,
  content: Value,
  type_name: String
}

LogEntry {
  timestamp: u64,
  operation: String,
  details: JsonValue
}
```


## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Whitespace token separation

*For any* IntentScript source containing whitespace characters (spaces, tabs, newlines), the lexer should correctly separate tokens on either side of the whitespace.

**Validates: Requirements 2.1**

### Property 2: Comment content ignored

*For any* string content following "//", the lexer should ignore all characters until end of line and not include them in any token.

**Validates: Requirements 2.2**

### Property 3: Valid identifier tokenization

*For any* string matching the pattern `[A-Za-z_][A-Za-z0-9_]*`, the lexer should produce an IDENT token with the correct lexeme.

**Validates: Requirements 2.3**

### Property 4: String literal parsing with escapes

*For any* valid double-quoted string containing escape sequences (`\n`, `\t`, `\"`, `\\`), the lexer should correctly parse the string and interpret escape sequences.

**Validates: Requirements 2.4**

### Property 5: Numeric type distinction

*For any* numeric literal, the lexer should correctly classify it as either an integer (no decimal point) or float (with decimal point).

**Validates: Requirements 2.5**

### Property 6: Task declaration parsing

*For any* valid task name and optional version, the parser should successfully parse the task declaration `task "name" v1.0 { ... }`.

**Validates: Requirements 3.1**

### Property 7: Section parsing completeness

*For any* valid combination of task sections (goal, input, constraints, output_schema, checks, run), the parser should successfully parse all sections into the AST.

**Validates: Requirements 3.2**

### Property 8: Input format equivalence

*For any* valid input declaration, parsing it in inline format `input: name: type` and block format `input: { name: type }` should produce equivalent AST representations.

**Validates: Requirements 3.3**

### Property 9: Pipeline step chaining

*For any* sequence of valid pipeline steps connected with `->`, the parser should correctly parse the pipeline and preserve step order.

**Validates: Requirements 3.4**

### Property 10: Type expression parsing

*For any* valid type expression (primitive, structured, or domain type), the parser should successfully parse it and produce the correct TypeExpr AST node.

**Validates: Requirements 3.5**

### Property 11: Parse error position accuracy

*For any* parse error, the reported line and column number should exactly match the position of the invalid token in the source.

**Validates: Requirements 4.1**

### Property 12: Unexpected token error content

*For any* unexpected token error, the diagnostic message should contain both what was expected and what was actually found.

**Validates: Requirements 4.2**

### Property 13: Missing section error reporting

*For any* task missing a required section (goal, input, or run), the compiler should report which specific section is missing.

**Validates: Requirements 4.3**

### Property 14: File AST structure

*For any* valid IntentScript file, the AST root should be a File node containing a collection of Task nodes.

**Validates: Requirements 5.1**

### Property 15: Task AST completeness

*For any* parsed task, the AST should contain fields for name, version, goal, inputs, constraints, output_schema, checks, and run pipeline (with required sections present).

**Validates: Requirements 5.2**

### Property 16: Expression AST representation

*For any* valid expression (literal, identifier, or function call), the AST should correctly represent its type and structure.

**Validates: Requirements 5.3**

### Property 17: Type categorization

*For any* type expression, the AST should correctly categorize it as primitive, structured, or domain-specific type.

**Validates: Requirements 5.4**

### Property 18: Span metadata preservation

*For any* AST node, the span metadata should accurately reflect the node's position in the source (line, column, offset, length).

**Validates: Requirements 5.5**

### Property 19: Input type validation

*For any* input declaration, the semantic analyzer should verify that the type annotation is valid and well-formed.

**Validates: Requirements 6.1**

### Property 20: Function call type checking

*For any* function call, the semantic analyzer should verify that argument types match the expected parameter types, or report a type error.

**Validates: Requirements 6.2**

### Property 21: Pipeline type compatibility

*For any* pipeline, the semantic analyzer should verify that the output type of each step is compatible with the input type of the next step.

**Validates: Requirements 6.3**

### Property 22: Optional type policy enforcement

*For any* usage of optional types, the semantic analyzer should enforce policy rules for explicit handling.

**Validates: Requirements 6.4**

### Property 23: Type mismatch error content

*For any* type mismatch, the error diagnostic should include both the expected type and the actual type found.

**Validates: Requirements 6.5**

### Property 24: Constraint contradiction detection

*For any* set of mutually exclusive constraints (e.g., `net = on` and `net = off`), the semantic analyzer should detect and report the contradiction.

**Validates: Requirements 7.1**

### Property 25: Policy-task conflict reporting

*For any* conflict between policy rules and task constraints, the error should identify both the policy rule and the task constraint.

**Validates: Requirements 7.2**

### Property 26: Ambiguity resolution policy

*For any* ambiguous construct, the semantic analyzer should report an error unless the policy explicitly allows resolution.

**Validates: Requirements 7.3**

### Property 27: Constraint set consistency

*For any* constraint set, the semantic analyzer should either produce a consistent set of constraints or fail compilation with a clear error.

**Validates: Requirements 7.4**

### Property 28: Compilation determinism

*For any* IntentScript source, compiling it twice with identical compiler version, policy hash, and inputs should produce byte-identical IR.

**Validates: Requirements 8.1**

### Property 29: IR serialization determinism

*For any* ExecutionPlan IR, serializing it to JSON multiple times should produce byte-identical output.

**Validates: Requirements 8.2**

### Property 30: Policy hash stability

*For any* policy, computing its hash multiple times should produce identical results, and any change to the policy should produce a different hash.

**Validates: Requirements 8.3**

### Property 31: IR metadata completeness

*For any* generated ExecutionPlan, the metadata should contain schema_version, task_name, task_version, compiler_version, and policy_hash.

**Validates: Requirements 8.4**

### Property 32: ExecutionPlan structure completeness

*For any* lowered task, the ExecutionPlan should contain all required fields: meta, inputs, capabilities, limits, steps, and outputs.

**Validates: Requirements 9.1**

### Property 33: Pipeline step lowering

*For any* pipeline with N steps, the lowered IR should contain N IRStep entries with correct id, kind, args, produces, and checks fields.

**Validates: Requirements 9.2**

### Property 34: Constraint to capability translation

*For any* constraint set, the lowering process should correctly translate constraints into the Capabilities structure (fs, net, exec, templates, exports).

**Validates: Requirements 9.3**

### Property 35: Check embedding in IR

*For any* check declaration, it should appear in the checks field of the appropriate IRStep in the lowered IR.

**Validates: Requirements 9.4**

### Property 36: IR JSON schema conformance

*For any* serialized ExecutionPlan, the JSON should be valid and conform to the IR schema version 1.0 specification.

**Validates: Requirements 9.5**

### Property 37: Execution determinism

*For any* ExecutionPlan with identical inputs and host behavior, executing it multiple times should produce identical artifacts and state transitions.

**Validates: Requirements 10.1**

### Property 38: Effect delegation to Host

*For any* step requiring side effects (file I/O, network, etc.), the runtime should delegate to the Host interface rather than performing the effect directly.

**Validates: Requirements 10.2**

### Property 39: Audit log completeness

*For any* effectful operation during execution, an entry should appear in the audit log with operation details.

**Validates: Requirements 10.3**

### Property 40: Bounded repair enforcement

*For any* execution with check failures, the number of repair passes should not exceed the max_repairs limit specified in the ExecutionPlan.

**Validates: Requirements 10.4**

### Property 41: Execution output completeness

*For any* successful execution, the result should include both the generated artifacts and a complete audit trail.

**Validates: Requirements 10.5**

### Property 42: Filesystem capability enforcement

*For any* file system operation, the runtime should verify that the fs capability is enabled and the path is within allowed read/write roots before delegating to Host.

**Validates: Requirements 11.1**

### Property 43: Network capability enforcement

*For any* network operation, the runtime should verify that the net capability is enabled (default is false) before delegating to Host.

**Validates: Requirements 11.2**

### Property 44: Exec capability enforcement

*For any* external command execution, the runtime should verify that the exec capability is enabled before delegating to Host.

**Validates: Requirements 11.3**

### Property 45: Template capability enforcement

*For any* template rendering operation, the runtime should verify that the templates capability is enabled before delegating to Host.

**Validates: Requirements 11.4**

### Property 46: Capability violation rejection

*For any* operation attempted without the required capability, the runtime should reject it and report a policy violation error.

**Validates: Requirements 11.5**

### Property 47: Format idempotence

*For any* valid IntentScript source, formatting it twice should produce identical output: format(format(x)) = format(x).

**Validates: Requirements 12.4**

### Property 48: CI JSON diagnostic validity

*For any* compilation result in CI mode, the JSON diagnostic output should be valid JSON and contain stable exit codes.

**Validates: Requirements 12.6**

### Property 49: Host read delegation

*For any* file read operation in the IR, the runtime should call the Host read_file method with the correct path.

**Validates: Requirements 13.1**

### Property 50: Host write delegation

*For any* file write operation in the IR, the runtime should call the Host write_file method with the correct path and content.

**Validates: Requirements 13.2**

### Property 51: Host template delegation

*For any* template rendering operation in the IR, the runtime should call the Host render_template method with the template name and variables.

**Validates: Requirements 13.3**

### Property 52: Host export delegation

*For any* export operation in the IR, the runtime should call the appropriate Host export method (export_xlsx, export_pdf, etc.).

**Validates: Requirements 13.4**

### Property 53: Host parse delegation

*For any* domain-specific parsing operation in the IR, the runtime should call the appropriate Host parse method (parse_openapi, parse_markdown, etc.).

**Validates: Requirements 13.5**

### Property 54: Unbounded loop rejection

*For any* source containing unbounded loop constructs, the compiler should reject it at compile time with an appropriate error.

**Validates: Requirements 14.1**

### Property 55: Bounded iteration enforcement

*For any* iteration construct in the source, the compiler should verify it is bounded (mapping over explicit collections) or reject it.

**Validates: Requirements 14.2**

### Property 56: Repair limit enforcement

*For any* execution with repeated check failures, the runtime should enforce the max_repairs limit and halt after reaching it.

**Validates: Requirements 14.3**

### Property 57: Resource limit violation handling

*For any* resource limit violation (memory, file size, etc.), the runtime should report the violation and halt execution.

**Validates: Requirements 14.5**

### Property 58: Check predicate support

*For any* valid check declaration using supported predicates (must_have_sections, must_not_contain, validate), the parser and runtime should correctly handle it.

**Validates: Requirements 15.1**

### Property 59: Check evaluation against artifacts

*For any* check in the ExecutionPlan, the runtime should evaluate it against the appropriate artifact (intermediate or final output).

**Validates: Requirements 15.2**

### Property 60: Check failure diagnostic content

*For any* check failure, the diagnostic should include the check name, expected condition, and actual result.

**Validates: Requirements 15.3**

### Property 61: Output schema validation

*For any* task with output_schema, the runtime should validate that the final output conforms to the declared schema.

**Validates: Requirements 15.4**

### Property 62: Successful execution marking

*For any* execution where all checks pass, the runtime should mark the task result as successful.

**Validates: Requirements 15.5**

## Error Handling

### Compiler Errors

The compiler produces three categories of errors:

1. **Lexical Errors**: Invalid characters, unterminated strings, malformed numbers
2. **Syntax Errors**: Unexpected tokens, missing required elements, malformed structures
3. **Semantic Errors**: Type mismatches, undefined identifiers, constraint contradictions, policy violations

All errors include:
- Precise source location (file, line, column)
- Error code for programmatic handling
- Human-readable message
- Suggested fix when applicable

### Runtime Errors

The runtime produces errors for:

1. **Capability Violations**: Attempted operations without required capabilities
2. **Host Errors**: Failures in Host operations (file not found, network error, etc.)
3. **Validation Failures**: Check failures, schema violations
4. **Resource Limits**: Timeout, max repairs exceeded, memory limits

Runtime errors include:
- Step ID where error occurred
- Error category and code
- Audit log context
- Recovery suggestions when applicable

### Error Recovery

**Compiler**: 
- Synchronizes on section boundaries to report multiple errors
- Preserves partial AST for better diagnostics
- Does not attempt to generate IR if semantic errors exist

**Runtime**:
- Bounded repair attempts for validation failures
- Graceful degradation with audit trail
- No silent failures - all errors are reported

## Testing Strategy

### Dual Testing Approach

The IntentScript compiler and runtime will be validated using both unit tests and property-based tests:

- **Unit tests** verify specific examples, edge cases, and error conditions
- **Property-based tests** verify universal properties that should hold across all inputs
- Together they provide comprehensive coverage: unit tests catch concrete bugs, property tests verify general correctness

### Unit Testing

Unit tests will cover:

1. **Lexer**: Specific token sequences, edge cases (empty input, only whitespace, unterminated strings)
2. **Parser**: Specific task structures, error recovery, partial parsing
3. **Semantic Analysis**: Specific type checking scenarios, constraint solving examples
4. **IR Lowering**: Specific task-to-IR transformations, capability mapping
5. **Runtime**: Specific execution scenarios, capability enforcement, audit logging
6. **CLI**: Command-line interface behavior, output formatting

Unit tests will use standard Rust testing with `#[test]` attributes and the `assert!` family of macros.

### Property-Based Testing

Property-based tests will verify the 62 correctness properties defined above. We will use **QuickCheck** (or **proptest**) as the property-based testing library for Rust.

**Configuration**:
- Each property-based test will run a minimum of 100 iterations
- Tests will use smart generators that constrain to valid input spaces
- Each test will be tagged with a comment referencing the design document property

**Property Test Structure**:
```rust
#[quickcheck]
fn property_name(input: ValidInput) -> bool {
    // Feature: intentscript-compiler, Property N: description
    // Validates: Requirements X.Y
    
    // Test implementation
    let result = system_under_test(input);
    verify_property(result)
}
```

**Generators**:
- `ArbitraryToken`: Generate valid tokens
- `ArbitraryIdent`: Generate valid identifiers
- `ArbitraryExpr`: Generate valid expressions
- `ArbitraryType`: Generate valid type expressions
- `ArbitraryTask`: Generate valid task definitions
- `ArbitraryPipeline`: Generate valid pipelines

**Key Properties to Test**:
1. **Determinism** (Properties 28, 29, 37): Critical for CI reproducibility
2. **Round-trip** (Property 8): Input format equivalence
3. **Idempotence** (Property 47): Formatting stability
4. **Capability enforcement** (Properties 42-46): Security guarantees
5. **Bounded execution** (Properties 54-56): Safety guarantees
6. **Error reporting** (Properties 11-13, 23, 25): Diagnostic quality

### Integration Testing

Integration tests will verify:
- End-to-end compilation: source → IR
- End-to-end execution: IR → artifacts
- CLI commands with real file system
- Host adapter implementations

### Test Organization

```
intentscript/
├── intentscript-parser/
│   ├── src/
│   │   └── lib.rs
│   └── tests/
│       ├── unit/
│       │   ├── lexer_tests.rs
│       │   └── parser_tests.rs
│       └── properties/
│           ├── lexer_properties.rs
│           └── parser_properties.rs
├── intentscript-compiler/
│   ├── src/
│   │   └── lib.rs
│   └── tests/
│       ├── unit/
│       │   ├── semantic_tests.rs
│       │   └── lowering_tests.rs
│       └── properties/
│           ├── determinism_properties.rs
│           └── type_checking_properties.rs
├── intentscript-runtime/
│   ├── src/
│   │   └── lib.rs
│   └── tests/
│       ├── unit/
│       │   ├── executor_tests.rs
│       │   └── capability_tests.rs
│       └── properties/
│           ├── execution_properties.rs
│           └── capability_properties.rs
└── intentscript-cli/
    └── tests/
        └── integration/
            ├── build_tests.rs
            ├── run_tests.rs
            └── lint_tests.rs
```

## Implementation Phases

### Phase 1: Language Core (Lexer + Parser)

**Goal**: Establish the language syntax and produce AST

**Deliverables**:
- Lexer with token types and position tracking
- Parser with recursive descent implementation
- AST data structures
- Basic error reporting with source locations
- Unit tests for lexer and parser
- Property tests for tokenization and parsing

**Success Criteria**:
- Can parse all example tasks from the spec
- Precise error reporting for syntax errors
- Properties 1-18 pass

### Phase 2: Type System and Semantic Analysis

**Goal**: Add type checking and constraint validation

**Deliverables**:
- Type checker with type inference
- Symbol table and scope management
- Constraint solver for contradiction detection
- Policy validation framework
- Unit tests for type checking
- Property tests for type system

**Success Criteria**:
- Type errors caught at compile time
- Constraint contradictions detected
- Properties 19-27 pass

### Phase 3: IR Generation

**Goal**: Lower typed AST to executable IR

**Deliverables**:
- IR data structures (ExecutionPlan)
- Lowering pass from AST to IR
- JSON serialization with canonical ordering
- Policy hash computation
- Deterministic compilation
- Unit tests for lowering
- Property tests for determinism

**Success Criteria**:
- Valid IR generated for all valid tasks
- Byte-identical compilation for identical inputs
- Properties 28-36 pass

### Phase 4: Runtime Execution

**Goal**: Execute IR with Host interface

**Deliverables**:
- Runtime executor with state machine
- Host trait definition
- Capability enforcement
- Audit logging
- Bounded repair mechanism
- Unit tests for runtime
- Property tests for execution

**Success Criteria**:
- Deterministic execution
- All capabilities enforced
- Complete audit trail
- Properties 37-62 pass

### Phase 5: CLI and Tooling

**Goal**: Developer experience and CI integration

**Deliverables**:
- CLI commands (build, run, lint, fmt, explain)
- JSON diagnostic output
- Formatting tool
- Documentation and examples

**Success Criteria**:
- All CLI commands functional
- CI-friendly output
- Properties 47-48 pass

### Phase 6: RustAPI Integration

**Goal**: Integrate with RustAPI ecosystem

**Deliverables**:
- RustAPI Host adapter
- OpenAPI validation tasks
- Cookbook validation tasks
- Project scaffolding tasks

**Success Criteria**:
- Example tasks from spec work end-to-end
- Integration with RustAPI build process

## Dependencies

### External Crates (Rust Implementation)

- **serde** + **serde_json**: IR serialization
- **quickcheck** or **proptest**: Property-based testing
- **clap**: CLI argument parsing
- **thiserror**: Error handling
- **sha2**: Policy hash computation

### Internal Dependencies

```
intentscript-core (types, IR)
    ↑
    ├── intentscript-parser (lexer, parser, AST)
    ├── intentscript-compiler (semantic analysis, lowering)
    └── intentscript-runtime (executor, Host trait)
        ↑
        └── intentscript-cli (commands)
            ↑
            └── intentscript-rustapi (RustAPI adapters)
```

## Open Questions

1. **Generator Strategy**: Should we use QuickCheck or proptest for property-based testing? (proptest has better shrinking)
2. **IR Format**: Should we support both JSON and binary formats for IR, or JSON only for MVP?
3. **Macro Embedding**: Should MVP include `intentscript!()` macro, or defer to Phase 2?
4. **Error Codes**: What numbering scheme should we use for error codes? (e.g., E0001, E0002, ...)
5. **Policy Format**: How should policies be authored? (TOML, JSON, or IntentScript syntax?)
6. **Host Mocking**: How should we mock Host implementations for testing? (trait objects, test doubles?)

## Future Enhancements

### Post-MVP Features

1. **Block Comments**: `/* ... */` syntax
2. **String Interpolation**: `"Hello, ${name}"` syntax
3. **Advanced Types**: Union types, intersection types, type aliases
4. **Module System**: Import/export between task files
5. **Incremental Compilation**: Cache IR for unchanged tasks
6. **WASM Runtime**: Compile runtime to WebAssembly
7. **Language Server Protocol**: IDE support with autocomplete, go-to-definition
8. **Debugger**: Step through IR execution
9. **Profiler**: Performance analysis of task execution
10. **Package Registry**: Share and reuse tasks

### Portability Targets

1. **WASM**: Compile compiler and runtime to WASM for browser/edge execution
2. **Python Host**: Python implementation of Host trait
3. **JavaScript Host**: Node.js/Deno implementation of Host trait
4. **Go Host**: Go implementation of Host trait

## Summary

This design establishes IntentScript as a standalone language with:

- **Clear language semantics** independent of implementation details
- **Robust compiler pipeline** with precise error reporting
- **Stable IR format** for deterministic, reviewable execution plans
- **Capability-based runtime** with security and auditability
- **Comprehensive testing strategy** with both unit and property-based tests

The architecture maintains strict separation between language, compiler, IR, and runtime, enabling portability while the Rust reference implementation provides a solid foundation for the RustAPI ecosystem.
