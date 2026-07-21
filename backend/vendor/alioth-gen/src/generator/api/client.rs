//! Frontend API Client Generator
//!
//! Generates TypeScript API client code for frontend applications.

use crate::generator::ir::{GeneratorEntity, GeneratorModel, PrimaryKeyType};
use crate::generator::{
    GenerateError, GeneratedFile, GeneratedOutput, GenerationMetadata, Generator,
};

/// API client generator options
#[derive(Debug, Clone)]
pub struct ClientGeneratorOptions {
    /// Client type: 'fetch' or 'axios'
    pub client_type: ClientType,
    /// Include React Query hooks
    pub include_react_query: bool,
    /// Base URL for API
    pub base_url: String,
}

#[derive(Debug, Clone)]
pub enum ClientType {
    Fetch,
    Axios,
}

impl Default for ClientGeneratorOptions {
    fn default() -> Self {
        Self {
            client_type: ClientType::Fetch,
            include_react_query: true,
            base_url: "/api".to_string(),
        }
    }
}

/// Frontend API client generator
pub struct FrontendClientGenerator {
    options: ClientGeneratorOptions,
}

impl FrontendClientGenerator {
    /// Create a new client generator with default options
    pub fn new() -> Self {
        Self {
            options: ClientGeneratorOptions::default(),
        }
    }

    /// Create with custom options
    pub fn with_options(options: ClientGeneratorOptions) -> Self {
        Self { options }
    }

    /// Generate API client for the model
    pub fn generate(&self, model: &GeneratorModel) -> Result<GeneratedOutput, GenerateError> {
        let mut files = Vec::new();

        // Generate client for each entity
        for entity in &model.entities {
            let client_code = self.generate_entity_client(entity);
            files.push(GeneratedFile {
                path: format!("api/{}.client.ts", entity.name.kebab).into(),
                content: client_code,
                checksum: String::new(),
            });

            // Generate React Query hooks if enabled
            if self.options.include_react_query {
                let hooks_code = self.generate_react_query_hooks(entity);
                files.push(GeneratedFile {
                    path: format!("api/{}.hooks.ts", entity.name.kebab).into(),
                    content: hooks_code,
                    checksum: String::new(),
                });
            }
        }

        // Generate index file
        let index_code = self.generate_index_file(model);
        files.push(GeneratedFile {
            path: "api/index.ts".into(),
            content: index_code,
            checksum: String::new(),
        });

        // Generate types file
        let types_code = self.generate_types_file(model);
        files.push(GeneratedFile {
            path: "api/types.ts".into(),
            content: types_code,
            checksum: String::new(),
        });

        let c_file_count = files.len();

        Ok(GeneratedOutput {
            files,
            metadata: GenerationMetadata {
                generator_name: "frontend_client".to_string(),
                entity_count: model.entities.len(),
                c_file_count,
            },
        })
    }

    /// Generate API client for an entity
    fn generate_entity_client(&self, entity: &GeneratorEntity) -> String {
        let entity_name = &entity.name.pascal;
        let plural_kebab = &entity.name.plural_kebab;
        let base_path = format!("{}/{}", self.options.base_url, plural_kebab);
        let id_type = match entity.primary_key_type {
            PrimaryKeyType::BigInt => "number",
            PrimaryKeyType::Uuid => "string",
        };

        let request_impl = match self.options.client_type {
            ClientType::Fetch => self.generate_fetch_impl(entity_name, &base_path, id_type),
            ClientType::Axios => self.generate_axios_impl(entity_name, &base_path, id_type),
        };

        format!(
            r#"//! API Client for {entity_name}

import {{ {entity_name}, {entity_name}Input }} from '../schemas/{entity_kebab}.schema';

const BASE_URL = '{base_path}';

{request_impl}
"#,
            entity_name = entity_name,
            entity_kebab = entity.name.kebab,
            base_path = base_path,
            request_impl = request_impl,
        )
    }

