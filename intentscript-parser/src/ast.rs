// AST data structures for IntentScript
// Represents the language-level structure of IntentScript programs

use intentscript_core::Span;
use serde::{Deserialize, Serialize};

/// Root AST node representing a complete IntentScript file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct File {
    pub tasks: Vec<Task>,
}

/// A task definition with name, version, and sections
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub name: String,
    pub version: Option<Version>,
    pub sections: Vec<Section>,
    pub span: Span,
}

/// Version number for a task
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: Option<u32>,
}

/// Task sections (goal, input, constraints, etc.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Section {
    Goal(Expr),
    Input(Vec<InputDecl>),
    Constraints(Vec<ConstraintDecl>),
    OutputSchema(TypeExpr),
    Checks(Vec<CheckDecl>),
    Run(Pipeline),
}

/// Input declaration with name, type, and optional default value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputDecl {
    pub name: String,
    pub type_expr: TypeExpr,
    pub default: Option<Literal>,
    pub span: Span,
}

/// Constraint declaration with name and value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstraintDecl {
    pub name: String,
    pub value: ConstraintValue,
    pub span: Span,
}

/// Value of a constraint (on/off or literal or expression)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConstraintValue {
    On,
    Off,
    Literal(Literal),
    Expr(Expr),
}

/// Check declaration with name and arguments
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckDecl {
    pub name: String,
    pub args: Vec<Expr>,
    pub span: Span,
}

/// Pipeline of steps connected with ->
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pipeline {
    pub steps: Vec<Step>,
    pub span: Span,
}

/// A single step in a pipeline
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Step {
    Call(CallExpr),
    Ident(String, Span),
}

/// Expression types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Literal(Literal, Span),
    Ident(String, Span),
    Call(CallExpr),
}

/// Function call expression with name and arguments
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallExpr {
    pub name: String,
    pub args: Vec<Arg>,
    pub span: Span,
}

/// Function argument (named or positional)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Arg {
    Named { name: String, value: Expr },
    Positional(Expr),
}

/// Literal values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

/// Type expressions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeExpr {
    Primitive(PrimitiveType, Span),
    Object {
        fields: Vec<(String, TypeExpr)>,
        span: Span,
    },
    List(Box<TypeExpr>, Span),
    Enum(Vec<String>, Span),
    Optional(Box<TypeExpr>, Span),
    Domain(DomainType, Span),
}

/// Primitive types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimitiveType {
    Bool,
    Int,
    Float,
    Text,
    Url,
    Email,
    Path,
    Bytes,
    Json,
}

/// Domain-specific types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainType {
    OpenApi,
    Markdown,
    Xlsx,
    Pdf,
}
