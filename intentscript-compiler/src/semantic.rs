// Semantic analysis for IntentScript
// Performs type checking, constraint validation, and symbol resolution

use intentscript_core::{Error, Span};
use intentscript_parser::{
    Arg, CallExpr, ConstraintDecl, ConstraintValue, Expr, File, InputDecl, Literal, Pipeline,
    PrimitiveType, Section, Step, Task, TypeExpr,
};
use std::collections::HashMap;

fn is_builtin_pipeline_step(name: &str) -> bool {
    matches!(
        name,
        "read_file"
            | "write_file"
            | "parse_openapi"
            | "parse_markdown"
            | "render_template"
            | "export_xlsx"
            | "export_pdf"
            | "validate"
            | "report"
    )
}

/// Symbol table for tracking inputs, constraints, and pipeline variables
#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    /// Input declarations with their types
    pub inputs: HashMap<String, TypeExpr>,
    /// Constraint declarations with their values
    pub constraints: HashMap<String, ConstraintValue>,
    /// Pipeline variables with their inferred types
    pub pipeline_vars: HashMap<String, TypeExpr>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an input declaration to the symbol table
    pub fn add_input(&mut self, name: String, type_expr: TypeExpr) {
        self.inputs.insert(name, type_expr);
    }

    /// Add a constraint declaration to the symbol table
    pub fn add_constraint(&mut self, name: String, value: ConstraintValue) {
        self.constraints.insert(name, value);
    }

    /// Add a pipeline variable to the symbol table
    pub fn add_pipeline_var(&mut self, name: String, type_expr: TypeExpr) {
        self.pipeline_vars.insert(name, type_expr);
    }

    /// Look up the type of an identifier
    pub fn lookup_type(&self, name: &str) -> Option<&TypeExpr> {
        self.inputs
            .get(name)
            .or_else(|| self.pipeline_vars.get(name))
    }
}

