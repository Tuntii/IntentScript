# Getting Started with IntentScript

This guide will help you write your first IntentScript task and understand the core concepts.

## Installation

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/intentscript.git
cd intentscript

# Build the project
cargo build --release

# The binary will be at target/release/intentscript
```

### Installing the CLI

```bash
# Install globally
cargo install --path intentscript-cli

# Verify installation
intentscript --version
```

## Your First Task

Let's create a simple task that validates a text file.

### Step 1: Create a Task File

Create a file named `validate.intent`:

```intentscript
task "FileValidator" v1.0 {
  goal: "Validate that a file is not empty"
  
  input: file_path: path
  
  constraints: {
    net = off
    fs_read_roots = ["./"]
  }
  
  output_schema: text
  
  checks: {
    must_not_be_empty
  }
  
  run:
    read_file(file_path) ->
    validate ->
    report(format: "text")
}
```

### Step 2: Understand the Structure

Let's break down each section:

#### Task Declaration
```intentscript
task "FileValidator" v1.0 {
```
- `task`: Keyword to start a task definition
- `"FileValidator"`: Task name (must be unique)
- `v1.0`: Version number (semantic versioning)

#### Goal
```intentscript
goal: "Validate that a file is not empty"
```
Describes what the task aims to achieve. This is documentation for humans and tools.

#### Input
```intentscript
input: file_path: path
```
Declares required inputs with their types:
- `file_path`: Input name
- `path`: Type (represents a filesystem path)

#### Constraints
```intentscript
constraints: {
  net = off
  fs_read_roots = ["./"]
}
```
Defines capability constraints:
- `net = off`: Disable network access
- `fs_read_roots`: Allow reading files from current directory

#### Output Schema
```intentscript
output_schema: text
```
Declares the expected output type.

#### Checks
```intentscript
checks: {
  must_not_be_empty
}
```
Validation predicates that must pass for the task to succeed.

#### Run Pipeline
```intentscript
run:
  read_file(file_path) ->
  validate ->
  report(format: "text")
```
The execution pipeline:
1. Read the file
2. Validate it
3. Generate a report

### Step 3: Compile the Task

```bash
intentscript build validate.intent -o validate.ir.json
```

This produces an Execution Plan (IR) in JSON format. The IR is:
- Deterministic (same source = same IR)
- Reviewable (human-readable JSON)
- Executable (can run without source)

### Step 4: Run the Task

Create a test file:
```bash
echo "Hello, IntentScript!" > test.txt
```

Execute the task:
```bash
intentscript run validate.ir.json --input file_path=test.txt
```

You should see output indicating the validation passed.

### Step 5: Test with Invalid Input

Try with an empty file:
```bash
touch empty.txt
intentscript run validate.ir.json --input file_path=empty.txt
```

The task should fail with a validation error.

## Understanding Types

IntentScript has a rich type system:

### Primitive Types

```intentscript
input: {
  name: text           // UTF-8 string
  age: int             // Integer number
  price: float         // Floating-point number
  active: bool         // Boolean (true/false)
  website: url         // URL
  contact: email       // Email address
  config_file: path    // Filesystem path
  data: bytes          // Raw bytes
  metadata: json       // JSON data
}
```

### Structured Types

```intentscript
// Object with fields
input: user: object {
  name: text
  age: int
  email: email
}

// List of items
input: tags: list[text]

// Enumeration
input: status: enum("draft", "published", "archived")

// Optional value
input: description: optional[text]
```

### Domain Types

```intentscript
input: {
  api_spec: openapi      // OpenAPI specification
  docs: markdown         // Markdown document
  spreadsheet: xlsx      // Excel file
  report: pdf            // PDF document
}
```

## Working with Inputs

### Required Inputs

```intentscript
input: name: text
```

Must be provided when running the task.

### Optional Inputs with Defaults

```intentscript
input: {
  name: text = "World"
  count: int = 10
  enabled: bool = true
}
```

Can be omitted; will use default value.

### Multiple Inputs

```intentscript
input: {
  source_file: path
  destination: path
  overwrite: bool = false
}
```

Use block syntax for multiple inputs.

## Understanding Constraints

Constraints control what your task can do:

### Filesystem Access

```intentscript
constraints: {
  fs_read_roots = ["./data", "./config"]
  fs_write_roots = ["./output"]
}
```

### Network Access

```intentscript
constraints: {
  net = on  // Enable network (off by default)
}
```

### Templates

```intentscript
constraints: {
  templates = on  // Enable template rendering
}
```

### Exports

```intentscript
constraints: {
  exports = on  // Enable XLSX, PDF exports
}
```

### External Commands

```intentscript
constraints: {
  exec = on  // Enable external command execution
}
```

## Writing Validation Checks

Checks are predicates that validate your data:

### Common Checks

```intentscript
checks: {
  must_not_be_empty
  must_be_valid_utf8
  must_have_sections(["Introduction", "Conclusion"])
  must_not_contain(["TODO", "FIXME"])
  validate_links
  validate_json_schema
}
```

### Custom Checks

```intentscript
checks: {
  must_include_paths_prefix("/api")
  must_use_uuid_format_for_params(["id", "user_id"])
  must_have_security_schemes(["bearerAuth"])
}
```

## Building Pipelines

Pipelines chain operations together:

### Basic Pipeline

```intentscript
run:
  read_file(input_file) ->
  parse_json ->
  validate ->
  report(format: "text")
```

### Pipeline with Arguments

```intentscript
run:
  read_file(source) ->
  transform(operation: "uppercase") ->
  write_file(destination) ->
  report(format: "json", output: "./report.json")
```

### Complex Pipeline

```intentscript
run:
  scan(directory, pattern: "**/*.md") ->
  parse_markdown ->
  validate ->
  render_template("summary", vars: {count: total}) ->
  write_file("./summary.md") ->
  report(format: "markdown")
```

## CLI Workflow

### Development Workflow

```bash
# 1. Write your task
vim my_task.intent

# 2. Check for errors
intentscript lint my_task.intent

# 3. Format the code
intentscript fmt my_task.intent

# 4. Compile to IR
intentscript build my_task.intent -o my_task.ir.json

# 5. Explain the execution plan
intentscript explain my_task.ir.json

# 6. Run the task
intentscript run my_task.ir.json --input key=value
```

### CI/CD Integration

```bash
# Lint in CI (exits with error code on failure)
intentscript lint tasks/*.intent

# Build with JSON diagnostics
intentscript build task.intent --json > diagnostics.json

# Run with timeout
intentscript run task.ir.json --timeout 30000
```

## Next Steps

Now that you understand the basics:

1. **Explore Examples**: Check out [examples/](../examples/) for more complex tasks
2. **Read the Language Reference**: See [language-reference.md](language-reference.md) for complete syntax
3. **Learn CLI Commands**: See [cli-reference.md](cli-reference.md) for all commands
4. **Create Custom Hosts**: See [host-trait.md](host-trait.md) for extensibility

## Common Patterns

### File Processing

```intentscript
task "ProcessFile" v1.0 {
  goal: "Process a file"
  input: {
    input_file: path
    output_file: path
  }
  constraints: {
    net = off
    fs_read_roots = ["./input"]
    fs_write_roots = ["./output"]
  }
  output_schema: path
  checks: {
    validate_output
  }
  run:
    read_file(input_file) ->
    transform ->
    write_file(output_file) ->
    report(format: "text")
}
```

### API Validation

```intentscript
task "ValidateAPI" v1.0 {
  goal: "Validate API specification"
  input: spec_file: path
  constraints: {
    net = off
  }
  checks: {
    must_have_security
    validate_schemas
  }
  run:
    read_file(spec_file) ->
    parse_openapi ->
    validate ->
    report(format: "markdown")
}
```

### Documentation Generation

```intentscript
task "GenerateDocs" v1.0 {
  goal: "Generate documentation"
  input: {
    source_dir: path
    output_dir: path
  }
  constraints: {
    templates = on
    fs_read_roots = ["./src"]
    fs_write_roots = ["./docs"]
  }
  output_schema: object {
    index: path
    pages: list[path]
  }
  checks: {
    validate_links
  }
  run:
    scan(source_dir, pattern: "**/*.md") ->
    parse_markdown ->
    render_template("docs") ->
    write_files(output_dir) ->
    report(format: "json")
}
```

## Troubleshooting

### Common Errors

**Parse Error: Unexpected token**
- Check syntax carefully
- Ensure all braces and parentheses are balanced
- Use `intentscript lint` to see precise error location

**Type Mismatch**
- Verify input types match expected types
- Check pipeline step compatibility
- Review type annotations

**Capability Violation**
- Ensure required capabilities are enabled in constraints
- Check read/write roots for filesystem access
- Verify network access is enabled if needed

**Validation Failure**
- Review check predicates
- Examine the audit log for details
- Use `intentscript explain` to understand the execution plan

### Getting Help

- Check the [Language Reference](language-reference.md)
- Review [Examples](../examples/)
- Read error messages carefully (they include line/column numbers)
- Use `intentscript explain` to understand execution plans

## Summary

You've learned:
- ✅ How to create a basic IntentScript task
- ✅ Understanding task structure and sections
- ✅ Working with types and inputs
- ✅ Setting capability constraints
- ✅ Writing validation checks
- ✅ Building execution pipelines
- ✅ Using the CLI tools

Continue to the [Language Reference](language-reference.md) for complete syntax details.
