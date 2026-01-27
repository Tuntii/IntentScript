use intentscript_core::Span;
use std::fmt;

/// A token in the IntentScript language
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, span: Span) -> Self {
        Self { kind, lexeme, span }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}('{}') at {}:{}",
            self.kind, self.lexeme, self.span.line, self.span.column
        )
    }
}

/// Token kinds in the IntentScript language
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Task,
    Goal,
    Input,
    Constraints,
    OutputSchema,
    Checks,
    Run,
    
    // Type keywords
    Bool,
    Int,
    Float,
    Text,
    Url,
    Email,
    Path,
    Bytes,
    Json,
    Object,
    List,
    Enum,
    Optional,
    
    // Domain type keywords
    OpenApi,
    Markdown,
    Xlsx,
    Pdf,
    
    // Literals
    StringLiteral(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),
    
    // Identifiers
    Ident(String),
    
    // Symbols
    LeftBrace,      // {
    RightBrace,     // }
    LeftParen,      // (
    RightParen,     // )
    LeftBracket,    // [
    RightBracket,   // ]
    Colon,          // :
    Comma,          // ,
    Equal,          // =
    Arrow,          // ->
    Pipe,           // |>
    
    // Special
    Comment(String),
    Whitespace,
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Task => write!(f, "task"),
            TokenKind::Goal => write!(f, "goal"),
            TokenKind::Input => write!(f, "input"),
            TokenKind::Constraints => write!(f, "constraints"),
            TokenKind::OutputSchema => write!(f, "output_schema"),
            TokenKind::Checks => write!(f, "checks"),
            TokenKind::Run => write!(f, "run"),
            TokenKind::Bool => write!(f, "bool"),
            TokenKind::Int => write!(f, "int"),
            TokenKind::Float => write!(f, "float"),
            TokenKind::Text => write!(f, "text"),
            TokenKind::Url => write!(f, "url"),
            TokenKind::Email => write!(f, "email"),
            TokenKind::Path => write!(f, "path"),
            TokenKind::Bytes => write!(f, "bytes"),
            TokenKind::Json => write!(f, "json"),
            TokenKind::Object => write!(f, "object"),
            TokenKind::List => write!(f, "list"),
            TokenKind::Enum => write!(f, "enum"),
            TokenKind::Optional => write!(f, "optional"),
            TokenKind::OpenApi => write!(f, "openapi"),
            TokenKind::Markdown => write!(f, "markdown"),
            TokenKind::Xlsx => write!(f, "xlsx"),
            TokenKind::Pdf => write!(f, "pdf"),
            TokenKind::StringLiteral(s) => write!(f, "string(\"{}\")", s),
            TokenKind::IntLiteral(i) => write!(f, "int({})", i),
            TokenKind::FloatLiteral(fl) => write!(f, "float({})", fl),
            TokenKind::BoolLiteral(b) => write!(f, "bool({})", b),
            TokenKind::Ident(id) => write!(f, "ident({})", id),
            TokenKind::LeftBrace => write!(f, "{{"),
            TokenKind::RightBrace => write!(f, "}}"),
            TokenKind::LeftParen => write!(f, "("),
            TokenKind::RightParen => write!(f, ")"),
            TokenKind::LeftBracket => write!(f, "["),
            TokenKind::RightBracket => write!(f, "]"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Equal => write!(f, "="),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::Pipe => write!(f, "|>"),
            TokenKind::Comment(_) => write!(f, "comment"),
            TokenKind::Whitespace => write!(f, "whitespace"),
            TokenKind::Eof => write!(f, "EOF"),
        }
    }
}

