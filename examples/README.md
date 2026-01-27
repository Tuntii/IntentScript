# IntentScript Examples

This directory contains example IntentScript tasks demonstrating various features of the language.

## Examples Overview

### 1. Simple Validation (`simple_validation.intent`)
**Difficulty:** Beginner  
**Features:** Basic file reading, simple validation checks

A minimal example that validates a text file. Great starting point for learning IntentScript.

**Usage:**
```bash
intentscript build examples/simple_validation.intent -o simple_validation.ir.json
intentscript run simple_validation.ir.json --input file_path=./README.md
```

### 2. OpenAPI Lint (`openapi_lint.intent`)
**Difficulty:** Intermediate  
**Features:** Domain-specific parsing, multiple checks, capability constraints

Validates an OpenAPI specification against RustAPI policies. Demonstrates how to enforce API design standards.

**Usage:**
```bash
intentscript build examples/openapi_lint.intent -o openapi_lint.ir.json
intentscript run openapi_lint.ir.json --input openapi_file=./api/openapi.yaml
```

### 3. Cookbook Validation (`cookbook_validation.intent`)
**Difficulty:** Intermediate  
**Features:** Directory scanning, markdown parsing, default values

Validates documentation in a cookbook directory, ensuring quality standards are met.

**Usage:**
```bash
intentscript build examples/cookbook_validation.intent -o cookbook_validation.ir.json
intentscript run cookbook_validation.ir.json
# Uses default docs_root="./cookbook"
```

### 4. Project Scaffolding (`project_scaffold.intent`)
**Difficulty:** Advanced  
**Features:** Template rendering, multiple inputs, file writing, enum types

Generates a new RustAPI project from templates with customizable options.

**Usage:**
```bash
intentscript build examples/project_scaffold.intent -o project_scaffold.ir.json
intentscript run project_scaffold.ir.json \
  --input project_name="my-api" \
  --input author="Your Name" \
  --input license="MIT" \
  --input include_auth=true
```

### 5. Data Export (`data_export.intent`)
**Difficulty:** Intermediate  
**Features:** JSON parsing, XLSX export, data transformation

Converts JSON data to Excel spreadsheets with formatting.

**Usage:**
```bash
intentscript build examples/data_export.intent -o data_export.ir.json
intentscript run data_export.ir.json --input data_file=./data/records.json
```

### 6. API Documentation (`api_documentation.intent`)
**Difficulty:** Advanced  
**Features:** OpenAPI parsing, template rendering, optional types, multiple formats

Generates comprehensive API documentation from OpenAPI specifications.

**Usage:**
```bash
intentscript build examples/api_documentation.intent -o api_docs.ir.json
intentscript run api_docs.ir.json \
  --input openapi_spec=./specs/api.yaml \
  --input output_format="markdown" \
  --input include_examples=true
```

## Key Concepts Demonstrated

### Task Structure
Every IntentScript task has:
- **goal**: What the task aims to achieve
- **input**: Required and optional inputs with types
- **constraints**: Capability and policy constraints
- **output_schema**: Expected output type
- **checks**: Validation predicates
- **run**: Execution pipeline

### Type System
Examples demonstrate:
- Primitive types: `text`, `path`, `bool`, `int`, `float`
- Structured types: `object`, `list`, `enum`, `optional`
- Domain types: `openapi`, `markdown`, `json`, `xlsx`, `pdf`

### Capabilities
Tasks declare required capabilities:
- `fs`: Filesystem access (read/write roots)
- `net`: Network access (off by default)
- `templates`: Template rendering
- `exports`: Export to structured formats (XLSX, PDF)
- `exec`: External command execution

### Pipeline Operators
- `->`: Chain steps in sequence
- Each step transforms data and passes it to the next

### Validation Checks
Declarative predicates that validate:
- Structure: `must_have_sections`, `must_include_paths_prefix`
- Content: `must_not_contain`, `validate_links`
- Schema: `validate_json_schema`, `validate_markdown_links`

## Learning Path

1. Start with `simple_validation.intent` to understand basic structure
2. Move to `openapi_lint.intent` to see domain-specific features
3. Explore `cookbook_validation.intent` for directory operations
4. Study `data_export.intent` for data transformation
5. Review `api_documentation.intent` for complex template rendering
6. Finally, examine `project_scaffold.intent` for advanced features

## Creating Your Own Tasks

To create a new task:

1. Define your goal clearly
2. Identify required inputs and their types
3. Specify capability constraints (principle of least privilege)
4. Define validation checks
5. Design the execution pipeline
6. Test with `intentscript lint` before building

Example template:
```intentscript
task "MyTask" v1.0 {
  goal: "Description of what this task does"
  
  input: {
    my_input: type
  }
  
  constraints: {
    net = off
    // Add required capabilities
  }
  
  output_schema: type
  
  checks: {
    // Add validation checks
  }
  
  run:
    step1 -> step2 -> step3
}
```

## Additional Resources

- [IntentScript Language Specification](../spec.md)
- [Design Document](../.kiro/specs/intentscript-compiler/design.md)
- [Requirements Document](../.kiro/specs/intentscript-compiler/requirements.md)

## Contributing

When adding new examples:
1. Include comprehensive comments explaining the task
2. Demonstrate at least one unique feature
3. Update this README with usage instructions
4. Test the example end-to-end
