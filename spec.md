# IntentScript Language Specification (MVP)

**Status:** Draft (derived from PRD+TRD v1, dated 2026-01-26)  
**Reference implementation:** Rust (RustAPI ecosystem)  
**Primary artifact:** Deterministic **Execution Plan IR** (serializable, versioned)

> IntentScript is a declarative task language designed to express **intent**, **constraints**, **schemas**, and **validation checks**, and to compile them into a deterministic execution plan that can run in CI and tooling.  
> MVP is intentionally constrained: **no unbounded loops**; iteration is only allowed via bounded mapping over explicit collections.

---

## 1. Goals and Non-goals

### 1.1 Goals (MVP)
- Enable authoring of workflow tasks in a readable DSL aligned to RustAPI development/operations.
- Compile tasks into a deterministic **IR (Execution Plan)** that can be versioned, reviewed, and executed in CI.
- Provide first-class validation: schema checks, policy enforcement, and contradiction detection.
- Support extensibility via Host adapters while keeping a safe, testable core runtime.
- Integrate with RustAPI artifacts: OpenAPI outputs, cookbook/docs, project templates.

### 1.2 Non-goals (v1)
- General-purpose language replacement / full Turing-completeness.
- Native machine-code generation (compile-to-binary).
- Ambient network access by default (effects must be explicitly enabled by host policy).
- Unbounded agentic behavior (runtime must remain bounded and policy-driven).

---

## 2. Conformance and Determinism

### 2.1 Deterministic compilation
Given identical:
- IntentScript source (normalized),
- compiler version,
- policy hash,
- and inputs,

…the compiler **MUST** produce byte-identical IR (after canonical serialization rules).

### 2.2 Deterministic runtime
The runtime executes IR as a **deterministic state machine**. Any effectful operation MUST go through the Host interface and MUST be recorded in an audit log. Network access is **OFF by default** in MVP.

### 2.3 Capability gating
All side effects are gated by a capability policy included in the IR:
- fs (read/write roots),
- net (bool),
- exec (bool),
- templates (bool),
- exports (bool).

---

## 3. Source Files and Top-level Structure

### 3.1 File unit
A file contains **one or more** `task` blocks.

### 3.2 Task block (overview)
Each task defines:
- `goal`
- `input`
- `constraints`
- `output_schema` (optional but recommended)
- `checks`
- `run` pipeline

---

## 4. Lexical Structure

### 4.1 Whitespace
- Spaces, tabs, and newlines are whitespace.
- Newlines may be significant for readability but are not semantically significant except for diagnostics.

### 4.2 Comments
MVP supports line comments:
- `// comment until end of line`

(Block comments are reserved for future versions.)

### 4.3 Identifiers
An identifier matches:

- `IDENT := [A-Za-z_][A-Za-z0-9_]*`

### 4.4 String literals
- Double-quoted UTF-8 strings: `"..."`.
- Escape sequences are implementation-defined in MVP; reference implementation SHOULD support `\n`, `\t`, `\"`, `\\`.

### 4.5 Numbers
- `INT := [0-9]+`
- `FLOAT := [0-9]+ "." [0-9]+`
- Version literals may use dotted numeric formats like `v1.0` or `"1.0"`; the reference implementation normalizes to semantic components.

---

## 5. Syntax (MVP)

### 5.1 Informal grammar
This grammar is intentionally small and focused on task definition.

