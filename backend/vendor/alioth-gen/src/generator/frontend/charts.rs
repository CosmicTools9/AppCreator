//! Chart Component Generator
//!
//! Generates chart components using recharts for data visualization.

use crate::generator::ir::{GeneratorEntity, GeneratorModel};
use crate::generator::{
    GenerateError, GeneratedFile, GeneratedOutput, GenerationMetadata, Generator,
};

/// Chart type
#[derive(Debug, Clone)]
pub enum ChartType {
    Line,
    Bar,
    Pie,
    Area,
    Composed,
}

/// Chart generator options
#[derive(Debug, Clone)]
pub struct ChartGeneratorOptions {
    /// Default chart type
    pub chart_type: ChartType,
    /// Include tooltips
    pub tooltips: bool,
    /// Include legend
    pub legend: bool,
    /// Responsive charts
    pub responsive: bool,
}

impl Default for ChartGeneratorOptions {
    fn default() -> Self {
        Self {
            chart_type: ChartType::Bar,
            tooltips: true,
            legend: true,
            responsive: true,
        }
    }
}

/// Chart component generator
pub struct ChartComponentGenerator {
    #[allow(dead_code)]
    options: ChartGeneratorOptions, // Reserved for future configuration
}

impl ChartComponentGenerator {
    /// Create a new chart generator
    pub fn new() -> Self {
        Self {
            options: ChartGeneratorOptions::default(),
        }
    }

    /// Create with custom options
    pub fn with_options(options: ChartGeneratorOptions) -> Self {
        Self { options }
    }

    /// Generate chart components for the model
    pub fn generate(&self, model: &GeneratorModel) -> Result<GeneratedOutput, GenerateError> {
        let mut files = Vec::new();

        for entity in &model.entities {
            // Generate analytics dashboard
            let dashboard = self.generate_analytics_dashboard(entity);
            files.push(GeneratedFile {
                path: format!("components/analytics/{}-analytics.tsx", entity.name.kebab).into(),
                content: dashboard,
                checksum: String::new(),
            });

            // Generate stat cards
            let stat_cards = self.generate_stat_cards(entity);
            files.push(GeneratedFile {
                path: format!("components/analytics/{}-stats.tsx", entity.name.kebab).into(),
                content: stat_cards,
                checksum: String::new(),
            });
        }

        // Generate shared chart components
        let shared_components = self.generate_shared_components();
        files.push(GeneratedFile {
            path: "components/charts/index.tsx".into(),
            content: shared_components,
            checksum: String::new(),
        });

        let c_file_count = files.len();

        Ok(GeneratedOutput {
            files,
            metadata: GenerationMetadata {
                generator_name: "chart_components".to_string(),
                entity_count: model.entities.len(),
                c_file_count,
            },
        })
    }

