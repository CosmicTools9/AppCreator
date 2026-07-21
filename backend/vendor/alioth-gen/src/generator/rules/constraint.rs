//! Constraint Definition Module
//!
//! **注意**：核心实现已迁移至 `runtime-engine` crate。
//! 本模块保留以维持向后兼容，所有类型均从 `runtime-engine` 重新导出。

pub use runtime_contract::expression::{
    extract_field_references, parse_constraint_expression, BinaryOp, Constraint, ConstraintExpr,
    ConstraintLevel, ConstraintLiteral, ConstraintParseError, ConstraintViolation, Constraints,
    UnaryOp,
};