```ebnf
File        := { Task } ;

Task        := "task" String Version? "{" TaskBody "}" ;
Version     := "v" VersionNum ;
VersionNum  := INT "." INT { "." INT } ;

TaskBody    := { Section } ;

Section     := GoalSection
            | InputSection
            | ConstraintsSection
            | OutputSchemaSection
            | ChecksSection
            | RunSection ;

GoalSection         := "goal" ":" Expr ;
InputSection        := "input" ":" InputDecls ;
ConstraintsSection  := "constraints" ":" ConstraintDecls ;
OutputSchemaSection := "output_schema" ":" TypeExpr ;
ChecksSection       := "checks" ":" CheckDecls ;
RunSection          := "run" ":" Pipeline ;

InputDecls  := (InputDecl | BlockInputs) ;
BlockInputs := "{" { InputDecl } "}" ;

InputDecl   := IDENT ":" TypeExpr [ "=" Literal ] ;

ConstraintDecls := (ConstraintDecl | BlockConstraints) ;
BlockConstraints := "{" { ConstraintDecl } "}" ;

ConstraintDecl := IDENT "=" ( "on" | "off" | Literal ) | Expr ;

CheckDecls  := (CheckDecl | BlockChecks) ;
BlockChecks := "{" { CheckDecl } "}" ;

CheckDecl   := CallExpr | IDENT LiteralList ;

Pipeline    := Step { ("->" | "|>") Step } ;
Step        := CallExpr | IDENT ;

Expr        := CallExpr | IDENT | Literal ;
CallExpr    := IDENT "(" [ ArgList ] ")" ;
ArgList     := Arg { "," Arg } ;
Arg         := IDENT ":" Expr | Expr ;

Literal     := String | INT | FLOAT | "true" | "false" ;
LiteralList := "[" [ Literal { "," Literal } ] "]" ;
TypeExpr    := TypeAtom | TypeConstruct ;
TypeAtom    := "bool" | "int" | "float" | "text" | "url" | "email" | "path" | "bytes" | "json"
             | "openapi" | "markdown" | "xlsx" | "pdf" ;
TypeConstruct := "object" "{" { IDENT ":" TypeExpr } "}"
               | "list" "[" TypeExpr "]"
               | "enum" "(" String { "," String } ")"
               | "optional" "[" TypeExpr "]" ;
```

> Notes:
> - The document examples use both inline and block-style sections (e.g., `constraints:` followed by indented lines).  
> - `|>` is reserved; MVP may implement only `->`.

### 5.2 Required sections
MVP requires at minimum:
- `goal`
- `input`
- `run`

`checks` and `constraints` are strongly recommended, and policies may enforce them.

---

## 6. Type System (Draft, MVP)

### 6.1 Primitive types
- `bool`, `int`, `float`, `text`, `url`, `email`, `path`, `bytes`, `json`

### 6.2 Structured types
- `object { key: type }`
- `list[type]`
- `enum("A","B")`
- `optional[type]`

### 6.3 Domain types
- `openapi`, `markdown`, `xlsx`, `pdf`  
  (domain types are primarily used as intermediate artifacts and output targets)

### 6.4 Type checking rules
- Each `input` binding has a declared type.
- Calls and checks have expected argument shapes; mismatches are compile-time errors.
- `optional[T]` values must be explicitly handled by checks/policy (policy-defined).

---

## 7. Semantics

### 7.1 Declarative task semantics
A task describes **what** must be achieved (goal, checks) and **how** it is executed (pipeline steps), not arbitrary computation.

### 7.2 Bounded execution
- Unbounded loops are forbidden.
- Iteration is permitted only as bounded maps over explicit collections (host-provided or explicitly enumerated).
- Runtime may perform bounded repairs depending on policy (`max_repairs`, default/typical is 2).

### 7.3 Checks and validations
Checks are declarative predicates executed against intermediate artifacts and/or final outputs.

Example forms:
```intentscript
checks:
  must_have_sections ["Overview","Setup","Examples"]
  must_not_contain ["TODO","WIP"]
  validate(output against output_schema)
max_repairs = 2
```

Failures:
- produce diagnostics (with source locations when possible),
- and MAY trigger bounded repair passes depending on policy.

### 7.4 Constraints
Constraints define capability toggles and policy rules.

Example:
```intentscript
constraints:
  net = off
  forbid ambiguous_terms
```

---

## 8. Diagnostics

### 8.1 Requirements
Diagnostics MUST be:
- precise (line/column),
- actionable,
- stable in CI (machine-readable JSON format is supported by CLI).

### 8.2 Diagnostic categories
- Parse errors: syntax, unexpected tokens
- Static analysis: missing sections, unknown identifiers, type mismatches
- Contradictions/ambiguity: constraint solver findings
- Policy violations: forbidden patterns, disabled capabilities, etc.

---

## 9. Compiler Pipeline