    /// Generate analytics dashboard
    fn generate_analytics_dashboard(&self, entity: &GeneratorEntity) -> String {
        let entity_name = &entity.name.pascal;
        let entity_plural = &entity.name.plural_pascal;
        let entity_kebab = &entity.name.kebab;

        let lines = vec![
            "\"use client\";".to_string(),
            "".to_string(),
            "import { useMemo } from \"react\";".to_string(),
            "import { Card, CardContent, CardDescription, CardHeader, CardTitle } from \"@/components/ui/card\";".to_string(),
            "import { Tabs, TabsContent, TabsList, TabsTrigger } from \"@/components/ui/tabs\";".to_string(),
            format!("import {{ {}Stats }} from \"./{}-stats\";", entity_plural, entity_kebab),
            "import { AreaChart, Area, BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, Cell } from \"recharts\";".to_string(),
            format!("import {{ use{}List }} from \"@/api/{}.hooks\";", entity_plural, entity_kebab),
            "".to_string(),
            "const COLORS = [\"#0088FE\", \"#00C49F\", \"#FFBB28\", \"#FF8042\", \"#8884D8\"];".to_string(),
            "".to_string(),
            format!("export function {}Analytics() {{", entity_name),
            format!("  const {{ data: items, isLoading }} = use{}List();", entity_plural),
            "".to_string(),
            "  const chartData = useMemo(() => {".to_string(),
            "    if (!items) return [];".to_string(),
            "".to_string(),
            "    const grouped = items.reduce((acc: Record<string, number>, item: any) => {".to_string(),
            "      const date = new Date(item.createdAt).toLocaleDateString();".to_string(),
            "      acc[date] = (acc[date] || 0) + 1;".to_string(),
            "      return acc;".to_string(),
            "    }, {});".to_string(),
            "".to_string(),
            "    return Object.entries(grouped).map(([date, count]) => ({".to_string(),
            "      date,".to_string(),
            "      count,".to_string(),
            "    }));".to_string(),
            "  }, [items]);".to_string(),
            "".to_string(),
            "  if (isLoading) {".to_string(),
            "    return <div>Loading analytics...</div>;".to_string(),
            "  }".to_string(),
            "".to_string(),
            "  return (".to_string(),
            "    <div className=\"space-y-6\">".to_string(),
            format!("      <{}Stats />", entity_plural),
            "".to_string(),
            "      <Tabs defaultValue=\"trend\" className=\"w-full\">".to_string(),
            "        <TabsList>".to_string(),
            "          <TabsTrigger value=\"trend\">Trend</TabsTrigger>".to_string(),
            "          <TabsTrigger value=\"distribution\">Distribution</TabsTrigger>".to_string(),
            "        </TabsList>".to_string(),
            "".to_string(),
            "        <TabsContent value=\"trend\">".to_string(),
            "          <Card>".to_string(),
            "            <CardHeader>".to_string(),
            "              <CardTitle>Creation Trend</CardTitle>".to_string(),
            format!("              <CardDescription>Daily {} creation over time</CardDescription>", entity_name),
            "            </CardHeader>".to_string(),
            "            <CardContent>".to_string(),
            "              <ResponsiveContainer width=\"100%\" height={300}>".to_string(),
            "                <AreaChart data={chartData}>".to_string(),
            "                  <CartesianGrid strokeDasharray=\"3 3\" />".to_string(),
            "                  <XAxis dataKey=\"date\" />".to_string(),
            "                  <YAxis />".to_string(),
            "                  <Tooltip />".to_string(),
            "                  <Legend />".to_string(),
            "                  <Area".to_string(),
            "                    type=\"monotone\"".to_string(),
            "                    dataKey=\"count\"".to_string(),
            "                    name=\"Created\"".to_string(),
            "                    stroke=\"#8884d8\"".to_string(),
            "                    fill=\"#8884d8\"".to_string(),
            "                    fillOpacity={0.3}".to_string(),
            "                  />".to_string(),
            "                </AreaChart>".to_string(),
            "              </ResponsiveContainer>".to_string(),
            "            </CardContent>".to_string(),
            "          </Card>".to_string(),
            "        </TabsContent>".to_string(),
            "".to_string(),
            "        <TabsContent value=\"distribution\">".to_string(),
            "          <Card>".to_string(),
            "            <CardHeader>".to_string(),
            "              <CardTitle>Distribution</CardTitle>".to_string(),
            "              <CardDescription>Distribution by date</CardDescription>".to_string(),
            "            </CardHeader>".to_string(),
            "            <CardContent>".to_string(),
            "              <ResponsiveContainer width=\"100%\" height={300}>".to_string(),
            "                <BarChart data={chartData}>".to_string(),
            "                  <CartesianGrid strokeDasharray=\"3 3\" />".to_string(),
            "                  <XAxis dataKey=\"date\" />".to_string(),
            "                  <YAxis />".to_string(),
            "                  <Tooltip />".to_string(),
            "                  <Legend />".to_string(),
            "                  <Bar dataKey=\"count\" name=\"Count\">".to_string(),
            "                    {chartData.map((_: any, index: number) => (".to_string(),
            "                      <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />".to_string(),
            "                    ))}".to_string(),
            "                  </Bar>".to_string(),
            "                </BarChart>".to_string(),
            "              </ResponsiveContainer>".to_string(),
            "            </CardContent>".to_string(),
            "          </Card>".to_string(),
            "        </TabsContent>".to_string(),
            "      </Tabs>".to_string(),
            "    </div>".to_string(),
            "  );".to_string(),
            "}".to_string(),
        ];

        lines.join("\n")
    }

