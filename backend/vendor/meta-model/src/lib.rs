//! Alioth 元数据模型契约
//!
//! 仅包含纯数据类型，无实现、无数据库依赖、无复杂业务逻辑。
//! 由 `alioth-gen` 生成代码时使用，由 `meta-services` 运行时服务使用。

pub mod version;

pub mod exception;
pub mod ir1;
pub mod ir2;
pub mod ontology;
pub mod permission;
pub mod quality;
