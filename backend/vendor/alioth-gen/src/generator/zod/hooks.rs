//! React Hook Form Integration
//!
//! Generates React Hook Form hooks with Zod resolver.

use crate::generator::ir::{GeneratorEntity, GeneratorModel};
use crate::generator::{
    GenerateError, GeneratedFile, GeneratedOutput, GenerationMetadata, Generator,
};

/// React Hook Form generator
pub struct ReactHookFormGenerator;

impl ReactHookFormGenerator {
    /// Generate useForm hook for an entity
    pub fn generate_form_hook(entity: &GeneratorEntity) -> String {
        let entity_name = &entity.name.pascal;
        let _entity_camel = &entity.name.camel;

        format!(
            r#"import {{ useForm }} from 'react-hook-form';
import {{ zodResolver }} from '@hookform/resolvers/zod';
import {{ {}Schema, {}Input }} from './{}.schema';

export interface Use{}FormOptions {{
  defaultValues?: Partial<{}Input>;
  onSubmit?: (data: {}Input) => void | Promise<void>;
}}

export function use{}Form(options: Use{}FormOptions = {{}}) {{
  const {{ defaultValues, onSubmit }} = options;

  const form = useForm<{}Input>({{
    resolver: zodResolver({}Schema),
    defaultValues: defaultValues ?? {{}},
  }});

  const handleSubmit = form.handleSubmit(async (data) => {{
    if (onSubmit) {{
      await onSubmit(data);
    }}
  }});

  return {{
    ...form,
    handleSubmit,
  }};
}}

export type {}FormReturn = ReturnType<typeof use{}Form>;"#,
            entity_name,
            entity_name,
            entity.name.kebab,
            entity_name,
            entity_name,
            entity_name,
            entity_name,
            entity_name,
            entity_name,
            entity_name,
            entity_name,
            entity_name,
        )
    }

    /// Generate form component
    pub fn generate_form_component(entity: &GeneratorEntity) -> String {
        let entity_name = &entity.name.pascal;
        let fields: Vec<String> = entity
            .fields
            .iter()
            .filter(|f| f.name.snake != "id")
            .map(|f| {
                format!(
                    r#"        <div className="form-field">
          <label htmlFor="{0}">{1}</label>
          <input
            id="{0}"
            {{...register('{0}')}}
            className={{errors.{0} ? 'error' : ''}}
          />
          {{errors.{0} && <span className="error-message">{{errors.{0}.message}}</span>}}
        </div>"#,
                    f.name.camel, f.name.pascal
                )
            })
            .collect();

        format!(
            r#"import React from 'react';
import {{ use{}Form }} from './use{}Form';

export interface {}FormProps {{
  onSubmit?: (data: {}Input) => void | Promise<void>;
  defaultValues?: Partial<{}Input>;
}}

export function {}Form({{ onSubmit, defaultValues }}: {}FormProps) {{
  const {{ register, handleSubmit, formState: {{ errors }} }} = use{}Form({{
    onSubmit,
    defaultValues,
  }});

  return (
    <form onSubmit={{handleSubmit}} className="{}-form">
{}
      <button type="submit">Submit</button>
    </form>
  );
}}"#,
            entity_name,
            entity_name,
            entity_name,
            entity_name,
            entity_name,
            entity_name,
            entity_name,
            entity_name,
            entity.name.kebab,
            fields.join("\n")
        )
    }

    /// Generate API hooks using TanStack Query
    pub fn generate_api_hooks(entity: &GeneratorEntity) -> String {
        let entity_name = &entity.name.pascal;
        let entity_plural = &entity.name.plural_pascal;
        let base_path = format!("/api/{}", entity.name.plural_kebab);

        format!(
            r#"import {{ useQuery, useMutation, useQueryClient }} from '@tanstack/react-query';
import {{ {}Input }} from './{}.schema';

const {}_KEY = '{}';

// List query
export function use{}List() {{
  return useQuery({{
    queryKey: [{}_KEY],
    queryFn: async () => {{
      const response = await fetch('{}');
      if (!response.ok) throw new Error('Failed to fetch {}');
      return response.json();
    }},
  }});
}}

// Get by ID
export function use{}(id: string | undefined) {{
  return useQuery({{
    queryKey: [{}_KEY, id],
    queryFn: async () => {{
      if (!id) return null;
      const response = await fetch(`{}/${{id}}`);
      if (!response.ok) throw new Error('Failed to fetch {}');
      return response.json();
    }},
    enabled: !!id,
  }});
}}

// Create mutation
export function useCreate{}() {{
  const queryClient = useQueryClient();

  return useMutation({{
    mutationFn: async (data: {}Input) => {{
      const response = await fetch('{}', {{
        method: 'POST',
        headers: {{ 'Content-Type': 'application/json' }},
        body: JSON.stringify(data),
      }});
      if (!response.ok) throw new Error('Failed to create {}');
      return response.json();
    }},
    onSuccess: () => {{
      queryClient.invalidateQueries({{ queryKey: [{}_KEY] }});
    }},
  }});
}}

// Update mutation
export function useUpdate{}() {{
  const queryClient = useQueryClient();

  return useMutation({{
    mutationFn: async ({{ id, data }}: {{ id: string; data: {}Input }}) => {{
      const response = await fetch(`{}/${{id}}`, {{
        method: 'PUT',
        headers: {{ 'Content-Type': 'application/json' }},
        body: JSON.stringify(data),
      }});
      if (!response.ok) throw new Error('Failed to update {}');
      return response.json();
    }},
    onSuccess: (_, variables) => {{
      queryClient.invalidateQueries({{ queryKey: [{}_KEY] }});
      queryClient.invalidateQueries({{ queryKey: [{}_KEY, variables.id] }});
    }},
  }});
}}

// Delete mutation
export function useDelete{}() {{
  const queryClient = useQueryClient();

  return useMutation({{
    mutationFn: async (id: string) => {{
      const response = await fetch(`{}/${{id}}`, {{
        method: 'DELETE',
      }});
      if (!response.ok) throw new Error('Failed to delete {}');
      return response.json();
    }},
    onSuccess: () => {{
      queryClient.invalidateQueries({{ queryKey: [{}_KEY] }});
    }},
  }});
}}"#,
            entity_name,
            entity.name.kebab,
            entity.name.screaming_snake,
            base_path,
            entity_plural,
            entity.name.screaming_snake,
            base_path,
            entity.name.plural_snake,
            entity_name,
            entity.name.screaming_snake,
            base_path,
            entity.name.snake,
            entity_name,
            entity_name,
            base_path,
            entity.name.snake,
            entity.name.screaming_snake,
            entity_name,
            entity_name,
            base_path,
            entity.name.snake,
            entity.name.screaming_snake,
            entity.name.screaming_snake,
            entity_name,
            base_path,
            entity.name.snake,
            entity.name.screaming_snake,
        )
    }
}