    /// Generate stat cards
    fn generate_stat_cards(&self, entity: &GeneratorEntity) -> String {
        let _entity_name = &entity.name.pascal; // Reserved for future use
        let entity_plural = &entity.name.plural_pascal;
        let entity_kebab = &entity.name.kebab;

        let lines = vec![
            "\"use client\";".to_string(),
            "".to_string(),
            "import { Card, CardContent, CardHeader, CardTitle } from \"@/components/ui/card\";".to_string(),
            "import { useMemo } from \"react\";".to_string(),
            format!("import {{ use{}List }} from \"@/api/{}.hooks\";", entity_plural, entity_kebab),
            "".to_string(),
            format!("export function {}Stats() {{", entity_plural),
            format!("  const {{ data: items }} = use{}List();", entity_plural),
            "".to_string(),
            "  const stats = useMemo(() => {".to_string(),
            "    if (!items) return { total: 0, recent: 0 };".to_string(),
            "".to_string(),
            "    const total = items.length;".to_string(),
            "    const recent = items.filter(".to_string(),
            "      (item: any) => new Date(item.createdAt) > new Date(Date.now() - 7 * 24 * 60 * 60 * 1000)".to_string(),
            "    ).length;".to_string(),
            "".to_string(),
            "    return { total, recent };".to_string(),
            "  }, [items]);".to_string(),
            "".to_string(),
            "  return (".to_string(),
            "    <div className=\"grid gap-4 md:grid-cols-2 lg:grid-cols-4\">".to_string(),
            "      <Card>".to_string(),
            "        <CardHeader className=\"flex flex-row items-center justify-between space-y-0 pb-2\">".to_string(),
            format!("          <CardTitle className=\"text-sm font-medium\">Total {}</CardTitle>", entity_plural),
            "        </CardHeader>".to_string(),
            "        <CardContent>".to_string(),
            "          <div className=\"text-2xl font-bold\">{stats.total}</div>".to_string(),
            "          <p className=\"text-xs text-muted-foreground\">All time</p>".to_string(),
            "        </CardContent>".to_string(),
            "      </Card>".to_string(),
            "".to_string(),
            "      <Card>".to_string(),
            "        <CardHeader className=\"flex flex-row items-center justify-between space-y-0 pb-2\">".to_string(),
            "          <CardTitle className=\"text-sm font-medium\">Recent</CardTitle>".to_string(),
            "        </CardHeader>".to_string(),
            "        <CardContent>".to_string(),
            "          <div className=\"text-2xl font-bold\">{stats.recent}</div>".to_string(),
            "          <p className=\"text-xs text-muted-foreground\">Last 7 days</p>".to_string(),
            "        </CardContent>".to_string(),
            "      </Card>".to_string(),
            "".to_string(),
            "      <Card>".to_string(),
            "        <CardHeader className=\"flex flex-row items-center justify-between space-y-0 pb-2\">".to_string(),
            "          <CardTitle className=\"text-sm font-medium\">Growth</CardTitle>".to_string(),
            "        </CardHeader>".to_string(),
            "        <CardContent>".to_string(),
            "          <div className=\"text-2xl font-bold\">".to_string(),
            "            {stats.total > 0 ? ((stats.recent / stats.total) * 100).toFixed(1) : 0}%".to_string(),
            "          </div>".to_string(),
            "          <p className=\"text-xs text-muted-foreground\">Weekly growth</p>".to_string(),
            "        </CardContent>".to_string(),
            "      </Card>".to_string(),
            "".to_string(),
            "      <Card>".to_string(),
            "        <CardHeader className=\"flex flex-row items-center justify-between space-y-0 pb-2\">".to_string(),
            "          <CardTitle className=\"text-sm font-medium\">Active</CardTitle>".to_string(),
            "        </CardHeader>".to_string(),
            "        <CardContent>".to_string(),
            "          <div className=\"text-2xl font-bold\">{stats.total}</div>".to_string(),
            "          <p className=\"text-xs text-muted-foreground\">Currently active</p>".to_string(),
            "        </CardContent>".to_string(),
            "      </Card>".to_string(),
            "    </div>".to_string(),
            "  );".to_string(),
            "}".to_string(),
        ];

        lines.join("\n")
    }

    /// Generate shared chart components
    fn generate_shared_components(&self) -> String {
        vec![
            "\"use client\";".to_string(),
            "".to_string(),
            "// Re-export recharts components".to_string(),
            "export {".to_string(),
            "  LineChart,".to_string(),
            "  Line,".to_string(),
            "  BarChart,".to_string(),
            "  Bar,".to_string(),
            "  PieChart,".to_string(),
            "  Pie,".to_string(),
            "  AreaChart,".to_string(),
            "  Area,".to_string(),
            "  XAxis,".to_string(),
            "  YAxis,".to_string(),
            "  CartesianGrid,".to_string(),
            "  Tooltip,".to_string(),
            "  Legend,".to_string(),
            "  ResponsiveContainer,".to_string(),
            "  Cell,".to_string(),
            "} from \"recharts\";".to_string(),
        ]
        .join("\n")
    }
}

impl Default for ChartComponentGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator for ChartComponentGenerator {
    fn name(&self) -> &'static str {
        "chart_components"
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
    use crate::generator::ir::EntityName;
    use crate::generator::ir::PrimaryKeyType;

    fn create_test_entity() -> GeneratorEntity {
        GeneratorEntity {
            name: EntityName {
                raw: "Sale".to_string(),
                snake: "sale".to_string(),
                camel: "sale".to_string(),
                pascal: "Sale".to_string(),
                kebab: "sale".to_string(),
                screaming_snake: "SALE".to_string(),
                plural_snake: "sales".to_string(),
                plural_pascal: "Sales".to_string(),
                plural_kebab: "sales".to_string(),
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
    fn test_generate_analytics_dashboard() {
        let gen = ChartComponentGenerator::new();
        let entity = create_test_entity();
        let code = gen.generate_analytics_dashboard(&entity);

        assert!(code.contains("function SaleAnalytics"));
        assert!(code.contains("recharts"));
        assert!(code.contains("AreaChart"));
        assert!(code.contains("useSalesList"));
    }

    #[test]
    fn test_generate_stat_cards() {
        let gen = ChartComponentGenerator::new();
        let entity = create_test_entity();
        let code = gen.generate_stat_cards(&entity);

        assert!(code.contains("function SalesStats"));
        assert!(code.contains("Total Sales"));
        assert!(code.contains("Recent"));
    }
}