pub struct Lexer {
    source: Vec<char>,
    current: usize,      // Character index
    line: u32,
    column: u32,
    byte_offset: usize,  // Byte offset for span tracking
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
            byte_offset: 0,
        }
    }

    /// Get the next token from the source
    pub fn next_token(&mut self) -> Token {
        // Skip whitespace and comments
        self.skip_whitespace_and_comments();

        if self.is_at_end() {
            return self.make_token(TokenKind::Eof, "");
        }

        let start_line = self.line;
        let start_column = self.column;
        let start_byte_offset = self.byte_offset;
        let start_char_index = self.current;

        let ch = self.advance();

        let kind = match ch {
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '[' => TokenKind::LeftBracket,
            ']' => TokenKind::RightBracket,
            ':' => TokenKind::Colon,
            ',' => TokenKind::Comma,
            '=' => TokenKind::Equal,
            '-' if self.peek() == '>' => {
                self.advance();
                TokenKind::Arrow
            }
            '|' if self.peek() == '>' => {
                self.advance();
                TokenKind::Pipe
            }
            '"' => return self.string_literal(start_line, start_column, start_byte_offset, start_char_index),
            c if c.is_ascii_digit() => {
                return self.number_literal(start_line, start_column, start_byte_offset, start_char_index)
            }
            c if c.is_alphabetic() || c == '_' => {
                return self.identifier_or_keyword(start_line, start_column, start_byte_offset, start_char_index)
            }
            _ => {
                // Unknown character - create an error token
                let lexeme = ch.to_string();
                let span = Span::new(start_line, start_column, start_byte_offset, lexeme.len());
                return Token::new(TokenKind::Eof, lexeme, span); // Treat as EOF for now
            }
        };

        let end_char_index = self.current;
        let lexeme: String = self.source[start_char_index..end_char_index].iter().collect();
        let span = Span::new(start_line, start_column, start_byte_offset, lexeme.len());
        Token::new(kind, lexeme, span)
    }

    /// Peek at all remaining tokens without consuming them
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            if self.is_at_end() {
                break;
            }

            let ch = self.peek();
            match ch {
                ' ' | '\t' | '\r' | '\n' => {
                    self.advance();
                }
                '/' if self.peek_next() == '/' => {
                    // Line comment - skip until end of line
                    self.advance(); // consume first /
                    self.advance(); // consume second /
                    while !self.is_at_end() && self.peek() != '\n' {
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn string_literal(&mut self, start_line: u32, start_column: u32, start_byte_offset: usize, start_char_index: usize) -> Token {
        let mut value = String::new();

        while !self.is_at_end() && self.peek() != '"' {
            let ch = self.advance();
            if ch == '\\' && !self.is_at_end() {
                // Handle escape sequences
                let next = self.advance();
                match next {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    _ => {
                        // Invalid escape - just include both characters
                        value.push('\\');
                        value.push(next);
                    }
                }
            } else {
                value.push(ch);
            }
        }

        // Consume closing quote
        if !self.is_at_end() {
            self.advance();
        }

        let end_char_index = self.current;
        let lexeme: String = self.source[start_char_index..end_char_index].iter().collect();
        let span = Span::new(start_line, start_column, start_byte_offset, lexeme.len());
        Token::new(TokenKind::StringLiteral(value), lexeme, span)
    }

    fn number_literal(&mut self, start_line: u32, start_column: u32, start_byte_offset: usize, start_char_index: usize) -> Token {
        // Consume all digits
        while !self.is_at_end() && self.peek().is_ascii_digit() {
            self.advance();
        }

        // Check for decimal point
        let is_float = !self.is_at_end() && self.peek() == '.' && {
            // Look ahead to ensure there's a digit after the dot
            self.current + 1 < self.source.len() && self.source[self.current + 1].is_ascii_digit()
        };

        if is_float {
            self.advance(); // consume '.'
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let end_char_index = self.current;
        let lexeme: String = self.source[start_char_index..end_char_index].iter().collect();
        let span = Span::new(start_line, start_column, start_byte_offset, lexeme.len());

        if is_float {
            let value = lexeme.parse::<f64>().unwrap_or(0.0);
            Token::new(TokenKind::FloatLiteral(value), lexeme, span)
        } else {
            let value = lexeme.parse::<i64>().unwrap_or(0);
            Token::new(TokenKind::IntLiteral(value), lexeme, span)
        }
    }

    fn identifier_or_keyword(
        &mut self,
        start_line: u32,
        start_column: u32,
        start_byte_offset: usize,
        start_char_index: usize,
    ) -> Token {
        // Consume all alphanumeric characters and underscores
        while !self.is_at_end() {
            let ch = self.peek();
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else if ch == '.' && self.current > start_char_index && self.source[start_char_index] == 'v' {
                // Special case: version identifiers like v1.0 or v1.0.2
                // Only allow dots in identifiers that start with 'v'
                self.advance();
            } else {
                break;
            }
        }

        let end_char_index = self.current;
        let lexeme: String = self.source[start_char_index..end_char_index].iter().collect();
        let span = Span::new(start_line, start_column, start_byte_offset, lexeme.len());

        // Check if it's a keyword
        let kind = match lexeme.as_str() {
            "task" => TokenKind::Task,
            "goal" => TokenKind::Goal,
            "input" => TokenKind::Input,
            "constraints" => TokenKind::Constraints,
            "output_schema" => TokenKind::OutputSchema,
            "checks" => TokenKind::Checks,
            "run" => TokenKind::Run,
            "bool" => TokenKind::Bool,
            "int" => TokenKind::Int,
            "float" => TokenKind::Float,
            "text" => TokenKind::Text,
            "url" => TokenKind::Url,
            "email" => TokenKind::Email,
            "path" => TokenKind::Path,
            "bytes" => TokenKind::Bytes,
            "json" => TokenKind::Json,
            "object" => TokenKind::Object,
            "list" => TokenKind::List,
            "enum" => TokenKind::Enum,
            "optional" => TokenKind::Optional,
            "openapi" => TokenKind::OpenApi,
            "markdown" => TokenKind::Markdown,
            "xlsx" => TokenKind::Xlsx,
            "pdf" => TokenKind::Pdf,
            "true" => TokenKind::BoolLiteral(true),
            "false" => TokenKind::BoolLiteral(false),
            _ => TokenKind::Ident(lexeme.clone()),
        };

        Token::new(kind, lexeme, span)
    }

    fn advance(&mut self) -> char {
        let ch = self.source[self.current];
        self.current += 1;
        self.byte_offset += ch.len_utf8();

        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }

        ch
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source[self.current + 1]
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn make_token(&self, kind: TokenKind, lexeme: &str) -> Token {
        let span = Span::new(self.line, self.column, self.byte_offset, lexeme.len());
        Token::new(kind, lexeme.to_string(), span)
    }
}
