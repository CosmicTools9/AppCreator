//! 生成器注册表，用于管理多个生成器

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::Generator;

/// 所有可用生成器的注册表
pub struct GeneratorRegistry {
    generators: RwLock<HashMap<String, Arc<dyn Generator>>>,
}

impl GeneratorRegistry {
    /// 创建新的空注册表
    pub fn new() -> Self {
        Self {
            generators: RwLock::new(HashMap::new()),
        }
    }

    /// 注册生成器
    pub fn register<G: Generator + 'static>(&self, generator: G) {
        let name = generator.name().to_string();
        let arc: Arc<dyn Generator> = Arc::new(generator);
        self.generators.write().unwrap().insert(name, arc);
    }

    /// 通过名称获取生成器
    pub fn get(&self, name: &str) -> Option<Arc<dyn Generator>> {
        self.generators.read().unwrap().get(name).cloned()
    }

    /// 检查生成器是否已注册
    pub fn has(&self, name: &str) -> bool {
        self.generators.read().unwrap().contains_key(name)
    }

    /// 获取所有已注册的生成器名称
    pub fn names(&self) -> Vec<String> {
        self.generators.read().unwrap().keys().cloned().collect()
    }

    /// 遍历所有生成器
    pub fn iter(&self) -> impl Iterator<Item = (String, Arc<dyn Generator>)> {
        self.generators
            .read()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), Arc::clone(v)))
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// 注销生成器
    pub fn unregister(&self, name: &str) -> Option<Arc<dyn Generator>> {
        self.generators.write().unwrap().remove(name)
    }

    /// 清空所有生成器
    pub fn clear(&self) {
        self.generators.write().unwrap().clear();
    }
}

impl Default for GeneratorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::{
        GenerateError, GeneratedOutput, GenerationMetadata, GeneratorModel, ValidationError,
    };

    struct MockGenerator;

    impl Generator for MockGenerator {
        fn name(&self) -> &'static str {
            "mock"
        }

        fn generate(&self, _model: &GeneratorModel) -> Result<GeneratedOutput, GenerateError> {
            Ok(GeneratedOutput {
                files: vec![],
                metadata: GenerationMetadata {
                    generator_name: "mock".to_string(),
                    entity_count: 0,
                    c_file_count: 0,
                },
            })
        }

        fn validate(&self, _model: &GeneratorModel) -> Result<(), ValidationError> {
            Ok(())
        }

        fn supports_incremental(&self) -> bool {
            false
        }

        fn file_extensions(&self) -> Vec<&'static str> {
            vec![".txt"]
        }
    }

    #[test]
    fn test_register_and_get() {
        let registry = GeneratorRegistry::new();
        registry.register(MockGenerator);

        assert!(registry.has("mock"));
        assert!(registry.get("mock").is_some());
        assert!(!registry.has("nonexistent"));
    }

    #[test]
    fn test_names() {
        let registry = GeneratorRegistry::new();
        registry.register(MockGenerator);

        let names = registry.names();
        assert_eq!(names.len(), 1);
        assert_eq!(names[0], "mock");
    }
}
