//! ontology-gen-bridge: MappingOutput → MetaModule adapter
//!
//! Bridges the OMP/skill ontology pipeline (MappingOutput from ontology-mapping)
//! to the code generation pipeline (MetaModule for alioth-gen's ModuleApiGenerator).

pub mod adapter;

use alioth_gen::generator::module::ModuleApiGenerator;
use ontology_mapping::output::MappingOutput;
use std::fs;
use std::path::Path;

/// Full pipeline: read MappingOutput JSON → convert to MetaModule → generate → write.
///
/// Called by both the CLI binary and AppAgent's Generating state.
pub fn generate_service(
    output: &MappingOutput,
    output_dir: &Path,
    module_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let module = adapter::mapping_output_to_meta_module(output, module_name)?;
    let generator = ModuleApiGenerator::new();
    let generated = generator.generate(&module)?;

    for file in &generated.files {
        let target = output_dir.join(&file.path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&target, &file.content)?;
        println!("  wrote {}", target.display());
    }
    println!("Generated {} files to {}", generated.files.len(), output_dir.display());
    Ok(())
}

pub use adapter::{mapping_output_to_meta_module, AdapterError};
