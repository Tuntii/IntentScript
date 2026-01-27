// Property-based tests for the lexer
// Feature: intentscript-compiler

use intentscript_parser::{Lexer, TokenKind};
use quickcheck::{Arbitrary, Gen, TestResult};
use quickcheck_macros::quickcheck;

// Helper to generate valid identifiers
#[derive(Clone, Debug)]
struct ValidIdent(String);

impl Arbitrary for ValidIdent {
    fn arbitrary(g: &mut Gen) -> Self {
        let first_chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";
        let rest_chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_";
        
        let len = usize::arbitrary(g) % 20 + 1; // 1-20 characters
        let mut s = String::new();
        
        // First character
        let idx = usize::arbitrary(g) % first_chars.len();
        s.push(first_chars.chars().nth(idx).unwrap());
        
        // Rest of characters
        for _ in 1..len {
            let idx = usize::arbitrary(g) % rest_chars.len();
            s.push(rest_chars.chars().nth(idx).unwrap());
        }
        
        ValidIdent(s)
    }
}

/// Feature: intentscript-compiler, Property 1: Whitespace token separation
/// Validates: Requirements 2.1
/// 
/// For any IntentScript source containing whitespace characters (spaces, tabs, newlines),
/// the lexer should correctly separate tokens on either side of the whitespace.
#[quickcheck]
fn property_whitespace_token_separation(ident1: ValidIdent, ident2: ValidIdent) -> TestResult {
    // Skip if identifiers are keywords
    let keywords = [
        "task", "goal", "input", "constraints", "output_schema", "checks", "run",
        "bool", "int", "float", "text", "url", "email", "path", "bytes", "json",
        "object", "list", "enum", "optional", "openapi", "markdown", "xlsx", "pdf",
        "true", "false"
    ];
    
    if keywords.contains(&ident1.0.as_str()) || keywords.contains(&ident2.0.as_str()) {
        return TestResult::discard();
    }
    
    // Test with different whitespace separators
    let whitespace_variants = vec![" ", "  ", "\t", "\n", " \t ", "\n\n", " \n\t "];
    
    for ws in whitespace_variants {
        let source = format!("{}{}{}", ident1.0, ws, ident2.0);
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize();
        
        // Filter out EOF token
        let non_eof_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.kind != TokenKind::Eof)
            .collect();
        
        // Should have exactly 2 tokens (the two identifiers)
        if non_eof_tokens.len() != 2 {
            return TestResult::failed();
        }
        
        // First token should be first identifier
        if let TokenKind::Ident(ref id) = non_eof_tokens[0].kind {
            if id != &ident1.0 {
                return TestResult::failed();
            }
        } else {
            return TestResult::failed();
        }
        
        // Second token should be second identifier
        if let TokenKind::Ident(ref id) = non_eof_tokens[1].kind {
            if id != &ident2.0 {
                return TestResult::failed();
            }
        } else {
            return TestResult::failed();
        }
    }
    
    TestResult::passed()
}

/// Feature: intentscript-compiler, Property 2: Comment content ignored
/// Validates: Requirements 2.2
/// 
/// For any string content following "//", the lexer should ignore all characters
/// until end of line and not include them in any token.
#[quickcheck]
fn property_comment_content_ignored(ident: ValidIdent, comment_text: String) -> TestResult {
    // Skip if identifier is a keyword
    let keywords = [
        "task", "goal", "input", "constraints", "output_schema", "checks", "run",
        "bool", "int", "float", "text", "url", "email", "path", "bytes", "json",
        "object", "list", "enum", "optional", "openapi", "markdown", "xlsx", "pdf",
        "true", "false"
    ];
    
    if keywords.contains(&ident.0.as_str()) {
        return TestResult::discard();
    }
    
    // Filter out newlines from comment text to keep it on one line
    let comment_text: String = comment_text.chars().filter(|&c| c != '\n' && c != '\r').collect();
    
    // Test: identifier followed by comment
    let source = format!("{} // {}", ident.0, comment_text);
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();
    
    // Filter out EOF token
    let non_eof_tokens: Vec<_> = tokens.iter()
        .filter(|t| t.kind != TokenKind::Eof)
        .collect();
    
    // Should have exactly 1 token (the identifier), comment should be ignored
    if non_eof_tokens.len() != 1 {
        return TestResult::failed();
    }
    
    // Token should be the identifier
    if let TokenKind::Ident(ref id) = non_eof_tokens[0].kind {
        if id != &ident.0 {
            return TestResult::failed();
        }
    } else {
        return TestResult::failed();
    }
    
    // Test: comment followed by identifier on next line
    let source = format!("// {}\n{}", comment_text, ident.0);
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();
    
    let non_eof_tokens: Vec<_> = tokens.iter()
        .filter(|t| t.kind != TokenKind::Eof)
        .collect();
    
    // Should have exactly 1 token (the identifier)
    if non_eof_tokens.len() != 1 {
        return TestResult::failed();
    }
    
    if let TokenKind::Ident(ref id) = non_eof_tokens[0].kind {
        if id != &ident.0 {
            return TestResult::failed();
        }
    } else {
        return TestResult::failed();
    }
    
    TestResult::passed()
}