    /// Generate Fetch API implementation
    fn generate_fetch_impl(&self, entity_name: &str, _base_path: &str, id_type: &str) -> String {
        format!(
            r#"export class {entity_name}Client {{
  private baseUrl: string;

  constructor(baseUrl: string = BASE_URL) {{
    this.baseUrl = baseUrl;
  }}

  /** List all {entity_plural} */
  async list(): Promise<{entity_name}[]> {{
    const response = await fetch(this.baseUrl);
    if (!response.ok) throw new Error(`Failed to fetch {entity_plural}: ${{response.statusText}}`);
    return response.json();
  }}

  /** Get {entity_name} by ID */
  async get(id: {id_type}): Promise<{entity_name}> {{
    const response = await fetch(`${{this.baseUrl}}/${{id}}`);
    if (!response.ok) throw new Error(`{entity_name} not found: ${{response.statusText}}`);
    return response.json();
  }}

  /** Create a new {entity_name} */
  async create(data: {entity_name}Input): Promise<{entity_name}> {{
    const response = await fetch(this.baseUrl, {{
      method: 'POST',
      headers: {{ 'Content-Type': 'application/json' }},
      body: JSON.stringify(data),
    }});
    if (!response.ok) throw new Error(`Failed to create {entity_name}: ${{response.statusText}}`);
    return response.json();
  }}

  /** Update {entity_name} */
  async update(id: {id_type}, data: {entity_name}Input): Promise<{entity_name}> {{
    const response = await fetch(`${{this.baseUrl}}/${{id}}`, {{
      method: 'PUT',
      headers: {{ 'Content-Type': 'application/json' }},
      body: JSON.stringify(data),
    }});
    if (!response.ok) throw new Error(`Failed to update {entity_name}: ${{response.statusText}}`);
    return response.json();
  }}

  /** Delete {entity_name} */
  async delete(id: {id_type}): Promise<void> {{
    const response = await fetch(`${{this.baseUrl}}/${{id}}`, {{
      method: 'DELETE',
    }});
    if (!response.ok) throw new Error(`Failed to delete {entity_name}: ${{response.statusText}}`);
  }}
}}

// Default client instance
export const {entity_camel}Client = new {entity_name}Client();
"#,
            entity_name = entity_name,
            entity_plural = entity_name.to_lowercase() + "s",
            entity_camel = entity_name[..1].to_lowercase() + &entity_name[1..],
            id_type = id_type,
        )
    }

    /// Generate Axios implementation
    fn generate_axios_impl(&self, entity_name: &str, _base_path: &str, id_type: &str) -> String {
        format!(
            r#"import axios from 'axios';

const api = axios.create({{
  baseURL: BASE_URL,
  headers: {{ 'Content-Type': 'application/json' }},
}});

export class {entity_name}Client {{
  /** List all {entity_plural} */
  async list(): Promise<{entity_name}[]> {{
    const {{ data }} = await api.get('/');
    return data;
  }}

  /** Get {entity_name} by ID */
  async get(id: {id_type}): Promise<{entity_name}> {{
    const {{ data }} = await api.get(`/${{id}}`);
    return data;
  }}

  /** Create a new {entity_name} */
  async create(data: {entity_name}Input): Promise<{entity_name}> {{
    const {{ data: created }} = await api.post('/', data);
    return created;
  }}

  /** Update {entity_name} */
  async update(id: {id_type}, data: {entity_name}Input): Promise<{entity_name}> {{
    const {{ data: updated }} = await api.put(`/${{id}}`, data);
    return updated;
  }}

  /** Delete {entity_name} */
  async delete(id: {id_type}): Promise<void> {{
    await api.delete(`/${{id}}`);
  }}
}}

// Default client instance
export const {entity_camel}Client = new {entity_name}Client();
"#,
            entity_name = entity_name,
            entity_plural = entity_name.to_lowercase() + "s",
            entity_camel = entity_name[..1].to_lowercase() + &entity_name[1..],
            id_type = id_type,
        )
    }

    /// Generate React Query hooks
    fn generate_react_query_hooks(&self, entity: &GeneratorEntity) -> String {
        let entity_name = &entity.name.pascal;
        let entity_camel = entity.name.camel.clone();
        let entity_plural = &entity.name.plural_snake;
        let query_key = entity.name.screaming_snake.clone();
        let id_type = match entity.primary_key_type {
            PrimaryKeyType::BigInt => "number",
            PrimaryKeyType::Uuid => "string",
        };

        format!(
            r#"//! React Query hooks for {entity_name}

import {{ useQuery, useMutation, useQueryClient, UseQueryResult }} from '@tanstack/react-query';
import {{ {entity_name}, {entity_name}Input }} from '../schemas/{entity_kebab}.schema';
import {{ {entity_camel}Client }} from './{entity_kebab}.client';

const QUERY_KEY = '{query_key}';

/** Hook to list all {entity_plural} */
export function use{entity_plural_pascal}List(): UseQueryResult<{entity_name}[], Error> {{
  return useQuery({{
    queryKey: [QUERY_KEY],
    queryFn: () => {entity_camel}Client.list(),
  }});
}}

/** Hook to get a single {entity_name} */
export function use{entity_name}(id: {id_type} | undefined): UseQueryResult<{entity_name}, Error> {{
  return useQuery({{
    queryKey: [QUERY_KEY, id],
    queryFn: () => {entity_camel}Client.get(id!),
    enabled: id !== undefined,
  }});
}}

/** Hook to create a {entity_name} */
export function useCreate{entity_name}() {{
  const queryClient = useQueryClient();
  
  return useMutation({{
    mutationFn: (data: {entity_name}Input) => {entity_camel}Client.create(data),
    onSuccess: () => {{
      queryClient.invalidateQueries({{ queryKey: [QUERY_KEY] }});
    }},
  }});
}}

/** Hook to update a {entity_name} */
export function useUpdate{entity_name}() {{
  const queryClient = useQueryClient();
  
  return useMutation({{
    mutationFn: ({{ id, data }}: {{ id: {id_type}; data: {entity_name}Input }}) =>
      {entity_camel}Client.update(id, data),
    onSuccess: (_, variables) => {{
      queryClient.invalidateQueries({{ queryKey: [QUERY_KEY] }});
      queryClient.invalidateQueries({{ queryKey: [QUERY_KEY, variables.id] }});
    }},
  }});
}}

/** Hook to delete a {entity_name} */
export function useDelete{entity_name}() {{
  const queryClient = useQueryClient();
  
  return useMutation({{
    mutationFn: (id: {id_type}) => {entity_camel}Client.delete(id),
    onSuccess: () => {{
      queryClient.invalidateQueries({{ queryKey: [QUERY_KEY] }});
    }},
  }});
}}
"#,
            entity_name = entity_name,
            entity_camel = entity_camel,
            entity_kebab = entity.name.kebab,
            entity_plural = entity_plural,
            entity_plural_pascal = entity.name.plural_pascal,
            query_key = query_key,
            id_type = id_type,
        )
    }

    /// Generate index file
    fn generate_index_file(&self, model: &GeneratorModel) -> String {
        let mut exports = Vec::new();

        for entity in &model.entities {
            exports.push(format!("export * from './{}.client';", entity.name.kebab));
            if self.options.include_react_query {
                exports.push(format!("export * from './{}.hooks';", entity.name.kebab));
            }
        }

        exports.join("\n")
    }

    /// Generate shared types file
    fn generate_types_file(&self, model: &GeneratorModel) -> String {
        let mut lines = vec![
            "//! Shared API types".to_string(),
            "".to_string(),
            "export interface ApiError {".to_string(),
            "  message: string;".to_string(),
            "  code?: string;".to_string(),
            "  status: number;".to_string(),
            "}".to_string(),
            "".to_string(),
            "export interface PaginatedResponse<T> {".to_string(),
            "  data: T[];".to_string(),
            "  total: number;".to_string(),
            "  page: number;".to_string(),
            "  pageSize: number;".to_string(),
            "  totalPages: number;".to_string(),
            "}".to_string(),
        ];

        // Add entity re-exports
        lines.push("".to_string());
        for entity in &model.entities {
            lines.push(format!(
                "export {{ {0}, {0}Input }} from '../schemas/{1}.schema';",
                entity.name.pascal, entity.name.kebab
            ));
        }

        lines.join("\n")
    }
}