### 9.1 Stages
| Stage | Input | Output | Notes |
|---|---|---|---|
| Parse | IntentScript source | AST | Precise diagnostics; preserves code blocks |
| Analyze | AST | Typed AST + symbols | Type checking, contradiction detection, policy checks |
| Lower | Typed AST | IR (Execution Plan) | Deterministic, semver-versioned, serializable |
| Execute | IR + inputs + host | Artifacts | State machine; bounded repairs; audit log |

### 9.2 Static analysis (MVP)
At minimum:
- missing required sections,
- unknown identifiers,
- type mismatches.

### 9.3 Constraint solving (MVP)
- Detect contradictions and ambiguous constructs (policy-driven).
- Contradictions are compile-time errors unless policy allows resolution.

---

## 10. IR: Execution Plan (Contract)

### 10.1 Purpose
IR is the contractual interface between compiler and runtime. It is stable, reviewable, suitable for version control, and sufficient to run without re-parsing source in CI.

### 10.2 Core shape (structural contract)
```text
ExecutionPlan {
  meta: { task_name, version, compiler_version, policy_hash },
  inputs: [InputSpec],
  steps: [Step { id, kind, args, produces, checks }],
  outputs: [ArtifactSpec],
  limits: { max_repairs, timeout_ms },
  capabilities: { fs, net, exec, templates, exports }
}
```

### 10.3 IR JSON-ish schema (draft)
```json
{
  "schema_version": "1.0",
  "meta": {
    "task_name": "RustApiOpenApiLint",
    "task_version": "1.0",
    "compiler_version": "0.1.0",
    "policy_hash": "sha256:..."
  },
  "inputs": [
    { "name": "openapi_file", "type": "path", "required": true }
  ],
  "capabilities": {
    "fs": { "read_roots": ["./"], "write_roots": ["./artifacts/"] },
    "net": false,
    "exec": false,
    "templates": true,
    "exports": true
  },
  "limits": { "max_repairs": 2, "timeout_ms": 60000 },
  "steps": [
    { "id": "s1", "kind": "read_file", "args": {"path_ref":"openapi_file"}, "produces":"bytes:openapi" },
    { "id": "s2", "kind": "parse_openapi", "args": {"bytes_ref":"s1"}, "produces":"openapi:doc" },
    { "id": "s3", "kind": "validate", "args": {"doc_ref":"s2"}, "checks":[{"name":"must_include_paths_prefix","args":{"prefix":"/api"}}] },
    { "id": "s4", "kind": "report", "args": {"format":"markdown", "out":"./artifacts/openapi_lint.md"} }
  ],
  "outputs": [
    { "path": "./artifacts/openapi_lint.md", "type": "markdown" }
  ]
}
```

### 10.4 Versioning and migrations
- IR schema MUST be semver-versioned.
- Changes SHOULD be migration-friendly; policy hash and compiler version MUST be embedded.

---

## 11. Runtime Model

### 11.1 State machine
Runtime executes steps as a deterministic state machine:
`plan -> generate -> validate -> repair -> finalize` (lifecycle is task/policy dependent).

### 11.2 Host interface (capability-based)
The runtime core executes no direct side effects; it delegates to the Host:

```rust
pub trait Host {
  fn read(&self, p: &str) -> Result<Vec<u8>>;
  fn write(&self, p: &str, bytes: &[u8]) -> Result<()>;
  fn render_template(&self, name: &str, vars: serde_json::Value) -> Result<String>;
  fn export_xlsx(&self, spec: XlsxSpec, rows: Vec<Row>) -> Result<Vec<u8>>;
  fn parse_openapi(&self, bytes: &[u8]) -> Result<OpenApiDoc>;
}
```

> This snippet is a reference shape from the PRD; exact signatures may evolve, but the **capability gating** and **trait-based boundary** are normative.

### 11.3 Audit log
All effects MUST be recorded in an append-only log suitable for CI review.

---

## 12. CLI (Developer UX)

### 12.1 Commands (MVP)
- `intentscript build` — compile source to IR
- `intentscript run` — execute IR with a Host
- `intentscript lint` — static checks and policy checks
- `intentscript fmt` — formatting (best-effort in MVP)
- `intentscript explain` — explain plan/steps and why checks passed/failed

### 12.2 CI output
CLI SHOULD support machine-readable JSON diagnostics and stable exit codes.

---

## 13. Embedding (Optional / Future)

