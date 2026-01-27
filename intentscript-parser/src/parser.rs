use crate::ast::*;
use crate::lexer::{Lexer, Token, TokenKind};
use intentscript_core::{Error, Span};

/// Parser for IntentScript source code
/// Converts token stream into AST using recursive descent parsing
pub struct Parser {
    lexer: Lexer,
    current: Token,
    previous: Token,
    errors: Vec<Error>,
}

impl Parser {
    /// Create a new parser for the given source code
    pub fn new(source: &str) -> Self {
        let mut lexer = Lexer::new(source);
        let first_token = lexer.next_token();
        let eof_token = Token::new(
            TokenKind::Eof,
            String::new(),
            Span::new(1, 1, 0, 0),
        );
        
        Self {
            lexer,
            current: first_token,
            previous: eof_token,
            errors: Vec::new(),
        }
    }

    /// Peek at the current token without consuming it
    pub fn peek(&self) -> &Token {
        &self.current
    }

    /// Peek at the current token kind
    pub fn peek_kind(&self) -> &TokenKind {
        &self.current.kind
    }

    /// Advance to the next token and return the previous one
    pub fn next_token(&mut self) -> Token {
        let mut next = self.lexer.next_token();
        
        // Skip whitespace and comments
        while matches!(next.kind, TokenKind::Whitespace | TokenKind::Comment(_)) {
            next = self.lexer.next_token();
        }
        
        self.previous = std::mem::replace(&mut self.current, next);
        self.previous.clone()
    }

    /// Expect a specific token kind and consume it, or report an error
    pub fn expect(&mut self, expected: TokenKind, context: &str) -> Result<Token, Error> {
        if std::mem::discriminant(&self.current.kind) == std::mem::discriminant(&expected) {
            Ok(self.next_token())
        } else {
            let error = Error::parse(
                self.current.span.clone(),
                format!(
                    "Expected {} in {}, found {}",
                    token_kind_name(&expected),
                    context,
                    token_kind_name(&self.current.kind)
                ),
            );
            Err(error)
        }
    }