/// Type checker for IntentScript
pub struct TypeChecker {
    symbol_table: SymbolTable,
    errors: Vec<Error>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
        }
    }

    /// Get the symbol table
    pub fn symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    /// Get collected errors
    pub fn errors(&self) -> &[Error] {
        &self.errors
    }

    /// Add an error to the error list
    fn add_error(&mut self, error: Error) {
        self.errors.push(error);
    }

    /// Check if a type expression is valid
    pub fn check_type_expr(&mut self, type_expr: &TypeExpr) -> Result<(), Error> {
        match type_expr {
            TypeExpr::Primitive(_, _) => Ok(()),
            TypeExpr::Domain(_, _) => Ok(()),
            TypeExpr::List(inner, _) => self.check_type_expr(inner),
            TypeExpr::Optional(inner, _) => self.check_type_expr(inner),
            TypeExpr::Object { fields, .. } => {
                for (_, field_type) in fields {
                    self.check_type_expr(field_type)?;
                }
                Ok(())
            }
            TypeExpr::Enum(_, _) => Ok(()),
        }
    }

    /// Infer the type of a literal
    pub fn infer_literal_type(&self, literal: &Literal, span: Span) -> TypeExpr {
        match literal {
            Literal::String(_) => TypeExpr::Primitive(PrimitiveType::Text, span),
            Literal::Int(_) => TypeExpr::Primitive(PrimitiveType::Int, span),
            Literal::Float(_) => TypeExpr::Primitive(PrimitiveType::Float, span),
            Literal::Bool(_) => TypeExpr::Primitive(PrimitiveType::Bool, span),
        }
    }

    /// Infer the type of an expression
    pub fn infer_expr_type(&mut self, expr: &Expr) -> Result<TypeExpr, Error> {
        match expr {
            Expr::Literal(lit, span) => Ok(self.infer_literal_type(lit, *span)),
            Expr::Ident(name, span) => {
                if let Some(type_expr) = self.symbol_table.lookup_type(name) {
                    Ok(type_expr.clone())
                } else {
                    Err(Error::semantic(
                        *span,
                        format!("Undefined identifier: {}", name),
                    ))
                }
            }
            Expr::Call(call) => self.infer_call_type(call),
        }
    }

    /// Infer the type of a function call
    pub fn infer_call_type(&mut self, call: &CallExpr) -> Result<TypeExpr, Error> {
        // For now, we return a generic text type for function calls
        // In a full implementation, we would have a function signature table
        Ok(TypeExpr::Primitive(PrimitiveType::Text, call.span))
    }

    /// Check if two types are compatible
    pub fn types_compatible(&self, expected: &TypeExpr, actual: &TypeExpr) -> bool {
        match (expected, actual) {
            // Exact primitive match
            (TypeExpr::Primitive(p1, _), TypeExpr::Primitive(p2, _)) => p1 == p2,
            // Exact domain match
            (TypeExpr::Domain(d1, _), TypeExpr::Domain(d2, _)) => d1 == d2,
            // List compatibility
            (TypeExpr::List(inner1, _), TypeExpr::List(inner2, _)) => {
                self.types_compatible(inner1, inner2)
            }
            // Optional compatibility
            (TypeExpr::Optional(inner1, _), TypeExpr::Optional(inner2, _)) => {
                self.types_compatible(inner1, inner2)
            }
            // Non-optional can be assigned to optional
            (TypeExpr::Optional(inner, _), actual) => self.types_compatible(inner, actual),
            // Enum compatibility
            (TypeExpr::Enum(variants1, _), TypeExpr::Enum(variants2, _)) => {
                variants1 == variants2
            }
            // Object compatibility (structural)
            (
                TypeExpr::Object { fields: fields1, .. },
                TypeExpr::Object { fields: fields2, .. },
            ) => {
                if fields1.len() != fields2.len() {
                    return false;
                }
                for ((name1, type1), (name2, type2)) in fields1.iter().zip(fields2.iter()) {
                    if name1 != name2 || !self.types_compatible(type1, type2) {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }

    /// Check an expression against an expected type
    pub fn check_expr(&mut self, expr: &Expr, expected: &TypeExpr) -> Result<(), Error> {
        let actual = self.infer_expr_type(expr)?;
        if !self.types_compatible(expected, &actual) {
            let span = match expr {
                Expr::Literal(_, s) => *s,
                Expr::Ident(_, s) => *s,
                Expr::Call(c) => c.span,
            };
            return Err(Error::type_error(
                span,
                format!("{:?}", expected),
                format!("{:?}", actual),
            ));
        }
        Ok(())
    }

    /// Check a function call's argument types
    pub fn check_call(&mut self, call: &CallExpr) -> Result<(), Error> {
        // Check that all arguments are well-typed
        for arg in &call.args {
            match arg {
                Arg::Named { value, .. } => {
                    self.infer_expr_type(value)?;
                }
                Arg::Positional(expr) => {
                    self.infer_expr_type(expr)?;
                }
            }
        }
        Ok(())
    }

    /// Check a pipeline for type compatibility between steps
    pub fn check_pipeline(&mut self, pipeline: &Pipeline) -> Result<(), Error> {
        if pipeline.steps.is_empty() {
            return Ok(());
        }

        // Check each step
        for step in &pipeline.steps {
            match step {
                Step::Call(call) => {
                    self.check_call(call)?;
                }
                Step::Ident(name, span) => {
                    if !is_builtin_pipeline_step(name)
                        && self.symbol_table.lookup_type(name).is_none()
                    {
                        self.add_error(Error::semantic(
                            *span,
                            format!("Undefined identifier in pipeline: {}", name),
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Check optional type handling according to policy
    pub fn check_optional_handling(&mut self, type_expr: &TypeExpr, _span: Span) -> Result<(), Error> {
        // Check if optional types are used without explicit handling
        if let TypeExpr::Optional(_, _) = type_expr {
            // For now, we just verify that optional types are recognized
            // A full implementation would check that they are explicitly handled
            Ok(())
        } else {
            Ok(())
        }
    }
}

/// Policy rules for constraint resolution
#[derive(Debug, Clone, Default)]
pub struct Policy {
    /// Constraint rules defined by policy
    pub constraints: HashMap<String, ConstraintValue>,
    /// Whether to allow ambiguity resolution
    pub allow_ambiguity_resolution: bool,
}

impl Policy {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a policy constraint
    pub fn add_constraint(&mut self, name: String, value: ConstraintValue) {
        self.constraints.insert(name, value);
    }
}

/// Constraint solver for detecting contradictions and conflicts
pub struct ConstraintSolver {
    /// Task constraints
    task_constraints: HashMap<String, Vec<(ConstraintValue, Span)>>,
    /// Policy constraints
    policy: Policy,
    /// Collected errors
    errors: Vec<Error>,
}

impl ConstraintSolver {
    pub fn new(policy: Policy) -> Self {
        Self {
            task_constraints: HashMap::new(),
            policy,
            errors: Vec::new(),
        }
    }

    /// Add a task constraint
    pub fn add_task_constraint(&mut self, name: String, value: ConstraintValue, span: Span) {
        self.task_constraints
            .entry(name)
            .or_insert_with(Vec::new)
            .push((value, span));
    }

    /// Detect contradictions in task constraints (mutually exclusive constraints)
    pub fn detect_contradictions(&mut self) -> Result<(), Vec<Error>> {
        for (name, values) in &self.task_constraints {
            // Check for On/Off contradictions
            let has_on = values.iter().any(|(v, _)| matches!(v, ConstraintValue::On));
            let has_off = values.iter().any(|(v, _)| matches!(v, ConstraintValue::Off));

            if has_on && has_off {
                // Find the spans for both
                let on_span = values
                    .iter()
                    .find(|(v, _)| matches!(v, ConstraintValue::On))
                    .map(|(_, s)| *s)
                    .unwrap_or_default();
                let off_span = values
                    .iter()
                    .find(|(v, _)| matches!(v, ConstraintValue::Off))
                    .map(|(_, s)| *s)
                    .unwrap_or_default();

                self.errors.push(Error::constraint(
                    on_span,
                    format!(
                        "Contradictory constraints for '{}': declared as both 'on' (at {:?}) and 'off' (at {:?})",
                        name, on_span, off_span
                    ),
                ));
            }

            // Check for multiple literal/expression values
            let literal_or_expr_values: Vec<_> = values
                .iter()
                .filter(|(v, _)| {
                    matches!(v, ConstraintValue::Literal(_) | ConstraintValue::Expr(_))
                })
                .collect();

            if literal_or_expr_values.len() > 1 {
                let first_span = literal_or_expr_values[0].1;
                let second_span = literal_or_expr_values[1].1;
                self.errors.push(Error::constraint(
                    first_span,
                    format!(
                        "Multiple conflicting values for constraint '{}': declared at {:?} and {:?}",
                        name, first_span, second_span
                    ),
                ));
            }
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Check for conflicts between policy rules and task constraints
    pub fn check_policy_conflicts(&mut self) -> Result<(), Vec<Error>> {
        for (name, task_values) in &self.task_constraints {
            if let Some(policy_value) = self.policy.constraints.get(name) {
                // Check if any task constraint conflicts with policy
                for (task_value, span) in task_values {
                    if !self.values_compatible(policy_value, task_value) {
                        self.errors.push(Error::policy_violation(
                            *span,
                            format!(
                                "Task constraint '{}' = {:?} conflicts with policy rule '{}' = {:?}",
                                name, task_value, name, policy_value
                            ),
                        ));
                    }
                }
            }
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Check if two constraint values are compatible
    fn values_compatible(&self, policy_value: &ConstraintValue, task_value: &ConstraintValue) -> bool {
        match (policy_value, task_value) {
            (ConstraintValue::On, ConstraintValue::Off) => false,
            (ConstraintValue::Off, ConstraintValue::On) => false,
            (ConstraintValue::Literal(l1), ConstraintValue::Literal(l2)) => l1 == l2,
            // Policy allows task to be more specific
            (ConstraintValue::On, ConstraintValue::On) => true,
            (ConstraintValue::Off, ConstraintValue::Off) => true,
            // If policy doesn't specify, task can set it
            _ => true,
        }
    }

    /// Resolve ambiguities according to policy
    pub fn resolve_ambiguities(&mut self) -> Result<HashMap<String, ConstraintValue>, Vec<Error>> {
        let mut resolved = HashMap::new();

        for (name, values) in &self.task_constraints {
            if values.len() > 1 {
                // Ambiguous - multiple declarations
                if !self.policy.allow_ambiguity_resolution {
                    let first_span = values[0].1;
                    self.errors.push(Error::constraint(
                        first_span,
                        format!(
                            "Ambiguous constraint '{}': multiple declarations found and policy does not allow automatic resolution",
                            name
                        ),
                    ));
                } else {
                    // Policy allows resolution - use first value
                    resolved.insert(name.clone(), values[0].0.clone());
                }
            } else if values.len() == 1 {
                resolved.insert(name.clone(), values[0].0.clone());
            }
        }

        // Add policy constraints that weren't overridden
        for (name, value) in &self.policy.constraints {
            if !resolved.contains_key(name) {
                resolved.insert(name.clone(), value.clone());
            }
        }

        if self.errors.is_empty() {
            Ok(resolved)
        } else {
            Err(self.errors.clone())
        }
    }

    /// Get all errors
    pub fn errors(&self) -> &[Error] {
        &self.errors
    }

    /// Solve constraints: detect contradictions, check policy conflicts, resolve ambiguities
    pub fn solve(&mut self) -> Result<HashMap<String, ConstraintValue>, Vec<Error>> {
        // First detect contradictions
        if let Err(e) = self.detect_contradictions() {
            return Err(e);
        }

        // Then check policy conflicts
        if let Err(e) = self.check_policy_conflicts() {
            return Err(e);
        }

        // Finally resolve ambiguities
        self.resolve_ambiguities()
    }
}

/// Semantic analyzer for IntentScript
pub struct SemanticAnalyzer {
    type_checker: TypeChecker,
    policy: Policy,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            type_checker: TypeChecker::new(),
            policy: Policy::new(),
        }
    }

    /// Create a new semantic analyzer with a policy
    pub fn with_policy(policy: Policy) -> Self {
        Self {
            type_checker: TypeChecker::new(),
            policy,
        }
    }

    /// Analyze a file and return errors if any
    pub fn analyze(&mut self, file: &File) -> Result<(), Vec<Error>> {
        for task in &file.tasks {
            self.analyze_task(task)?;
        }
        Ok(())
    }

    /// Analyze a single task
    pub fn analyze_task(&mut self, task: &Task) -> Result<(), Vec<Error>> {
        // First pass: collect symbols
        for section in &task.sections {
            match section {
                Section::Input(inputs) => {
                    for input in inputs {
                        self.collect_input(input)?;
                    }
                }
                Section::Constraints(constraints) => {
                    for constraint in constraints {
                        self.collect_constraint(constraint);
                    }
                }
                _ => {}
            }
        }

        // Second pass: type check
        for section in &task.sections {
            match section {
                Section::Goal(expr) => {
                    if let Err(e) = self.type_checker.infer_expr_type(expr) {
                        self.type_checker.add_error(e);
                    }
                }
                Section::Input(inputs) => {
                    for input in inputs {
                        self.check_input(input)?;
                    }
                }
                Section::OutputSchema(type_expr) => {
                    if let Err(e) = self.type_checker.check_type_expr(type_expr) {
                        self.type_checker.add_error(e);
                    }
                }
                Section::Run(pipeline) => {
                    if let Err(e) = self.type_checker.check_pipeline(pipeline) {
                        self.type_checker.add_error(e);
                    }
                }
                _ => {}
            }
        }

        // Check constraints using the constraint solver
        self.solve_constraints(task)?;

        // Return errors if any
        if !self.type_checker.errors().is_empty() {
            return Err(self.type_checker.errors().to_vec());
        }

        Ok(())
    }

    /// Collect an input declaration
    fn collect_input(&mut self, input: &InputDecl) -> Result<(), Vec<Error>> {
        self.type_checker
            .symbol_table
            .add_input(input.name.clone(), input.type_expr.clone());
        Ok(())
    }

    /// Collect a constraint declaration
    fn collect_constraint(&mut self, constraint: &ConstraintDecl) {
        self.type_checker
            .symbol_table
            .add_constraint(constraint.name.clone(), constraint.value.clone());
    }

    /// Check an input declaration
    fn check_input(&mut self, input: &InputDecl) -> Result<(), Vec<Error>> {
        // Check that the type expression is valid
        if let Err(e) = self.type_checker.check_type_expr(&input.type_expr) {
            self.type_checker.add_error(e);
        }

        // Check optional type handling
        if let Err(e) = self
            .type_checker
            .check_optional_handling(&input.type_expr, input.span)
        {
            self.type_checker.add_error(e);
        }

        // If there's a default value, check it matches the type
        if let Some(default) = &input.default {
            let default_type = self
                .type_checker
                .infer_literal_type(default, input.span);
            if !self
                .type_checker
                .types_compatible(&input.type_expr, &default_type)
            {
                self.type_checker.add_error(Error::type_error(
                    input.span,
                    format!("{:?}", input.type_expr),
                    format!("{:?}", default_type),
                ));
            }
        }

        Ok(())
    }

    /// Solve constraints using the constraint solver
    fn solve_constraints(&mut self, task: &Task) -> Result<(), Vec<Error>> {
        let mut solver = ConstraintSolver::new(self.policy.clone());

        // Collect all constraint declarations from the task
        for section in &task.sections {
            if let Section::Constraints(constraints) = section {
                for constraint in constraints {
                    solver.add_task_constraint(
                        constraint.name.clone(),
                        constraint.value.clone(),
                        constraint.span,
                    );
                }
            }
        }

        // Solve constraints
        match solver.solve() {
            Ok(_resolved) => {
                // Constraints are consistent
                Ok(())
            }
            Err(errors) => {
                // Add all constraint errors to type checker
                for error in &errors {
                    self.type_checker.add_error(error.clone());
                }
                Err(errors)
            }
        }
    }

    /// Get the symbol table
    pub fn symbol_table(&self) -> &SymbolTable {
        self.type_checker.symbol_table()
    }

    /// Get collected errors
    pub fn errors(&self) -> &[Error] {
        self.type_checker.errors()
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
