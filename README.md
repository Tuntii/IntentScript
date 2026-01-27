# IntentScript

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)]()
[![License](https://img.shields.io/badge/license-MIT-blue)]()

IntentScript is a declarative task language designed to express **intent**, **constraints**, **schemas**, and **validation checks**, and to compile them into a deterministic execution plan that can run in CI and tooling.

## Features

- 🎯 **Declarative Syntax**: Express what you want, not how to do it
- 🔒 **Capability-Based Security**: Fine-grained control over side effects
- ✅ **Built-in Validation**: Declarative checks and schema validation
- 🔄 **Deterministic Execution**: Same inputs always produce same outputs
- 📊 **Audit Trail**: Complete logging of all operations
- 🚀 **CI-First Design**: Fast compilation and stable diagnostics
- 🔌 **Extensible**: Custom Host adapters for different environments

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/intentscript.git
cd intentscript

# Build the project
cargo build --release

# Install the CLI
cargo install --path intentscript-cli
```

### Your First Task

Create a file `hello.intent`:

```intentscript
task "HelloWorld" v1.0 {
  goal: "Validate a greeting message"
  
  input: name: text = "World"
  
  constraints: {
    net = off
  }
  
  output_schema: text
  
  checks: {
    must_not_be_empty
  }
  
  run:
    format_greeting(name) ->
    validate ->
    report(format: "text")
}
```

Compile and run:

```bash
# Compile to IR
intentscript build hello.intent -o hello.ir.json

# Execute the task
intentscript run hello.ir.json --input name="Alice"

# Lint the source
intentscript lint hello.intent

# Format the source
intentscript fmt hello.intent
```

## Documentation

- **[Getting Started Guide](docs/getting-started.md)** - Learn the basics
- **[Language Reference](docs/language-reference.md)** - Complete syntax and semantics
- **[CLI Reference](docs/cli-reference.md)** - Command-line interface
- **[Host Trait Guide](docs/host-trait.md)** - Creating custom adapters
- **[Examples](examples/)** - Sample tasks demonstrating features

## Project Structure

This is a Rust workspace containing the following crates:

| Crate | Description |
|-------|-------------|
| **intentscript-core** | Core data structures (Span, Position, Error types) |
| **intentscript-parser** | Lexer and parser for IntentScript source |
| **intentscript-compiler** | Semantic analysis and IR lowering |
| **intentscript-runtime** | Execution engine with Host interface |
| **intentscript-cli** | Command-line interface |

## Architecture

```
┌─────────────────┐
│ IntentScript    │
│ Source (.intent)│
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Compiler        │
│ (Lex→Parse→     │
│  Analyze→Lower) │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Execution Plan  │
│ (JSON IR)       │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Runtime         │
│ (State Machine) │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Host Interface  │
│ (File I/O, etc) │
└─────────────────┘
```

## Key Concepts

### Tasks
A task is a unit of work with:
- **Goal**: What you want to achieve
- **Inputs**: Required data with types
- **Constraints**: Capability and policy rules
- **Checks**: Validation predicates
- **Pipeline**: Execution steps

### Type System
- **Primitives**: `text`, `int`, `float`, `bool`, `path`, `url`, `email`, `bytes`, `json`
- **Structured**: `object`, `list`, `enum`, `optional`
- **Domain**: `openapi`, `markdown`, `xlsx`, `pdf`

### Capabilities
Control what tasks can do:
- `fs`: Filesystem access (read/write roots)
- `net`: Network access (off by default)
- `templates`: Template rendering
- `exports`: Export to XLSX, PDF, etc.
- `exec`: External command execution

### Determinism
IntentScript guarantees:
- Same source + same inputs = same IR
- Same IR + same inputs = same outputs
- Complete audit trail of all operations

## Examples

### OpenAPI Validation
```intentscript
task "ValidateAPI" v1.0 {
  goal: "Validate OpenAPI specification"
  input: spec_file: path
  
  constraints: {
    net = off
  }
  
  checks: {
    must_have_security_schemes(["bearerAuth"])
    must_include_paths_prefix("/api")
  }
  
  run:
    read_file(spec_file) ->
    parse_openapi ->
    validate ->
    report(format: "markdown")
}
```

See [examples/](examples/) for more.

## Development

### Building

```bash
# Build all crates
cargo build

# Build with optimizations
cargo build --release

# Build specific crate
cargo build -p intentscript-cli
```

### Testing

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p intentscript-parser

# Run with output
cargo test -- --nocapture

# Run property-based tests
cargo test properties
```

### Linting

```bash
# Check code
cargo clippy

# Format code
cargo fmt
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `build` | Compile IntentScript source to IR |
| `run` | Execute an IR file |
| `lint` | Check source for errors and warnings |
| `fmt` | Format IntentScript source |
| `explain` | Explain an execution plan |

Run `intentscript --help` for detailed usage.

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Run `cargo test` and `cargo clippy`
5. Submit a pull request

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by declarative workflow languages and capability-based security
- Built with Rust for performance and safety
- Designed for the RustAPI ecosystem

## Resources

- [Language Specification](spec.md)
- [Design Document](.kiro/specs/intentscript-compiler/design.md)
- [Requirements Document](.kiro/specs/intentscript-compiler/requirements.md)
- [Task List](.kiro/specs/intentscript-compiler/tasks.md)

## Status

IntentScript is under active development. Core features are implemented:

- ✅ Lexer and parser
- ✅ Type system and semantic analysis
- ✅ IR generation and lowering
- ✅ Runtime execution engine
- ✅ CLI commands
- ✅ Property-based testing
- 🚧 Documentation (in progress)
- 🚧 Additional examples

See the [task list](.kiro/specs/intentscript-compiler/tasks.md) for current progress.