### 13.1 Rust macro embed (P2)
An optional embedding may exist:
- `intentscript!(...)` with compile-time validation.

MVP may ship file-based tasks only.

---

## 14. Reference Workflows (Examples)

### 14.1 OpenAPI Lint (policy gate)
```intentscript
task "RustApiOpenApiLint" v1.0 {
  goal: enforce openapi_policy(name:"rustapi-default")
  input: openapi_file: path
  constraints:
    net = off
    forbid ambiguous_terms
  checks:
    must_include_paths_prefix("/api")
    must_use_uuid_format_for_params(["id","uuid"])
    must_have_security_schemes(["bearerAuth"])
  run:
    read(openapi_file) -> parse_openapi -> validate -> report(format:"markdown")
}
```

### 14.2 Cookbook Validation
```intentscript
task "RustApiCookbookCheck" v1.0 {
  goal: validate_docs(scope:"cookbook")
  input: docs_root: path = "./cookbook"
  checks:
    must_have_sections ["Overview","Setup","Examples"]
    must_not_contain ["TODO","WIP"]
  run:
    scan(docs_root, pattern:"**/*.md") -> validate -> report(format:"json")
}
```

---

## 15. Requirements Matrix (Normative)

### 15.1 Functional requirements
| ID | Requirement | Priority |
|---|---|---|
| FR-01 | Parse IntentScript source into AST with precise error reporting (line/column). | P0 |
| FR-02 | Static analysis: missing sections, unknown identifiers, type mismatches. | P0 |
| FR-03 | Constraint solver: detect contradictions and ambiguous constructs (policy-driven). | P0 |
| FR-04 | Lower to deterministic IR (Execution Plan) with semantic versioning. | P0 |
| FR-05 | Runtime executes IR with lifecycle: plan -> generate -> validate -> repair -> finalize. | P0 |
| FR-06 | Schema validation for outputs (markdown, json, xlsx, openapi, etc.). | P0 |
| FR-07 | Host adapter interface for effectful operations (file IO, templates, exports, optional web). | P0 |
| FR-08 | CLI: build/run/lint/fmt/explain; supports CI JSON diagnostics. | P1 |
| FR-09 | Rust macro embed option: intentscript!(...) with compile-time validation. | P2 |

### 15.2 Non-functional requirements
| ID | Requirement | Priority |
|---|---|---|
| NFR-01 | Deterministic execution by default (pinned adapters, seedable transforms). | P0 |
| NFR-02 | Sandboxable runtime: capability gates network, filesystem, external commands. | P0 |
| NFR-03 | Human-grade diagnostics: actionable errors and lints. | P0 |
| NFR-04 | CI-first: stable and fast compile (< 2s typical task). | P1 |
| NFR-05 | Extensibility without compromising core safety (capabilities model). | P1 |

---

## 16. Suggested Crate Layout (Reference Implementation)

Keep compiler and runtime separate. Runtime accepts only IR + Host; enables CI execution and embedding.

| Crate | Responsibility | Public API surface (draft) |
|---|---|---|
| intentscript-core | AST/IR types, serde, policy structs | `ast::*`, `ir::*`, `policy::*` |
| intentscript-parser | Source -> AST | `parse_str()`, `parse_file()` |
| intentscript-compiler | AST -> IR, semantic checks | `compile(ast, policy) -> plan` |
| intentscript-runtime | Execute plan | `execute(plan, inputs, host) -> result` |
| intentscript-cli | Developer UX | `build/run/lint/fmt/explain` |
| intentscript-rustapi | RustAPI adapters | `openapi_lint::*`, `cookbook::*`, `scaffold::*` |

---

## 17. Open Questions (Non-normative)
- What OpenAPI generator format does RustAPI emit today (JSON/YAML), and where is it produced in the build?
- Where should tasks live in RustAPI repo: `/tasks`, `/tools/intentscript`, or a separate repo?
- Should MVP include macro-embed (proc_macro) or keep DSL files only?
- Which output targets matter most to RustAPI users: markdown, json, xlsx, code scaffolds?
- What is the compatibility policy for IR schema changes (semver + migrations)?

---

## 18. Changelog (Spec)
- v0.1 (2026-01-27): Initial MVP spec authored from PRD+TRD v1 (2026-01-26).
