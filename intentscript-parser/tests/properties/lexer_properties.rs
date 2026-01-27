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

