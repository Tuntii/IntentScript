pub mod semantic;
pub mod lowering;
pub mod ir;

pub use semantic::{SemanticAnalyzer, Policy};
pub use lowering::Lowering;
pub use ir::*;