    /// Check if current token matches the given kind
    pub fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.current.kind) == std::mem::discriminant(kind)
    }

    /// Check if current token is EOF
    pub fn is_at_end(&self) -> bool {
        matches!(self.current.kind, TokenKind::Eof)
    }

    /// Consume a token if it matches the expected kind
    pub fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(&kind) {
            self.next_token();
            true
        } else {
            false
        }
    }

    /// Record an error and continue parsing (for error recovery)
    pub fn error(&mut self, error: Error) {
        self.errors.push(error);
    }

    /// Synchronize parser state after an error
    /// Advances to the next section boundary or statement
    pub fn synchronize(&mut self) {
        self.next_token();

        while !self.is_at_end() {
            // Synchronize on section keywords
            match self.current.kind {
                TokenKind::Goal
                | TokenKind::Input
                | TokenKind::Constraints
                | TokenKind::OutputSchema
                | TokenKind::Checks
                | TokenKind::Run
                | TokenKind::Task => return,
                _ => {
                    self.next_token();
                }
            }
        }
    }

    /// Get all accumulated errors
    pub fn get_errors(&self) -> &[Error] {
        &self.errors
    }

    /// Check if any errors have been recorded
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Parse a complete IntentScript file
    pub fn parse_file(&mut self) -> Result<File, Vec<Error>> {
        let mut tasks = Vec::new();

        while !self.is_at_end() {
            match self.parse_task() {
                Ok(task) => tasks.push(task),
                Err(e) => {
                    self.error(e);
                    self.synchronize();
                }
            }
        }

        if self.has_errors() {
            Err(self.errors.clone())
        } else {
            Ok(File { tasks })
        }
    }

    /// Parse a task declaration
    /// Syntax: task "name" v1.0 { sections... }
    pub fn parse_task(&mut self) -> Result<Task, Error> {
        let start_span = self.current.span.clone();

        // Expect 'task' keyword
        self.expect(TokenKind::Task, "task declaration")?;

        // Expect task name (string literal)
        let name = match &self.current.kind {
            TokenKind::StringLiteral(s) => {
                let name = s.clone();
                self.next_token();
                name
            }
            _ => {
                return Err(Error::parse(
                    self.current.span.clone(),
                    "Expected task name (string literal) after 'task'",
                ));
            }
        };

        // Optional version - check for identifier starting with 'v' followed by numbers
        let version = if let TokenKind::Ident(v) = &self.current.kind {
            if v.starts_with('v') && v.len() > 1 {
                let version_str = v[1..].to_string(); // Clone the string to avoid borrow issues
                self.next_token();
                Some(self.parse_version(&version_str)?)
            } else {
                None
            }
        } else {
            None
        };

        // Expect opening brace
        self.expect(TokenKind::LeftBrace, "task body")?;

        // Parse sections
        let mut sections = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            match self.parse_section() {
                Ok(section) => sections.push(section),
                Err(e) => {
                    self.error(e);
                    self.synchronize();
                }
            }
        }

        // Expect closing brace
        let end_span = self.current.span.clone();
        self.expect(TokenKind::RightBrace, "task body")?;

        let span = Span::new(
            start_span.line,
            start_span.column,
            start_span.offset,
            end_span.offset + end_span.length - start_span.offset,
        );

        Ok(Task {
            name,
            version,
            sections,
            span,
        })
    }

    /// Parse a version string like "1.0" or "1.0.2"
    fn parse_version(&self, version_str: &str) -> Result<Version, Error> {
        let parts: Vec<&str> = version_str.split('.').collect();
        
        if parts.is_empty() || parts.len() > 3 {
            return Err(Error::parse(
                self.previous.span.clone(),
                format!("Invalid version format: {}", version_str),
            ));
        }

        let major = parts[0].parse::<u32>().map_err(|_| {
            Error::parse(
                self.previous.span.clone(),
                format!("Invalid major version number: {}", parts[0]),
            )
        })?;

        let minor = if parts.len() > 1 {
            parts[1].parse::<u32>().map_err(|_| {
                Error::parse(
                    self.previous.span.clone(),
                    format!("Invalid minor version number: {}", parts[1]),
                )
            })?
        } else {
            0
        };

        let patch = if parts.len() > 2 {
            Some(parts[2].parse::<u32>().map_err(|_| {
                Error::parse(
                    self.previous.span.clone(),
                    format!("Invalid patch version number: {}", parts[2]),
                )
            })?)
        } else {
            None
        };

        Ok(Version {
            major,
            minor,
            patch,
        })
    }

    /// Parse a section (goal, input, constraints, etc.)
    pub fn parse_section(&mut self) -> Result<Section, Error> {
        match &self.current.kind {
            TokenKind::Goal => self.parse_goal(),
            TokenKind::Input => self.parse_input(),
            TokenKind::Constraints => self.parse_constraints(),
            TokenKind::OutputSchema => self.parse_output_schema(),
            TokenKind::Checks => self.parse_checks(),
            TokenKind::Run => self.parse_run(),
            _ => Err(Error::parse(
                self.current.span.clone(),
                format!(
                    "Expected section keyword (goal, input, constraints, output_schema, checks, run), found {}",
                    token_kind_name(&self.current.kind)
                ),
            )),
        }
    }

    /// Parse goal section
    /// Syntax: goal: <expr>
    fn parse_goal(&mut self) -> Result<Section, Error> {
        self.expect(TokenKind::Goal, "goal section")?;
        self.expect(TokenKind::Colon, "goal section")?;
        let expr = self.parse_expr()?;
        Ok(Section::Goal(expr))
    }

    /// Parse input section
    /// Syntax: input: name: type or input: { name: type, ... }
    fn parse_input(&mut self) -> Result<Section, Error> {
        self.expect(TokenKind::Input, "input section")?;
        self.expect(TokenKind::Colon, "input section")?;

        let mut inputs = Vec::new();

        // Check for block-style or inline-style
        if self.check(&TokenKind::LeftBrace) {
            // Block style: input: { name: type, ... }
            self.next_token(); // consume '{'

            while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
                inputs.push(self.parse_input_decl()?);

                if !self.check(&TokenKind::RightBrace) {
                    self.expect(TokenKind::Comma, "input declarations")?;
                }
            }

            self.expect(TokenKind::RightBrace, "input block")?;
        } else {
            // Inline style: input: name: type
            inputs.push(self.parse_input_decl()?);
        }

        Ok(Section::Input(inputs))
    }

    /// Parse a single input declaration
    /// Syntax: name: type or name: type = default
    fn parse_input_decl(&mut self) -> Result<InputDecl, Error> {
        let start_span = self.current.span.clone();

        // Parse name
        let name = match &self.current.kind {
            TokenKind::Ident(id) => {
                let name = id.clone();
                self.next_token();
                name
            }
            _ => {
                return Err(Error::parse(
                    self.current.span.clone(),
                    "Expected identifier for input name",
                ));
            }
        };

        self.expect(TokenKind::Colon, "input declaration")?;

        // Parse type
        let type_expr = self.parse_type_expr()?;

        // Optional default value
        let default = if self.match_token(TokenKind::Equal) {
            Some(self.parse_literal()?)
        } else {
            None
        };

        let end_span = self.previous.span.clone();
        let span = Span::new(
            start_span.line,
            start_span.column,
            start_span.offset,
            end_span.offset + end_span.length - start_span.offset,
        );

        Ok(InputDecl {
            name,
            type_expr,
            default,
            span,
        })
    }

    /// Parse constraints section
    /// Syntax: constraints: { name = value, ... }
    fn parse_constraints(&mut self) -> Result<Section, Error> {
        self.expect(TokenKind::Constraints, "constraints section")?;
        self.expect(TokenKind::Colon, "constraints section")?;
        self.expect(TokenKind::LeftBrace, "constraints block")?;

        let mut constraints = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            constraints.push(self.parse_constraint_decl()?);

            if !self.check(&TokenKind::RightBrace) {
                self.expect(TokenKind::Comma, "constraint declarations")?;
            }
        }

        self.expect(TokenKind::RightBrace, "constraints block")?;

        Ok(Section::Constraints(constraints))
    }

    /// Parse a single constraint declaration
    /// Syntax: name = value
    fn parse_constraint_decl(&mut self) -> Result<ConstraintDecl, Error> {
        let start_span = self.current.span.clone();

        // Parse name
        let name = match &self.current.kind {
            TokenKind::Ident(id) => {
                let name = id.clone();
                self.next_token();
                name
            }
            _ => {
                return Err(Error::parse(
                    self.current.span.clone(),
                    "Expected identifier for constraint name",
                ));
            }
        };

        self.expect(TokenKind::Equal, "constraint declaration")?;

        // Parse value (on, off, literal, or expression)
        let value = match &self.current.kind {
            TokenKind::Ident(id) if id == "on" => {
                self.next_token();
                ConstraintValue::On
            }
            TokenKind::Ident(id) if id == "off" => {
                self.next_token();
                ConstraintValue::Off
            }
            _ => {
                // Try to parse as literal first, then as expression
                if let Ok(lit) = self.parse_literal() {
                    ConstraintValue::Literal(lit)
                } else {
                    let expr = self.parse_expr()?;
                    ConstraintValue::Expr(expr)
                }
            }
        };

        let end_span = self.previous.span.clone();
        let span = Span::new(
            start_span.line,
            start_span.column,
            start_span.offset,
            end_span.offset + end_span.length - start_span.offset,
        );

        Ok(ConstraintDecl { name, value, span })
    }

    /// Parse output_schema section
    /// Syntax: output_schema: <type>
    fn parse_output_schema(&mut self) -> Result<Section, Error> {
        self.expect(TokenKind::OutputSchema, "output_schema section")?;
        self.expect(TokenKind::Colon, "output_schema section")?;
        let type_expr = self.parse_type_expr()?;
        Ok(Section::OutputSchema(type_expr))
    }

    /// Parse checks section
    /// Syntax: checks: { check_name(args), ... }
    fn parse_checks(&mut self) -> Result<Section, Error> {
        self.expect(TokenKind::Checks, "checks section")?;
        self.expect(TokenKind::Colon, "checks section")?;
        self.expect(TokenKind::LeftBrace, "checks block")?;

        let mut checks = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            checks.push(self.parse_check_decl()?);

            if !self.check(&TokenKind::RightBrace) {
                self.expect(TokenKind::Comma, "check declarations")?;
            }
        }

        self.expect(TokenKind::RightBrace, "checks block")?;

        Ok(Section::Checks(checks))
    }

    /// Parse a single check declaration
    /// Syntax: check_name(arg1, arg2, ...)
    fn parse_check_decl(&mut self) -> Result<CheckDecl, Error> {
        let start_span = self.current.span.clone();

        // Parse check name
        let name = match &self.current.kind {
            TokenKind::Ident(id) => {
                let name = id.clone();
                self.next_token();
                name
            }
            _ => {
                return Err(Error::parse(
                    self.current.span.clone(),
                    "Expected identifier for check name",
                ));
            }
        };

        // Parse arguments
        self.expect(TokenKind::LeftParen, "check arguments")?;

        let mut args = Vec::new();
        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            args.push(self.parse_expr()?);

            if !self.check(&TokenKind::RightParen) {
                self.expect(TokenKind::Comma, "check arguments")?;
            }
        }

        let end_span = self.current.span.clone();
        self.expect(TokenKind::RightParen, "check arguments")?;

        let span = Span::new(
            start_span.line,
            start_span.column,
            start_span.offset,
            end_span.offset + end_span.length - start_span.offset,
        );

        Ok(CheckDecl { name, args, span })
    }

    /// Parse run section
    /// Syntax: run: <pipeline>
    fn parse_run(&mut self) -> Result<Section, Error> {
        self.expect(TokenKind::Run, "run section")?;
        self.expect(TokenKind::Colon, "run section")?;
        let pipeline = self.parse_pipeline()?;
        Ok(Section::Run(pipeline))
    }

    /// Parse an expression (literal, identifier, or call)
    pub fn parse_expr(&mut self) -> Result<Expr, Error> {
        let span = self.current.span.clone();

        match &self.current.kind {
            TokenKind::StringLiteral(_)
            | TokenKind::IntLiteral(_)
            | TokenKind::FloatLiteral(_)
            | TokenKind::BoolLiteral(_) => {
                let lit = self.parse_literal()?;
                Ok(Expr::Literal(lit, span))
            }
            TokenKind::Ident(id) => {
                let name = id.clone();
                self.next_token();

                // Check if this is a function call
                if self.check(&TokenKind::LeftParen) {
                    let call_expr = self.parse_call_expr_with_name(name, span)?;
                    Ok(Expr::Call(call_expr))
                } else {
                    Ok(Expr::Ident(name, span))
                }
            }
            _ => Err(Error::parse(
                self.current.span.clone(),
                format!(
                    "Expected expression (literal, identifier, or call), found {}",
                    token_kind_name(&self.current.kind)
                ),
            )),
        }
    }

    /// Parse a literal value
    fn parse_literal(&mut self) -> Result<Literal, Error> {
        match &self.current.kind {
            TokenKind::StringLiteral(s) => {
                let lit = Literal::String(s.clone());
                self.next_token();
                Ok(lit)
            }
            TokenKind::IntLiteral(i) => {
                let lit = Literal::Int(*i);
                self.next_token();
                Ok(lit)
            }
            TokenKind::FloatLiteral(f) => {
                let lit = Literal::Float(*f);
                self.next_token();
                Ok(lit)
            }
            TokenKind::BoolLiteral(b) => {
                let lit = Literal::Bool(*b);
                self.next_token();
                Ok(lit)
            }
            _ => Err(Error::parse(
                self.current.span.clone(),
                format!(
                    "Expected literal value, found {}",
                    token_kind_name(&self.current.kind)
                ),
            )),
        }
    }

    /// Parse a function call expression
    /// Syntax: name(arg1, arg2, ...) or name(key1=val1, key2=val2, ...)
    pub fn parse_call_expr(&mut self) -> Result<CallExpr, Error> {
        let start_span = self.current.span.clone();

        // Parse function name
        let name = match &self.current.kind {
            TokenKind::Ident(id) => {
                let name = id.clone();
                self.next_token();
                name
            }
            _ => {
                return Err(Error::parse(
                    self.current.span.clone(),
                    "Expected identifier for function name",
                ));
            }
        };

        self.parse_call_expr_with_name(name, start_span)
    }

    /// Parse call expression when we already have the name
    fn parse_call_expr_with_name(&mut self, name: String, start_span: Span) -> Result<CallExpr, Error> {
        self.expect(TokenKind::LeftParen, "function call")?;

        let mut args = Vec::new();

        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            // Check if this is a named argument (name = value)
            if let TokenKind::Ident(id) = &self.current.kind {
                let arg_name = id.clone();
                let next_pos = self.current.span.clone();
                self.next_token();

                if self.match_token(TokenKind::Equal) {
                    // Named argument
                    let value = self.parse_expr()?;
                    args.push(Arg::Named {
                        name: arg_name,
                        value,
                    });
                } else {
                    // Positional argument (identifier)
                    args.push(Arg::Positional(Expr::Ident(arg_name, next_pos)));
                }
            } else {
                // Positional argument (non-identifier expression)
                let expr = self.parse_expr()?;
                args.push(Arg::Positional(expr));
            }

            if !self.check(&TokenKind::RightParen) {
                self.expect(TokenKind::Comma, "function arguments")?;
            }
        }

        let end_span = self.current.span.clone();
        self.expect(TokenKind::RightParen, "function call")?;

        let span = Span::new(
            start_span.line,
            start_span.column,
            start_span.offset,
            end_span.offset + end_span.length - start_span.offset,
        );

        Ok(CallExpr { name, args, span })
    }

    /// Parse a type expression
    pub fn parse_type_expr(&mut self) -> Result<TypeExpr, Error> {
        let span = self.current.span.clone();

        match &self.current.kind {
            // Primitive types
            TokenKind::Bool => {
                self.next_token();
                Ok(TypeExpr::Primitive(PrimitiveType::Bool, span))
            }
            TokenKind::Int => {
                self.next_token();
                Ok(TypeExpr::Primitive(PrimitiveType::Int, span))
            }
            TokenKind::Float => {
                self.next_token();
                Ok(TypeExpr::Primitive(PrimitiveType::Float, span))
            }
            TokenKind::Text => {
                self.next_token();
                Ok(TypeExpr::Primitive(PrimitiveType::Text, span))
            }
            TokenKind::Url => {
                self.next_token();
                Ok(TypeExpr::Primitive(PrimitiveType::Url, span))
            }
            TokenKind::Email => {
                self.next_token();
                Ok(TypeExpr::Primitive(PrimitiveType::Email, span))
            }
            TokenKind::Path => {
                self.next_token();
                Ok(TypeExpr::Primitive(PrimitiveType::Path, span))
            }
            TokenKind::Bytes => {
                self.next_token();
                Ok(TypeExpr::Primitive(PrimitiveType::Bytes, span))
            }
            TokenKind::Json => {
                self.next_token();
                Ok(TypeExpr::Primitive(PrimitiveType::Json, span))
            }
            // Domain types
            TokenKind::OpenApi => {
                self.next_token();
                Ok(TypeExpr::Domain(DomainType::OpenApi, span))
            }
            TokenKind::Markdown => {
                self.next_token();
                Ok(TypeExpr::Domain(DomainType::Markdown, span))
            }
            TokenKind::Xlsx => {
                self.next_token();
                Ok(TypeExpr::Domain(DomainType::Xlsx, span))
            }
            TokenKind::Pdf => {
                self.next_token();
                Ok(TypeExpr::Domain(DomainType::Pdf, span))
            }
            // Structured types
            TokenKind::Object => {
                self.next_token();
                self.expect(TokenKind::LeftBrace, "object type")?;

                let mut fields = Vec::new();
                while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
                    // Parse field name
                    let field_name = match &self.current.kind {
                        TokenKind::Ident(id) => {
                            let name = id.clone();
                            self.next_token();
                            name
                        }
                        _ => {
                            return Err(Error::parse(
                                self.current.span.clone(),
                                "Expected field name in object type",
                            ));
                        }
                    };

                    self.expect(TokenKind::Colon, "object field")?;

                    // Parse field type
                    let field_type = self.parse_type_expr()?;
                    fields.push((field_name, field_type));

                    if !self.check(&TokenKind::RightBrace) {
                        self.expect(TokenKind::Comma, "object fields")?;
                    }
                }

                let end_span = self.current.span.clone();
                self.expect(TokenKind::RightBrace, "object type")?;

                let full_span = Span::new(
                    span.line,
                    span.column,
                    span.offset,
                    end_span.offset + end_span.length - span.offset,
                );

                Ok(TypeExpr::Object {
                    fields,
                    span: full_span,
                })
            }
            TokenKind::List => {
                self.next_token();
                self.expect(TokenKind::LeftBracket, "list type")?;
                let element_type = self.parse_type_expr()?;
                let end_span = self.current.span.clone();
                self.expect(TokenKind::RightBracket, "list type")?;

                let full_span = Span::new(
                    span.line,
                    span.column,
                    span.offset,
                    end_span.offset + end_span.length - span.offset,
                );

                Ok(TypeExpr::List(Box::new(element_type), full_span))
            }
            TokenKind::Enum => {
                self.next_token();
                self.expect(TokenKind::LeftBracket, "enum type")?;

                let mut variants = Vec::new();
                while !self.check(&TokenKind::RightBracket) && !self.is_at_end() {
                    match &self.current.kind {
                        TokenKind::StringLiteral(s) => {
                            variants.push(s.clone());
                            self.next_token();
                        }
                        _ => {
                            return Err(Error::parse(
                                self.current.span.clone(),
                                "Expected string literal for enum variant",
                            ));
                        }
                    }

                    if !self.check(&TokenKind::RightBracket) {
                        self.expect(TokenKind::Comma, "enum variants")?;
                    }
                }

                let end_span = self.current.span.clone();
                self.expect(TokenKind::RightBracket, "enum type")?;

                let full_span = Span::new(
                    span.line,
                    span.column,
                    span.offset,
                    end_span.offset + end_span.length - span.offset,
                );

                Ok(TypeExpr::Enum(variants, full_span))
            }
            TokenKind::Optional => {
                self.next_token();
                self.expect(TokenKind::LeftBracket, "optional type")?;
                let inner_type = self.parse_type_expr()?;
                let end_span = self.current.span.clone();
                self.expect(TokenKind::RightBracket, "optional type")?;

                let full_span = Span::new(
                    span.line,
                    span.column,
                    span.offset,
                    end_span.offset + end_span.length - span.offset,
                );

                Ok(TypeExpr::Optional(Box::new(inner_type), full_span))
            }
            _ => Err(Error::parse(
                self.current.span.clone(),
                format!(
                    "Expected type expression, found {}",
                    token_kind_name(&self.current.kind)
                ),
            )),
        }
    }

    /// Parse a pipeline (sequence of steps connected with ->)
    pub fn parse_pipeline(&mut self) -> Result<Pipeline, Error> {
        let start_span = self.current.span.clone();
        let mut steps = Vec::new();

        // Parse first step
        steps.push(self.parse_step()?);

        // Parse remaining steps connected with ->
        while self.match_token(TokenKind::Arrow) {
            steps.push(self.parse_step()?);
        }

        let end_span = self.previous.span.clone();
        let span = Span::new(
            start_span.line,
            start_span.column,
            start_span.offset,
            end_span.offset + end_span.length - start_span.offset,
        );

        Ok(Pipeline { steps, span })
    }

    /// Parse a single pipeline step (identifier or call)
    fn parse_step(&mut self) -> Result<Step, Error> {
        let span = self.current.span.clone();

        match &self.current.kind {
            TokenKind::Ident(id) => {
                let name = id.clone();
                self.next_token();

                // Check if this is a function call
                if self.check(&TokenKind::LeftParen) {
                    let call_expr = self.parse_call_expr_with_name(name, span)?;
                    Ok(Step::Call(call_expr))
                } else {
                    Ok(Step::Ident(name, span))
                }
            }
            _ => Err(Error::parse(
                self.current.span.clone(),
                format!(
                    "Expected pipeline step (identifier or call), found {}",
                    token_kind_name(&self.current.kind)
                ),
            )),
        }
    }
}