impl Default for FrontendClientGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator for FrontendClientGenerator {
    fn name(&self) -> &'static str {
        "frontend_client"
    }

    fn generate(&self, model: &GeneratorModel) -> Result<GeneratedOutput, GenerateError> {
        self.generate(model)
    }

    fn validate(&self, _model: &GeneratorModel) -> Result<(), crate::generator::ValidationError> {
        Ok(())
    }

    fn supports_incremental(&self) -> bool {
        false
    }

    fn file_extensions(&self) -> Vec<&'static str> {
        vec!["ts"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::EntityName;

    fn create_test_entity() -> GeneratorEntity {
        GeneratorEntity {
            name: EntityName {
                raw: "Product".to_string(),
                snake: "product".to_string(),
                camel: "product".to_string(),
                pascal: "Product".to_string(),
                kebab: "product".to_string(),
                screaming_snake: "PRODUCT".to_string(),
                plural_snake: "products".to_string(),
                plural_pascal: "Products".to_string(),
                plural_kebab: "products".to_string(),
            },
            description: None,
            fields: vec![],
            relations: vec![],
            annotations: vec![],
            primary_key_type: PrimaryKeyType::BigInt,
            ..Default::default()
        }
    }

    #[test]
    fn test_generate_fetch_client() {
        let gen = FrontendClientGenerator::new();
        let entity = create_test_entity();
        let code = gen.generate_entity_client(&entity);

        assert!(code.contains("class ProductClient"));
        assert!(code.contains("fetch(this.baseUrl)"));
        assert!(code.contains("async list"));
        assert!(code.contains("async get"));
        assert!(code.contains("async create"));
    }

    #[test]
    fn test_generate_react_query_hooks() {
        let gen = FrontendClientGenerator::new();
        let entity = create_test_entity();
        let code = gen.generate_react_query_hooks(&entity);

        assert!(code.contains("useQuery"));
        assert!(code.contains("useMutation"));
        assert!(code.contains("useProductsList"));
        assert!(code.contains("useProduct"));
        assert!(code.contains("useCreateProduct"));
    }
}
