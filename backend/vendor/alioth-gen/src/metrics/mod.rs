//! Metrics module for tracking code generation performance

pub mod generation;

pub use generation::{init_metrics, record_generation, GenerationTimer, GenerationType};
