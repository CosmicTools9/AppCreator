//! 输出管理模块

pub mod diff;
pub mod fs;
pub mod memory;
pub mod protected;
pub mod writer;

pub use diff::{DiffEngine, DiffResult, DiskSnapshot, FileChange, FileInfo};
pub use fs::FileSystemWriter;
pub use memory::{CapturedFile, MemoryBufferWriter};
pub use protected::{
    check_conflicts, Conflict, ConflictReport, ConflictType, MarkerMetadata, MarkerParser,
    MergeEngine, MergeOptions, MergeResult, ProtectedRegion, GENERATED_MARKER_END,
    GENERATED_MARKER_START,
};
pub use writer::{add_ordering_markers, DryRunResult, OutputWriter, WriteError};
