//! Data Table Component Generator

use crate::generator::ir::{GeneratorEntity, GeneratorModel};
use crate::generator::{
    GenerateError, GeneratedFile, GeneratedOutput, GenerationMetadata, Generator,
};

/// Data table generator
pub struct DataTableGenerator;

impl DataTableGenerator {
    /// Create a new table generator
    pub fn new() -> Self {
        Self
    }

    /// Generate table components for the model
    pub fn generate(&self, model: &GeneratorModel) -> Result<GeneratedOutput, GenerateError> {
        let mut files = Vec::new();

        for entity in &model.entities {
            let table_component = self.generate_table_component(entity);
            files.push(GeneratedFile {
                path: format!("components/tables/{}-table.tsx", entity.name.kebab).into(),
                content: table_component,
                checksum: String::new(),
            });

            let columns_def = self.generate_columns_definition(entity);
            files.push(GeneratedFile {
                path: format!("components/tables/{}-columns.tsx", entity.name.kebab).into(),
                content: columns_def,
                checksum: String::new(),
            });
        }

        let c_file_count = files.len();

        Ok(GeneratedOutput {
            files,
            metadata: GenerationMetadata {
                generator_name: "data_table".to_string(),
                entity_count: model.entities.len(),
                c_file_count,
            },
        })
    }

    /// Generate table component
    fn generate_table_component(&self, entity: &GeneratorEntity) -> String {
        let entity_name = &entity.name.pascal;
        let entity_plural = &entity.name.plural_pascal;
        let entity_kebab = &entity.name.kebab;
        let _entity_plural_kebab = &entity.name.plural_kebab; // Reserved for future use

        let lines = vec![
            "\"use client\";".to_string(),
            "".to_string(),
            "import { useState } from \"react\";".to_string(),
            "import { useReactTable, getCoreRowModel, getPaginationRowModel, flexRender } from \"@tanstack/react-table\";".to_string(),
            "import { Button } from \"@/components/ui/button\";".to_string(),
            "import { Input } from \"@/components/ui/input\";".to_string(),
            "import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from \"@/components/ui/table\";".to_string(),
            format!("import {{ {0} }} from \"@/schemas/{1}.schema\";", entity_name, entity_kebab),
            format!("import {{ columns }} from \"./{0}-columns\";", entity_kebab),
            format!("import {{ use{0}List }} from \"@/api/{1}.hooks\";", entity_plural, entity_kebab),
            "".to_string(),
            format!("export function {0}Table() {{", entity_plural),
            format!("  const {{ data, isLoading }} = use{0}List();", entity_plural),
            "".to_string(),
            "  const table = useReactTable({".to_string(),
            "    data: data || [],".to_string(),
            "    columns,".to_string(),
            "    getCoreRowModel: getCoreRowModel(),".to_string(),
            "    getPaginationRowModel: getPaginationRowModel(),".to_string(),
            "  });".to_string(),
            "".to_string(),
            "  if (isLoading) {".to_string(),
            "    return <div>Loading...</div>;".to_string(),
            "  }".to_string(),
            "".to_string(),
            "  return (".to_string(),
            "    <div className=\"space-y-4\">".to_string(),
            "      <div className=\"rounded-md border\">".to_string(),
            "        <Table>".to_string(),
            "          <TableHeader>".to_string(),
            "            {table.getHeaderGroups().map((headerGroup) => (".to_string(),
            "              <TableRow key={headerGroup.id}>".to_string(),
            "                {headerGroup.headers.map((header) => (".to_string(),
            "                  <TableHead key={header.id}>".to_string(),
            "                    {header.isPlaceholder".to_string(),
            "                      ? null".to_string(),
            "                      : flexRender(header.column.columnDef.header, header.getContext())}".to_string(),
            "                  </TableHead>".to_string(),
            "                ))}".to_string(),
            "              </TableRow>".to_string(),
            "            ))}".to_string(),
            "          </TableHeader>".to_string(),
            "          <TableBody>".to_string(),
            "            {table.getRowModel().rows?.length ? (".to_string(),
            "              table.getRowModel().rows.map((row) => (".to_string(),
            "                <TableRow key={row.id}>".to_string(),
            "                  {row.getVisibleCells().map((cell) => (".to_string(),
            "                    <TableCell key={cell.id}>".to_string(),
            "                      {flexRender(cell.column.columnDef.cell, cell.getContext())}".to_string(),
            "                    </TableCell>".to_string(),
            "                  ))}".to_string(),
            "                </TableRow>".to_string(),
            "              ))".to_string(),
            "            ) : (".to_string(),
            "              <TableRow>".to_string(),
            "                <TableCell colSpan={columns.length} className=\"h-24 text-center\">".to_string(),
            "                  No results.".to_string(),
            "                </TableCell>".to_string(),
            "              </TableRow>".to_string(),
            "            )}".to_string(),
            "          </TableBody>".to_string(),
            "        </Table>".to_string(),
            "      </div>".to_string(),
            "    </div>".to_string(),
            "  );".to_string(),
            "}".to_string(),
        ];

        lines.join("\n")
    }

    /// Generate columns definition
    fn generate_columns_definition(&self, entity: &GeneratorEntity) -> String {
        let entity_name = &entity.name.pascal;
        let entity_kebab = &entity.name.kebab;

        let mut lines = vec![
            "\"use client\";".to_string(),
            "".to_string(),
            "import { ColumnDef } from \"@tanstack/react-table\";".to_string(),
            format!(
                "import {{ {0} }} from \"@/schemas/{1}.schema\";",
                entity_name, entity_kebab
            ),
            "".to_string(),
            format!("export const columns: ColumnDef<{0}>[] = [", entity_name),
            "  {".to_string(),
            "    accessorKey: \"id\",".to_string(),
            "    header: \"ID\",".to_string(),
            "  },".to_string(),
        ];

        for field in &entity.fields {
            if field.name.snake == "id" {
                continue;
            }
            lines.push("  {".to_string());
            lines.push(format!("    accessorKey: \"{}\",", field.name.camel));
            lines.push(format!("    header: \"{}\",", field.name.pascal));
            lines.push("  }".to_string());
        }

        lines.push("];".to_string());

        lines.join("\n")
    }
}

impl Default for DataTableGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator for DataTableGenerator {
    fn name(&self) -> &'static str {
        "data_table"
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
        vec!["tsx"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::{EntityName, PrimaryKeyType};

    fn create_test_entity() -> GeneratorEntity {
        GeneratorEntity {
            name: EntityName {
                raw: "Order".to_string(),
                snake: "order".to_string(),
                camel: "order".to_string(),
                pascal: "Order".to_string(),
                kebab: "order".to_string(),
                screaming_snake: "ORDER".to_string(),
                plural_snake: "orders".to_string(),
                plural_pascal: "Orders".to_string(),
                plural_kebab: "orders".to_string(),
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
    fn test_generate_table_component() {
        let gen = DataTableGenerator::new();
        let entity = create_test_entity();
        let code = gen.generate_table_component(&entity);

        assert!(code.contains("function OrdersTable"));
        assert!(code.contains("useReactTable"));
    }

    #[test]
    fn test_generate_columns_definition() {
        let gen = DataTableGenerator::new();
        let entity = create_test_entity();
        let code = gen.generate_columns_definition(&entity);

        assert!(code.contains("export const columns"));
        assert!(code.contains("ColumnDef<Order>"));
    }
}
