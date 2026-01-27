# CLI Reference

Complete reference for the IntentScript command-line interface.

## Installation

```bash
cargo install --path intentscript-cli
```

## Global Options

```bash
intentscript [OPTIONS] <COMMAND>
```

### Options

- `-h, --help` - Print help information
- `-V, --version` - Print version information
- `-v, --verbose` - Enable verbose output
- `-q, --quiet` - Suppress non-error output

## Commands

### `build`

Compile IntentScript source to Execution Plan IR.

```bash
intentscript build [OPTIONS] <SOURCE>
```

#### Arguments

- `<SOURCE>` - Path to IntentScript source file (.intent)

#### Options

- `-o, --output <FILE>` - Output IR file path (default: `<source>.ir.json`)
- `--json` - Output diagnostics in JSON format
- `--policy <FILE>` - Path to policy file (optional)
- `--check` - Check compilation without writing output

#### Examples

```bash
# Basic compilation
intentscript build task.intent

# Specify output file
intentscript build task.intent -o my_task.ir.json

# Check without writing
intentscript build task.intent --check

# JSON diagnostics for CI
intentscript build task.intent --json > diagnostics.json
```

#### Exit Codes

- `0` - Success
- `1` - Compilation error
- `2` - Warning (with --check)

### `run`

Execute an Execution Plan IR file.

```bash
intentscript run [OPTIONS] <IR_FILE>
```

#### Arguments

- `<IR_FILE>` - Path to Execution Plan IR file (.ir.json)

#### Options

- `--input <KEY=VALUE>` - Provide input values (can be repeated)
- `--host <TYPE>` - Host adapter type (default: `default`)
- `--timeout <MS>` - Execution timeout in milliseconds
- `--output-dir <DIR>` - Directory for output artifacts (default: `./artifacts`)
- `--audit-log <FILE>` - Path to write audit log (default: `./audit.log`)
- `--json` - Output results in JSON format

#### Examples

```bash
# Basic execution
intentscript run task.ir.json

# With inputs
intentscript run task.ir.json --input file_path=./data.txt --input count=10

# With timeout
intentscript run task.ir.json --timeout 30000

# Custom output directory
intentscript run task.ir.json --output-dir ./results

# JSON output for CI
intentscript run task.ir.json --json > results.json
```

#### Exit Codes

- `0` - Success (all checks passed)
- `1` - Runtime error
- `2` - Validation failure
- `3` - Timeout
- `4` - Capability violation

### `lint`

Check IntentScript source for errors and warnings.

```bash
intentscript lint [OPTIONS] <SOURCE>
```

#### Arguments

- `<SOURCE>` - Path to IntentScript source file (.intent)

#### Options

- `--json` - Output diagnostics in JSON format
- `--policy <FILE>` - Path to policy file (optional)
- `--strict` - Treat warnings as errors

#### Examples

```bash
# Basic linting
intentscript lint task.intent

# JSON output for CI
intentscript lint task.intent --json

# Strict mode
intentscript lint task.intent --strict

# Lint multiple files
intentscript lint tasks/*.intent
```

#### Diagnostic Format

Text format:
```
error[E0001]: unexpected token
  --> task.intent:5:10
   |
 5 |   input: name text
   |              ^^^^ expected ':', found 'text'
```

JSON format:
```json
{
  "diagnostics": [
    {
      "level": "error",
      "code": "E0001",
      "message": "unexpected token",
      "location": {
        "file": "task.intent",
        "line": 5,
        "column": 10
      },
      "suggestion": "expected ':', found 'text'"
    }
  ]
}
```

#### Exit Codes

- `0` - No errors
- `1` - Errors found
- `2` - Warnings found (with --strict)

### `fmt`

Format IntentScript source code.

```bash
intentscript fmt [OPTIONS] <SOURCE>
```

#### Arguments

- `<SOURCE>` - Path to IntentScript source file (.intent)

#### Options

- `--check` - Check if file is formatted without modifying
- `--write` - Write formatted output to file (default)
- `--stdout` - Print formatted output to stdout

#### Examples

```bash
# Format file in place
intentscript fmt task.intent

# Check formatting
intentscript fmt task.intent --check

# Print to stdout
intentscript fmt task.intent --stdout

# Format multiple files
intentscript fmt tasks/*.intent
```

#### Formatting Rules

- Consistent indentation (2 spaces)
- Aligned colons in input/constraint blocks
- Consistent spacing around operators
- Normalized string quotes
- Sorted imports (future feature)

#### Exit Codes

- `0` - Success (or already formatted with --check)
- `1` - Error
- `2` - Not formatted (with --check)

### `explain`

Explain an Execution Plan IR file.

```bash
intentscript explain [OPTIONS] <IR_FILE>
```

#### Arguments

- `<IR_FILE>` - Path to Execution Plan IR file (.ir.json)

#### Options

- `--format <FORMAT>` - Output format: `text`, `json`, `markdown` (default: `text`)
- `--verbose` - Include detailed step information
- `--audit-log <FILE>` - Include execution results from audit log

#### Examples

```bash
# Basic explanation
intentscript explain task.ir.json

# Verbose output
intentscript explain task.ir.json --verbose

# Markdown format
intentscript explain task.ir.json --format markdown > explanation.md

# With execution results
intentscript explain task.ir.json --audit-log ./audit.log
```

#### Output Sections

