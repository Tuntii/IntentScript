# IntentScript Language Reference

Complete reference for IntentScript syntax and semantics.

## Table of Contents

1. [Lexical Structure](#lexical-structure)
2. [Task Structure](#task-structure)
3. [Type System](#type-system)
4. [Expressions](#expressions)
5. [Constraints](#constraints)
6. [Validation Checks](#validation-checks)
7. [Pipelines](#pipelines)
8. [Semantics](#semantics)

## Lexical Structure

### Comments

Line comments start with `//` and continue to end of line:

```intentscript
// This is a comment
task "Example" v1.0 {  // Inline comment
  goal: "Demonstrate comments"
  run: step1  // Another comment
}
```

Block comments are reserved for future versions.

### Identifiers

Identifiers match the pattern `[A-Za-z_][A-Za-z0-9_]*`:

```intentscript
valid_identifier
_private
CamelCase
snake_case
CONSTANT
```

### Keywords

Reserved keywords:

```
task        goal        input       constraints
output_schema   checks      run         on
off         true        false       bool
int         float       text        url
email       path        bytes       json
openapi     markdown    xlsx        pdf
object      list        enum        optional
```

### String Literals

Double-quoted UTF-8 strings with escape sequences:

```intentscript
"simple string"
"string with\nnewline"
"string with\ttab"
"string with \"quotes\""
"string with\\backslash"
```

### Numeric Literals

Integers and floating-point numbers:

```intentscript
42          // Integer
3.14        // Float
0           // Zero
1000000     // Large number
0.001       // Small float
```

### Version Literals

Semantic version numbers:

```intentscript
v1.0
v1.2.3
v2.0.0
```

## Task Structure

### Task Declaration

```intentscript
task <name> [version] {
  <sections>
}
```

Example:

```intentscript
task "MyTask" v1.0 {
  goal: "Task description"
  input: param: type
  constraints: { net = off }
  output_schema: type
  checks: { validation }
  run: pipeline
}
```

### Required Sections

Minimum required sections:
- `goal`
- `input` (can be empty)
- `run`

### Optional Sections

- `constraints`
- `output_schema`
- `checks`

### Section Order

Sections can appear in any order, but conventional order is:
1. `goal`
2. `input`
3. `constraints`
4. `output_schema`
5. `checks`
6. `run`

## Type System

### Primitive Types

#### `bool`
Boolean value: `true` or `false`

```intentscript
input: enabled: bool = true
```

#### `int`
Integer number

```intentscript
input: count: int = 10
```

#### `float`
Floating-point number

```intentscript
input: price: float = 19.99
```

#### `text`
UTF-8 string

```intentscript
input: name: text = "default"
```

#### `url`
URL string

```intentscript
input: endpoint: url = "https://api.example.com"
```

#### `email`
Email address string

```intentscript
input: contact: email
```

#### `path`
Filesystem path

```intentscript
input: file: path
```

#### `bytes`
Raw byte array

```intentscript
input: data: bytes
```

#### `json`
JSON data

```intentscript
input: config: json
```

### Structured Types

#### `object`
Object with named fields

```intentscript
input: user: object {
  name: text
  age: int
  email: email
}
```

Nested objects:

```intentscript
input: config: object {
  database: object {
    host: text
    port: int
  }
  cache: object {
    enabled: bool
    ttl: int
  }
}
```

#### `list`
Ordered collection of items

```intentscript
input: tags: list[text]
input: numbers: list[int]
input: users: list[object {
  name: text
  email: email
}]
```

#### `enum`
Enumeration of string values

```intentscript
input: status: enum("draft", "published", "archived")
input: level: enum("debug", "info", "warn", "error")
```

#### `optional`
Optional value (may be absent)

```intentscript
input: description: optional[text]
input: metadata: optional[json]
```

### Domain Types

#### `openapi`
OpenAPI specification document

```intentscript
input: api_spec: openapi
```

#### `markdown`
Markdown document

```intentscript
input: readme: markdown
```

#### `xlsx`
Excel spreadsheet

```intentscript
input: data: xlsx
```

#### `pdf`
PDF document

```intentscript
input: report: pdf
```

### Type Compatibility

Type checking rules:

1. **Exact match**: Types must match exactly
2. **Optional handling**: `optional[T]` requires explicit handling
3. **Subtyping**: No implicit conversions
4. **Domain types**: Treated as opaque values

## Expressions

### Literals

```intentscript
"string"
42
3.14
true
false
```

### Identifiers

Reference inputs or variables:

```intentscript
input_name
variable_name
```

### Function Calls

```intentscript
function_name()
function_name(arg1, arg2)
function_name(key1: value1, key2: value2)
```

Named arguments:

```intentscript
render_template(name: "template", vars: config)
```

Positional arguments:

```intentscript
format_string("Hello, %s", name)
```

Mixed arguments (positional first):

```intentscript
process(input_file, output: output_file, format: "json")
```

## Constraints

### Capability Constraints

#### Filesystem Access

```intentscript
constraints: {
  fs_read_roots = ["./data", "./config"]
  fs_write_roots = ["./output", "./artifacts"]
}
```

#### Network Access

```intentscript
constraints: {
  net = on   // Enable network (off by default)
}
```

#### Template Rendering

```intentscript
constraints: {
  templates = on
}
```

#### Exports

```intentscript
constraints: {
  exports = on  // Enable XLSX, PDF exports
}
```

#### External Commands

```intentscript
constraints: {
  exec = on
}
```

### Policy Constraints

```intentscript
constraints: {
  forbid_ambiguous_terms = on
  require_security_schemes = on
  max_repairs = 3
  timeout_ms = 60000
}
```

### Inline vs Block Syntax

Inline (single constraint):

```intentscript
constraints: net = off
```

Block (multiple constraints):

```intentscript
constraints: {
  net = off
  templates = on
  fs_read_roots = ["./"]
}
```

## Validation Checks

### Predefined Checks

#### `must_not_be_empty`
Validates that content is not empty

```intentscript
checks: {
  must_not_be_empty
}
```

#### `must_be_valid_utf8`
Validates UTF-8 encoding

```intentscript
checks: {
  must_be_valid_utf8
}
```

#### `must_have_sections`
Validates presence of sections

```intentscript
checks: {
  must_have_sections(["Introduction", "Conclusion"])
}
```

#### `must_not_contain`
Validates absence of patterns

```intentscript
checks: {
  must_not_contain(["TODO", "FIXME", "WIP"])
}
```

#### `must_include_paths_prefix`
Validates path prefixes

```intentscript
checks: {
  must_include_paths_prefix("/api")
}
```

#### `must_use_uuid_format_for_params`
Validates UUID format for parameters

```intentscript
checks: {
  must_use_uuid_format_for_params(["id", "user_id"])
}
```

#### `must_have_security_schemes`
Validates security schemes

```intentscript
checks: {
  must_have_security_schemes(["bearerAuth", "apiKey"])
}
```

#### `validate_links`
Validates all links in document

```intentscript
checks: {
  validate_links
}
```

#### `validate_json_schema`
Validates against JSON schema

```intentscript
checks: {
  validate_json_schema
}
```

#### `validate`
Generic validation against output schema

```intentscript
checks: {
  validate
}
```

### Custom Checks

Define custom validation predicates:

```intentscript
checks: {
  custom_check(arg1: value1, arg2: value2)
}
```

### Check Timing

Checks can run:
- **During execution**: On intermediate artifacts
- **After execution**: On final outputs
- **Both**: Specified in IR

## Pipelines

### Basic Pipeline

Chain steps with `->`:

```intentscript
run:
  step1 ->
  step2 ->
  step3
```

### Steps

#### Function Calls

```intentscript
run:
  read_file(input_path) ->
  parse_json ->
  validate
```

#### Identifiers

```intentscript
run:
  load_data ->
  transform ->
  save_results
```

### Step Arguments

Named arguments:

```intentscript
run:
  read_file(path: input_file) ->
  transform(operation: "uppercase") ->
  write_file(path: output_file)
```

Positional arguments:

```intentscript
run:
  format_string("Hello, %s", name) ->
  write_file(output_path)
```

### Data Flow

Data flows implicitly through pipeline:

```intentscript
run:
  read_file(input) ->      // Produces bytes
  parse_json ->            // Consumes bytes, produces json
  validate ->              // Consumes json
  report(format: "text")   // Produces text report
```

### Built-in Steps

#### File Operations

```intentscript
read_file(path)
write_file(path)
scan(directory, pattern: "**/*.md")
```

#### Parsing

```intentscript
parse_json
parse_openapi
parse_markdown
```

#### Validation

```intentscript
validate
validate_schema
```

#### Rendering

```intentscript
render_template(name, vars: {...})
```

#### Exports

```intentscript
export_xlsx(sheet: name, headers: true, output: path)
export_pdf(output: path)
```

#### Reporting

```intentscript
report(format: "text")
report(format: "json", output: path)
report(format: "markdown", output: path)
```

## Semantics

### Declarative Execution

Tasks describe **what** to achieve, not **how**:

```intentscript
task "Example" v1.0 {
  goal: "Validate API specification"
  // Declares intent, not implementation
  run: read_file(spec) -> parse_openapi -> validate
}
```

### Determinism

Same inputs always produce same outputs:

1. **Compilation**: Same source → same IR
2. **Execution**: Same IR + inputs → same outputs
3. **Audit**: Complete log of all operations

### Bounded Execution

No unbounded loops:

```intentscript
// ❌ Not allowed: unbounded loop
while (condition) { ... }

// ✅ Allowed: bounded iteration
map(items, transform)
```

Iteration only over explicit collections:

```intentscript
run:
  scan(directory, pattern: "*.md") ->  // Bounded by filesystem
  map(parse_markdown) ->                // Bounded by input
  validate
```

### Capability-Based Security

All side effects require explicit capabilities:

```intentscript
constraints: {
  fs_read_roots = ["./data"]   // Can only read from ./data
  fs_write_roots = ["./output"] // Can only write to ./output
  net = off                     // No network access
}
```

Violations are runtime errors:

```intentscript
// ❌ Error: Cannot read from ./secrets (not in fs_read_roots)
run: read_file("./secrets/key.txt")
```

### Validation and Repair

Checks validate outputs:

```intentscript
checks: {
  must_have_sections(["Introduction"])
}
```

Failed checks trigger bounded repair:

1. Execute pipeline
2. Run checks
3. If failed and repairs < max_repairs:
   - Attempt repair
   - Go to step 2
4. If failed and repairs >= max_repairs:
   - Report failure

### Type Checking

Static type checking at compile time:

```intentscript
input: count: int

// ❌ Error: Type mismatch
run: format_string(count)  // Expects text, got int
```

Optional types require explicit handling:

```intentscript
input: description: optional[text]

// ❌ Error: Optional not handled
run: format_string(description)

// ✅ Correct: Handle optional
run: format_string(description or "No description")
```

### Error Handling

Errors are reported with precise locations:

```
error[E0042]: type mismatch
  --> task.intent:12:15
   |
12 |   run: format_string(count)
   |                      ^^^^^ expected text, found int
   |
help: convert to text: format_string(to_text(count))
```

Categories:
- **Parse errors**: Syntax issues
- **Type errors**: Type mismatches
- **Semantic errors**: Constraint violations
- **Runtime errors**: Execution failures

## Grammar (EBNF)

```ebnf
File        ::= Task*

Task        ::= "task" String Version? "{" Section* "}"
Version     ::= "v" Number ("." Number)*

Section     ::= GoalSection
              | InputSection
              | ConstraintsSection
              | OutputSchemaSection
              | ChecksSection
              | RunSection

GoalSection         ::= "goal" ":" Expr
InputSection        ::= "input" ":" (InputDecl | "{" InputDecl* "}")
ConstraintsSection  ::= "constraints" ":" (ConstraintDecl | "{" ConstraintDecl* "}")
OutputSchemaSection ::= "output_schema" ":" TypeExpr
ChecksSection       ::= "checks" ":" (CheckDecl | "{" CheckDecl* "}")
RunSection          ::= "run" ":" Pipeline

InputDecl      ::= Ident ":" TypeExpr ("=" Literal)?
ConstraintDecl ::= Ident "=" ("on" | "off" | Literal | Expr)
CheckDecl      ::= CallExpr | Ident

Pipeline ::= Step ("->" Step)*
Step     ::= CallExpr | Ident

Expr     ::= CallExpr | Ident | Literal
CallExpr ::= Ident "(" ArgList? ")"
ArgList  ::= Arg ("," Arg)*
Arg      ::= (Ident ":")? Expr

Literal ::= String | Number | "true" | "false"

TypeExpr ::= PrimitiveType
           | "object" "{" (Ident ":" TypeExpr)* "}"
           | "list" "[" TypeExpr "]"
           | "enum" "(" String ("," String)* ")"
           | "optional" "[" TypeExpr "]"
           | DomainType

PrimitiveType ::= "bool" | "int" | "float" | "text"
                | "url" | "email" | "path" | "bytes" | "json"

DomainType ::= "openapi" | "markdown" | "xlsx" | "pdf"

Ident  ::= [A-Za-z_][A-Za-z0-9_]*
String ::= '"' ([^"\\] | '\\' .)* '"'
Number ::= [0-9]+ ("." [0-9]+)?
```

## Best Practices

### 1. Use Descriptive Names

```intentscript
// ❌ Bad
task "T1" v1.0 { ... }

// ✅ Good
task "ValidateOpenAPISpec" v1.0 { ... }
```

### 2. Document Goals Clearly

```intentscript
// ❌ Bad
goal: "Do stuff"

// ✅ Good
goal: "Validate OpenAPI specification against RustAPI policies"
```

### 3. Use Least Privilege

```intentscript
// ❌ Bad: Too permissive
constraints: {
  fs_read_roots = ["/"]
  net = on
}

// ✅ Good: Minimal permissions
constraints: {
  fs_read_roots = ["./data"]
  net = off
}
```

### 4. Provide Defaults

```intentscript
// ❌ Bad: No defaults
input: {
  timeout: int
  retries: int
}

// ✅ Good: Sensible defaults
input: {
  timeout: int = 30000
  retries: int = 3
}
```

### 5. Use Specific Types

```intentscript
// ❌ Bad: Too generic
input: config: text

// ✅ Good: Specific type
input: config: json
```

### 6. Add Comprehensive Checks

```intentscript
// ❌ Bad: No validation
checks: {}

// ✅ Good: Multiple checks
checks: {
  must_not_be_empty
  validate_schema
  must_have_required_fields
}
```

## See Also

- [Getting Started Guide](getting-started.md)
- [CLI Reference](cli-reference.md)
- [Host Trait Guide](host-trait.md)
- [Examples](../examples/)