impl Default for ReactHookFormGenerator {
    fn default() -> Self {
        Self
    }
}

/// Combined hook generator
pub struct HookGenerator;

impl Generator for HookGenerator {
    fn name(&self) -> &'static str {
        "react_hooks"
    }

    fn generate(&self, model: &GeneratorModel) -> Result<GeneratedOutput, GenerateError> {
        let mut files = Vec::new();

        for entity in &model.entities {
            // Form hook
            let form_hook = ReactHookFormGenerator::generate_form_hook(entity);
            files.push(GeneratedFile {
                path: format!("hooks/use{}Form.ts", entity.name.pascal).into(),
                content: form_hook,
                checksum: String::new(),
            });

            // API hooks
            let api_hooks = ReactHookFormGenerator::generate_api_hooks(entity);
            files.push(GeneratedFile {
                path: format!("hooks/use{}Api.ts", entity.name.pascal).into(),
                content: api_hooks,
                checksum: String::new(),
            });
        }

        // Index file
        let exports: Vec<_> = model
            .entities
            .iter()
            .flat_map(|e| {
                vec![
                    format!("export * from './use{}Form';", e.name.pascal),
                    format!("export * from './use{}Api';", e.name.pascal),
                ]
            })
            .collect();

        files.push(GeneratedFile {
            path: "hooks/index.ts".into(),
            content: exports.join("\n"),
            checksum: String::new(),
        });

        let c_file_count = files.len();

        Ok(GeneratedOutput {
            files,
            metadata: GenerationMetadata {
                generator_name: self.name().to_string(),
                entity_count: model.entities.len(),
                c_file_count,
            },
        })
    }

    fn validate(&self, _model: &GeneratorModel) -> Result<(), crate::generator::ValidationError> {
        Ok(())
    }

    fn supports_incremental(&self) -> bool {
        false
    }

    fn file_extensions(&self) -> Vec<&'static str> {
        vec!["ts", "tsx"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::{EntityName, FieldName, GeneratorField};

    fn create_test_entity() -> GeneratorEntity {
        GeneratorEntity {
            name: EntityName {
                raw: "User".to_string(),
                snake: "users".to_string(),
                camel: "users".to_string(),
                pascal: "Users".to_string(),
                kebab: "users".to_string(),
                screaming_snake: "USERS".to_string(),
                plural_snake: "users".to_string(),
                plural_pascal: "Users".to_string(),
                plural_kebab: "users".to_string(),
            },
            description: None,
            fields: vec![GeneratorField {
                name: FieldName {
                    raw: "email".to_string(),
                    snake: "email".to_string(),
                    camel: "email".to_string(),
                    pascal: "Email".to_string(),
                },
                field_type: crate::generator::ir::GeneratorFieldType::Text,
                description: None,
                nullable: false,
                unique: false,
                indexed: false,
                default_value: None,
                validations: vec![],
                annotations: vec![],
                ..Default::default()
            }],
            relations: vec![],
            annotations: vec![],
            primary_key_type: crate::generator::ir::PrimaryKeyType::BigInt,
            ..Default::default()
        }
    }

    #[test]
    fn test_generate_form_hook() {
        let entity = create_test_entity();
        let hook = ReactHookFormGenerator::generate_form_hook(&entity);

        assert!(hook.contains("useUsersForm"));
        assert!(hook.contains("zodResolver"));
        assert!(hook.contains("UsersSchema"));
        assert!(hook.contains("UsersInput"));
    }

    #[test]
    fn test_generate_api_hooks() {
        let entity = create_test_entity();
        let hooks = ReactHookFormGenerator::generate_api_hooks(&entity);

        assert!(hooks.contains("useUsersList"));
        assert!(hooks.contains("useUsers"));
        assert!(hooks.contains("useCreateUsers"));
        assert!(hooks.contains("useUpdateUsers"));
        assert!(hooks.contains("useDeleteUsers"));
        assert!(hooks.contains("@tanstack/react-query"));
    }
}