1. **Metadata**: Task name, version, compiler version, policy hash
2. **Inputs**: Required and optional inputs with types
3. **Capabilities**: Enabled capabilities and constraints
4. **Limits**: Execution limits (max repairs, timeout)
5. **Steps**: Execution pipeline with step details
6. **Outputs**: Expected output artifacts
7. **Checks**: Validation checks and their status (if audit log provided)

#### Exit Codes

- `0` - Success
- `1` - Error reading IR file

## Environment Variables

### `INTENTSCRIPT_POLICY_PATH`

Default path to policy file.

```bash
export INTENTSCRIPT_POLICY_PATH=./policies/default.toml
intentscript build task.intent
```

### `INTENTSCRIPT_HOST`

Default host adapter type.

```bash
export INTENTSCRIPT_HOST=custom
intentscript run task.ir.json
```

### `INTENTSCRIPT_LOG_LEVEL`

Logging level: `error`, `warn`, `info`, `debug`, `trace`.

```bash
export INTENTSCRIPT_LOG_LEVEL=debug
intentscript run task.ir.json
```

## Configuration File

Create `.intentscript.toml` in your project root:

```toml
[build]
policy = "./policies/default.toml"
output_dir = "./ir"

[run]
host = "default"
timeout = 60000
output_dir = "./artifacts"
audit_log = "./audit.log"

[lint]
strict = false

[fmt]
indent = 2
max_line_length = 100
```

## CI/CD Integration

### GitHub Actions

```yaml
name: IntentScript CI

on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Install IntentScript
        run: cargo install --path intentscript-cli
      - name: Lint tasks
        run: intentscript lint tasks/*.intent --json > lint-results.json
      - name: Upload results
        uses: actions/upload-artifact@v2
        with:
          name: lint-results
          path: lint-results.json

  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Install IntentScript
        run: cargo install --path intentscript-cli
      - name: Build tasks
        run: |
          for task in tasks/*.intent; do
            intentscript build "$task" -o "ir/$(basename "$task" .intent).ir.json"
          done
      - name: Upload IR
        uses: actions/upload-artifact@v2
        with:
          name: execution-plans
          path: ir/

  test:
    runs-on: ubuntu-latest
    needs: build
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Install IntentScript
        run: cargo install --path intentscript-cli
      - name: Download IR
        uses: actions/download-artifact@v2
        with:
          name: execution-plans
          path: ir/
      - name: Run tasks
        run: |
          for ir in ir/*.ir.json; do
            intentscript run "$ir" --json > "results/$(basename "$ir" .ir.json).json"
          done
      - name: Upload results
        uses: actions/upload-artifact@v2
        with:
          name: test-results
          path: results/
```

### GitLab CI

```yaml
stages:
  - lint
  - build
  - test

lint:
  stage: lint
  script:
    - cargo install --path intentscript-cli
    - intentscript lint tasks/*.intent --json > lint-results.json
  artifacts:
    reports:
      junit: lint-results.json

build:
  stage: build
  script:
    - cargo install --path intentscript-cli
    - mkdir -p ir
    - for task in tasks/*.intent; do
        intentscript build "$task" -o "ir/$(basename "$task" .intent).ir.json";
      done
  artifacts:
    paths:
      - ir/

test:
  stage: test
  dependencies:
    - build
  script:
    - cargo install --path intentscript-cli
    - mkdir -p results
    - for ir in ir/*.ir.json; do
        intentscript run "$ir" --json > "results/$(basename "$ir" .ir.json).json";
      done
  artifacts:
    paths:
      - results/
```

## Shell Completion

Generate shell completion scripts:

### Bash

```bash
intentscript completions bash > /etc/bash_completion.d/intentscript
```

### Zsh

```bash
intentscript completions zsh > /usr/local/share/zsh/site-functions/_intentscript
```

### Fish

```bash
intentscript completions fish > ~/.config/fish/completions/intentscript.fish
```

### PowerShell

```powershell
intentscript completions powershell > $PROFILE
```

## Troubleshooting

### Common Issues

**Command not found**
```bash
# Ensure cargo bin is in PATH
export PATH="$HOME/.cargo/bin:$PATH"
```

**Permission denied**
```bash
# Make binary executable
chmod +x ~/.cargo/bin/intentscript
```

**Slow compilation**
```bash
# Use release build
cargo build --release
cargo install --path intentscript-cli --release
```

**JSON parsing errors**
```bash
# Validate IR file
jq . task.ir.json
```

## Advanced Usage

### Custom Host Adapters

```bash
# Use custom host implementation
intentscript run task.ir.json --host custom

# With host configuration
intentscript run task.ir.json --host custom --host-config config.toml
```

### Debugging

```bash
# Enable debug logging
INTENTSCRIPT_LOG_LEVEL=debug intentscript run task.ir.json

# Trace execution
INTENTSCRIPT_LOG_LEVEL=trace intentscript run task.ir.json --verbose

# Dump IR
intentscript build task.intent --output - | jq .
```

### Performance Profiling

```bash
# Time compilation
time intentscript build task.intent

# Time execution
time intentscript run task.ir.json

# Profile with cargo
cargo build --release --bin intentscript
perf record target/release/intentscript run task.ir.json
perf report
```

## See Also

- [Getting Started Guide](getting-started.md)
- [Language Reference](language-reference.md)
- [Host Trait Guide](host-trait.md)
- [Examples](../examples/)