/// Feature: intentscript-compiler, Property 3: Valid identifier tokenization
/// Validates: Requirements 2.3
/// 
/// For any string matching the pattern [A-Za-z_][A-Za-z0-9_]*, the lexer should
/// produce an IDENT token with the correct lexeme.
#[quickcheck]
fn property_valid_identifier_tokenization(ident: ValidIdent) -> TestResult {
    // Skip if identifier is a keyword
    let keywords = [
        "task", "goal", "input", "constraints", "output_schema", "checks", "run",
        "bool", "int", "float", "text", "url", "email", "path", "bytes", "json",
        "object", "list", "enum", "optional", "openapi", "markdown", "xlsx", "pdf",
        "true", "false"
    ];
    
    if keywords.contains(&ident.0.as_str()) {
        return TestResult::discard();
    }
    
    let source = ident.0.clone();
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();
    
    // Filter out EOF token
    let non_eof_tokens: Vec<_> = tokens.iter()
        .filter(|t| t.kind != TokenKind::Eof)
        .collect();
    
    // Should have exactly 1 token
    if non_eof_tokens.len() != 1 {
        return TestResult::failed();
    }
    
    // Token should be an identifier with the correct lexeme
    if let TokenKind::Ident(ref id) = non_eof_tokens[0].kind {
        if id != &ident.0 {
            return TestResult::failed();
        }
        if non_eof_tokens[0].lexeme != ident.0 {
            return TestResult::failed();
        }
    } else {
        return TestResult::failed();
    }
    
    TestResult::passed()
}

/// Feature: intentscript-compiler, Property 4: String literal parsing with escapes
/// Validates: Requirements 2.4
/// 
/// For any valid double-quoted string containing escape sequences (\n, \t, \", \\),
/// the lexer should correctly parse the string and interpret escape sequences.
#[quickcheck]
fn property_string_literal_parsing_with_escapes(content: String) -> TestResult {
    // Filter out quotes and backslashes from the content to avoid complications
    // We'll test escapes separately
    let content: String = content.chars()
        .filter(|&c| c != '"' && c != '\\')
        .take(50) // Limit length
        .collect();
    
    if content.is_empty() {
        return TestResult::discard();
    }
    
    // Test 1: Simple string without escapes
    let source = format!("\"{}\"", content);
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();
    
    let non_eof_tokens: Vec<_> = tokens.iter()
        .filter(|t| t.kind != TokenKind::Eof)
        .collect();
    
    if non_eof_tokens.len() != 1 {
        return TestResult::failed();
    }
    
    if let TokenKind::StringLiteral(ref value) = non_eof_tokens[0].kind {
        if value != &content {
            return TestResult::failed();
        }
    } else {
        return TestResult::failed();
    }
    
    // Test 2: String with escape sequences
    let test_cases = vec![
        (r#""hello\nworld""#, "hello\nworld"),
        (r#""tab\there""#, "tab\there"),
        (r#""quote\"here""#, "quote\"here"),
        (r#""backslash\\here""#, "backslash\\here"),
        (r#""multiple\n\t\"\\"#, "multiple\n\t\"\\"),
    ];
    
    for (source, expected) in test_cases {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        
        let non_eof_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.kind != TokenKind::Eof)
            .collect();
        
        if non_eof_tokens.len() != 1 {
            return TestResult::failed();
        }
        
        if let TokenKind::StringLiteral(ref value) = non_eof_tokens[0].kind {
            if value != expected {
                return TestResult::failed();
            }
        } else {
            return TestResult::failed();
        }
    }
    
    TestResult::passed()
}

/// Feature: intentscript-compiler, Property 5: Numeric type distinction
/// Validates: Requirements 2.5
/// 
/// For any numeric literal, the lexer should correctly classify it as either
/// an integer (no decimal point) or float (with decimal point).
#[quickcheck]
fn property_numeric_type_distinction(int_part: u32, frac_part: u32) -> TestResult {
    // Test 1: Integer literal
    let int_source = format!("{}", int_part);
    let mut lexer = Lexer::new(&int_source);
    let tokens = lexer.tokenize();
    
    let non_eof_tokens: Vec<_> = tokens.iter()
        .filter(|t| t.kind != TokenKind::Eof)
        .collect();
    
    if non_eof_tokens.len() != 1 {
        return TestResult::failed();
    }
    
    // Should be an integer literal
    match &non_eof_tokens[0].kind {
        TokenKind::IntLiteral(value) => {
            if *value != int_part as i64 {
                return TestResult::failed();
            }
        }
        _ => return TestResult::failed(),
    }
    
    // Test 2: Float literal
    let float_source = format!("{}.{}", int_part, frac_part);
    let mut lexer = Lexer::new(&float_source);
    let tokens = lexer.tokenize();
    
    let non_eof_tokens: Vec<_> = tokens.iter()
        .filter(|t| t.kind != TokenKind::Eof)
        .collect();
    
    if non_eof_tokens.len() != 1 {
        return TestResult::failed();
    }
    
    // Should be a float literal
    match &non_eof_tokens[0].kind {
        TokenKind::FloatLiteral(value) => {
            let expected: f64 = float_source.parse().unwrap();
            // Use approximate comparison for floats
            if (value - expected).abs() > 1e-10 {
                return TestResult::failed();
            }
        }
        _ => return TestResult::failed(),
    }
    
    // Test 3: Edge case - number followed by dot but no digit should be int + error
    // For now, we'll just test that integers without decimal points are integers
    let test_cases = vec![
        ("0", true),
        ("123", true),
        ("999999", true),
        ("0.0", false),
        ("1.5", false),
        ("123.456", false),
    ];
    
    for (source, should_be_int) in test_cases {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        
        let non_eof_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.kind != TokenKind::Eof)
            .collect();
        
        if non_eof_tokens.len() != 1 {
            return TestResult::failed();
        }
        
        let is_int = matches!(non_eof_tokens[0].kind, TokenKind::IntLiteral(_));
        let is_float = matches!(non_eof_tokens[0].kind, TokenKind::FloatLiteral(_));
        
        if should_be_int && !is_int {
            return TestResult::failed();
        }
        if !should_be_int && !is_float {
            return TestResult::failed();
        }
    }
    
    TestResult::passed()
}