/// Helper function to get a human-readable name for a token kind
fn token_kind_name(kind: &TokenKind) -> String {
    match kind {
        TokenKind::Task => "keyword 'task'".to_string(),
        TokenKind::Goal => "keyword 'goal'".to_string(),
        TokenKind::Input => "keyword 'input'".to_string(),
        TokenKind::Constraints => "keyword 'constraints'".to_string(),
        TokenKind::OutputSchema => "keyword 'output_schema'".to_string(),
        TokenKind::Checks => "keyword 'checks'".to_string(),
        TokenKind::Run => "keyword 'run'".to_string(),
        TokenKind::LeftBrace => "'{'".to_string(),
        TokenKind::RightBrace => "'}'".to_string(),
        TokenKind::LeftParen => "'('".to_string(),
        TokenKind::RightParen => "')'".to_string(),
        TokenKind::LeftBracket => "'['".to_string(),
        TokenKind::RightBracket => "']'".to_string(),
        TokenKind::Colon => "':'".to_string(),
        TokenKind::Comma => "','".to_string(),
        TokenKind::Equal => "'='".to_string(),
        TokenKind::Arrow => "'->'".to_string(),
        TokenKind::Pipe => "'|>'".to_string(),
        TokenKind::StringLiteral(_) => "string literal".to_string(),
        TokenKind::IntLiteral(_) => "integer literal".to_string(),
        TokenKind::FloatLiteral(_) => "float literal".to_string(),
        TokenKind::BoolLiteral(_) => "boolean literal".to_string(),
        TokenKind::Ident(_) => "identifier".to_string(),
        TokenKind::Eof => "end of file".to_string(),
        _ => format!("{:?}", kind),
    }
}
