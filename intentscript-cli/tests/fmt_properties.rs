// Property-based tests for the fmt command
// Feature: intentscript-compiler, Property 47: Format idempotence
// Validates: Requirements 12.4

use intentscript_parser::{File, Parser};
use quickcheck::{Arbitrary, Gen, TestResult};
use quickcheck_macros::quickcheck;

// Helper to create a formatter
fn format_file(file: &File) -> String {
    // We need to access the formatter from the CLI crate
    // For now, we'll use the parser to format
    let mut formatter = intentscript_cli::formatter::Formatter::new();
    formatter.format_file(file)
}

/// Property 47: Format idempotence
/// For any valid IntentScript source, formatting it twice should produce identical output
/// format(format(x)) = format(x)
#[quickcheck]
fn property_format_idempotence(source: ValidIntentScriptSource) -> TestResult {
    // Parse the source
    let mut parser = Parser::new(&source.0);
    let file = match parser.parse_file() {
        Ok(f) => f,
        Err(_) => return TestResult::discard(), // Skip invalid sources
    };

    // Format once
    let formatted_once = format_file(&file);

    // Parse the formatted output
    let mut parser2 = Parser::new(&formatted_once);
    let file2 = match parser2.parse_file() {
        Ok(f) => f,
        Err(_) => {
            // If the formatted output doesn't parse, that's a bug
            return TestResult::failed();
        }
    };

    // Format again
    let formatted_twice = format_file(&file2);

    // Check idempotence: format(format(x)) == format(x)
    TestResult::from_bool(formatted_once.trim() == formatted_twice.trim())
}

/// Wrapper for valid IntentScript source code
#[derive(Clone, Debug)]
struct ValidIntentScriptSource(String);

impl Arbitrary for ValidIntentScriptSource {
    fn arbitrary(g: &mut Gen) -> Self {
        // Generate a simple valid task
        let task_name = format!("task_{}", u8::arbitrary(g) % 10);
        let version = if bool::arbitrary(g) {
            format!(" v{}.{}", u8::arbitrary(g) % 5, u8::arbitrary(g) % 10)
        } else {
            String::new()
        };

        let goal = if bool::arbitrary(g) {
            format!("  goal: \"Test goal\"\n")
        } else {
            String::new()
        };

        let input = if bool::arbitrary(g) {
            format!("  input: name: text\n")
        } else {
            String::new()
        };

        let run = format!("  run: step1\n");

        let source = format!(
            "task \"{}\"{}  {{\n{}{}{}}}\n",
            task_name, version, goal, input, run
        );

        ValidIntentScriptSource(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_idempotence_simple() {
        let source = r#"task "simple" v1.0 {
  goal: "Test"
  run: step1
}"#;

        let mut parser = Parser::new(source);
        let file = parser.parse_file().unwrap();

        let formatted_once = format_file(&file);
        
        let mut parser2 = Parser::new(&formatted_once);
        let file2 = parser2.parse_file().unwrap();
        
        let formatted_twice = format_file(&file2);

        assert_eq!(formatted_once.trim(), formatted_twice.trim());
    }

    #[test]
    fn test_format_idempotence_with_input() {
        let source = r#"task "with_input" v1.0 {
  goal: "Test"
  input: name: text
  run: step1
}"#;

        let mut parser = Parser::new(source);
        let file = parser.parse_file().unwrap();

        let formatted_once = format_file(&file);
        
        let mut parser2 = Parser::new(&formatted_once);
        let file2 = parser2.parse_file().unwrap();
        
        let formatted_twice = format_file(&file2);

        assert_eq!(formatted_once.trim(), formatted_twice.trim());
    }
}
