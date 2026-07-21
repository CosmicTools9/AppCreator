//! Ontology Visualizer
//!
//! 本体可视化推演模型 - 支持图形化展示本体关系和推理过程
//! 生成可视化所需的数据结构，便于前端渲染和交互

use super::ontology::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 可视化图谱
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualGraph {
    /// 图谱标识
    pub id: String,
    /// 图谱名称
    pub name: String,
    /// 节点集合
    pub nodes: Vec<VisualNode>,
    /// 边集合
    pub edges: Vec<VisualEdge>,
    /// 布局配置
    pub layout: LayoutConfig,
    /// 视图状态
    pub view_state: ViewState,
}

/// 可视化节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualNode {
    /// 节点标识
    pub id: String,
    /// 节点标签
    pub label: String,
    /// 节点类型
    pub node_type: NodeType,
    /// 节点位置
    pub position: NodePosition,
    /// 节点样式
    pub style: NodeStyle,
    /// 节点数据（关联的本体信息）
    pub data: NodeData,
    /// 是否可交互
    pub interactive: bool,
}

/// 节点类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    /// 领域本体节点
    Domain,
    /// 属性节点
    Property,
    /// 关系节点
    Relation,
    /// 交易阶段节点
    TransactionPhase,
    /// 约束节点
    Constraint,
    /// 计算节点
    Computation,
    /// 预制件节点
    Prefab,
    /// 分组节点
    Group,
}

/// 节点位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePosition {
    /// X 坐标
    pub x: f64,
    /// Y 坐标
    pub y: f64,
    /// Z 坐标（用于3D布局）
    pub z: Option<f64>,
}

/// 节点样式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStyle {
    /// 背景颜色
    pub background_color: String,
    /// 边框颜色
    pub border_color: String,
    /// 边框宽度
    pub border_width: f64,
    /// 文字颜色
    pub text_color: String,
    /// 节点大小
    pub size: f64,
    /// 形状
    pub shape: NodeShape,
    /// 图标
    pub icon: Option<String>,
    /// 透明度
    pub opacity: f64,
}

/// 节点形状
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NodeShape {
    /// 矩形
    Rectangle,
    /// 圆角矩形
    RoundedRectangle,
    /// 圆形
    Circle,
    /// 椭圆形
    Ellipse,
    /// 菱形
    Diamond,
    /// 六边形
    Hexagon,
    /// 自定义形状
    Custom(String),
}

/// 节点数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeData {
    /// 本体标识
    pub ontology_id: String,
    /// 本体类型
    pub ontology_type: String,
    /// 附加属性
    pub properties: HashMap<String, String>,
    /// 详细描述
    pub description: Option<String>,
}

/// 可视化边
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualEdge {
    /// 边标识
    pub id: String,
    /// 源节点
    pub source: String,
    /// 目标节点
    pub target: String,
    /// 边标签
    pub label: Option<String>,
    /// 边类型
    pub edge_type: EdgeType,
    /// 边样式
    pub style: EdgeStyle,
    /// 是否为推断边
    pub is_inferred: bool,
    /// 推理来源
    pub inference_source: Option<String>,
}

/// 边类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    /// 继承关系
    Inheritance,
    /// 关联关系
    Association,
    /// 聚合关系
    Aggregation,
    /// 组合关系
    Composition,
    /// 依赖关系
    Dependency,
    /// 等价关系
    Equivalence,
    /// 互斥关系
    Disjoint,
    /// 转换关系
    Transition,
    /// 约束关系
    Constraint,
    /// 计算关系
    Computation,
    /// 预制件接口关系
    PrefabInterface,
    /// 自定义关系
    Custom(String),
}

/// 边样式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeStyle {
    /// 线条颜色
    pub color: String,
    /// 线条宽度
    pub width: f64,
    /// 线条样式
    pub line_style: LineStyle,
    /// 箭头类型
    pub arrow_type: ArrowType,
    /// 是否虚线
    pub dashed: bool,
    /// 透明度
    pub opacity: f64,
}

/// 线条样式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LineStyle {
    /// 实线
    Solid,
    /// 虚线
    Dashed,
    /// 点线
    Dotted,
    /// 双实线
    Double,
}

/// 箭头类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArrowType {
    /// 无箭头
    None,
    /// 单向箭头
    Arrow,
    /// 双向箭头
    Bidirectional,
    /// 菱形箭头（聚合）
    Diamond,
    /// 实心菱形（组合）
    FilledDiamond,
    /// 三角形（继承）
    Triangle,
    /// 空心三角形（实现）
    HollowTriangle,
}

/// 布局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    /// 布局算法
    pub algorithm: LayoutAlgorithm,
    /// 节点间距
    pub node_spacing: f64,
    /// 层级间距
    pub level_spacing: f64,
    /// 是否启用物理模拟
    pub physics_enabled: bool,
    /// 物理参数
    pub physics_params: PhysicsParams,
}

/// 布局算法
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LayoutAlgorithm {
    /// 层次布局（适合树形结构）
    Hierarchical,
    /// 力导向布局（适合复杂网络）
    ForceDirected,
    /// 圆形布局
    Circular,
    /// 网格布局
    Grid,
    /// 4D空间布局
    Space4D,
    /// 自定义布局
    Custom(String),
}

/// 物理参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsParams {
    /// 引力常数
    pub gravity: f64,
    /// 斥力常数
    pub repulsion: f64,
    /// 弹簧劲度系数
    pub spring_stiffness: f64,
    /// 阻尼系数
    pub damping: f64,
}

/// 视图状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewState {
    /// 当前缩放级别
    pub zoom: f64,
    /// 视图中心位置
    pub center: NodePosition,
    /// 选中的节点
    pub selected_nodes: Vec<String>,
    /// 高亮的节点
    pub highlighted_nodes: Vec<String>,
    /// 折叠的节点
    pub collapsed_nodes: Vec<String>,
    /// 过滤器
    pub filters: Vec<ViewFilter>,
}

/// 视图过滤器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewFilter {
    /// 过滤器类型
    pub filter_type: FilterType,
    /// 过滤器值
    pub value: String,
    /// 是否启用
    pub enabled: bool,
}

/// 过滤器类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FilterType {
    /// 按节点类型过滤
    NodeType,
    /// 按关系类型过滤
    EdgeType,
    /// 按命名空间过滤
    Namespace,
    /// 按预制件类型过滤
    PrefabType,
    /// 自定义过滤
    Custom(String),
}

/// 可视化推演引擎
pub struct VisualizerEngine;

impl VisualizerEngine {
    /// 从本体模型生成可视化图谱
    pub fn generate_graph(model: &OntologyModel) -> VisualGraph {
        let mut graph = VisualGraph {
            id: format!("graph_{}", model.id),
            name: format!("{} Visualization", model.name),
            nodes: vec![],
            edges: vec![],
            layout: Self::default_layout(),
            view_state: Self::default_view_state(),
        };

        // 1. 生成本体节点
        Self::generate_domain_nodes(model, &mut graph);

        // 2. 生成属性节点
        Self::generate_property_nodes(model, &mut graph);

        // 3. 生成关系边
        Self::generate_relation_edges(model, &mut graph);

        // 4. 生成继承边
        Self::generate_inheritance_edges(model, &mut graph);

        // 5. 生成交易生命周期节点（如果有）
        if let Some(lifecycle) = &model.transaction_lifecycle {
            Self::generate_lifecycle_nodes(lifecycle, &mut graph);
        }

        // 6. 生成约束节点
        Self::generate_constraint_nodes(model, &mut graph);

        // 7. 生成计算节点
        Self::generate_computation_nodes(model, &mut graph);

        // 8. 应用布局
        Self::apply_layout(&mut graph);

        graph
    }

    /// 生成推演视图（显示推理过程）
    pub fn generate_inference_view(
        model: &OntologyModel,
        inference_result: &OntologyInferenceResult,
    ) -> VisualGraph {
        let mut graph = Self::generate_graph(model);

        // 高亮推断出的关系
        for inferred in &inference_result.inferred_relations {
            if let Some(edge) = graph
                .edges
                .iter_mut()
                .find(|e| e.source == inferred.source && e.target == inferred.target)
            {
                edge.is_inferred = true;
                edge.inference_source = Some(inferred.inference_source.clone());
                edge.style.color = "#ff6d3f".to_string(); // 高亮颜色
                edge.style.dashed = true;
            }
        }

        // 高亮推断出的属性
        for inferred in &inference_result.inferred_properties {
            if let Some(node) = graph.nodes.iter_mut().find(|n| {
                n.data.ontology_id == inferred.ontology && n.node_type == NodeType::Property
            }) {
                node.style.border_color = "#ff6d3f".to_string();
                node.style.border_width = 3.0;
            }
        }

        // 标记冲突
        for conflict in &inference_result.conflicts {
            for ontology_id in &conflict.involved_ontologies {
                if let Some(node) = graph
                    .nodes
                    .iter_mut()
                    .find(|n| n.data.ontology_id == *ontology_id)
                {
                    match conflict.severity {
                        ConflictSeverity::Error => {
                            node.style.background_color = "#ff4444".to_string();
                        }
                        ConflictSeverity::Warning => {
                            node.style.background_color = "#ffaa00".to_string();
                        }
                        ConflictSeverity::Info => {
                            node.style.background_color = "#4488ff".to_string();
                        }
                    }
                }
            }
        }

        graph
    }

    /// 生成领域本体节点
    fn generate_domain_nodes(model: &OntologyModel, graph: &mut VisualGraph) {
        for (index, domain) in model.domains.iter().enumerate() {
            let color = Self::get_domain_color(&domain.kind);
            let shape = Self::get_domain_shape(&domain.kind);

            graph.nodes.push(VisualNode {
                id: domain.id.clone(),
                label: domain.name.clone(),
                node_type: NodeType::Domain,
                position: NodePosition {
                    x: (index as f64) * 200.0,
                    y: 100.0,
                    z: None,
                },
                style: NodeStyle {
                    background_color: color.clone(),
                    border_color: "#333333".to_string(),
                    border_width: 2.0,
                    text_color: "#ffffff".to_string(),
                    size: 40.0,
                    shape,
                    icon: None,
                    opacity: 1.0,
                },
                data: NodeData {
                    ontology_id: domain.id.clone(),
                    ontology_type: format!("{:?}", domain.kind),
                    properties: HashMap::new(),
                    description: domain.description.clone(),
                },
                interactive: true,
            });
        }
    }

    /// 生成属性节点
    fn generate_property_nodes(model: &OntologyModel, graph: &mut VisualGraph) {
        let mut property_index = 0;

        for domain in &model.domains {
            for prop in &domain.properties {
                let prop_id = format!("{}_prop_{}", domain.id, prop.id);

                graph.nodes.push(VisualNode {
                    id: prop_id.clone(),
                    label: prop.name.clone(),
                    node_type: NodeType::Property,
                    position: NodePosition {
                        x: property_index as f64 * 150.0,
                        y: 300.0,
                        z: None,
                    },
                    style: NodeStyle {
                        background_color: "#e8f4f8".to_string(),
                        border_color: "#2196f3".to_string(),
                        border_width: 1.0,
                        text_color: "#333333".to_string(),
                        size: 25.0,
                        shape: NodeShape::Ellipse,
                        icon: None,
                        opacity: 0.9,
                    },
                    data: NodeData {
                        ontology_id: domain.id.clone(),
                        ontology_type: "property".to_string(),
                        properties: {
                            let mut props = HashMap::new();
                            props.insert(
                                "property_type".to_string(),
                                format!("{:?}", prop.property_type),
                            );
                            props.insert("required".to_string(), prop.required.to_string());
                            props
                        },
                        description: prop.semantic_description.clone(),
                    },
                    interactive: true,
                });

                // 添加属性到领域的边
                graph.edges.push(VisualEdge {
                    id: format!("edge_{}_to_{}", domain.id, prop_id),
                    source: domain.id.clone(),
                    target: prop_id,
                    label: Some("hasProperty".to_string()),
                    edge_type: EdgeType::Association,
                    style: EdgeStyle {
                        color: "#2196f3".to_string(),
                        width: 1.0,
                        line_style: LineStyle::Solid,
                        arrow_type: ArrowType::Arrow,
                        dashed: false,
                        opacity: 0.7,
                    },
                    is_inferred: false,
                    inference_source: None,
                });

                property_index += 1;
            }
        }
    }

    /// 生成关系边
    fn generate_relation_edges(model: &OntologyModel, graph: &mut VisualGraph) {
        for relation in &model.relations {
            let edge_type = Self::map_relation_type(&relation.relation_type);
            let arrow_type = Self::get_arrow_type(&relation.relation_type);

            graph.edges.push(VisualEdge {
                id: relation.id.clone(),
                source: relation.source_ontology.clone(),
                target: relation.target_ontology.clone(),
                label: Some(relation.name.clone()),
                edge_type: edge_type.clone(),
                style: EdgeStyle {
                    color: "#666666".to_string(),
                    width: 1.5,
                    line_style: LineStyle::Solid,
                    arrow_type,
                    dashed: false,
                    opacity: 0.8,
                },
                is_inferred: false,
                inference_source: None,
            });

            // 如果是双向关系，添加反向边
            if relation.is_bidirectional {
                graph.edges.push(VisualEdge {
                    id: format!("{}_reverse", relation.id),
                    source: relation.target_ontology.clone(),
                    target: relation.source_ontology.clone(),
                    label: Some(format!("{}_reverse", relation.name)),
                    edge_type: edge_type.clone(),
                    style: EdgeStyle {
                        color: "#666666".to_string(),
                        width: 1.5,
                        line_style: LineStyle::Dashed,
                        arrow_type: ArrowType::Arrow,
                        dashed: true,
                        opacity: 0.5,
                    },
                    is_inferred: true,
                    inference_source: Some("Bidirectional relation".to_string()),
                });
            }
        }
    }

    /// 生成继承边
    fn generate_inheritance_edges(model: &OntologyModel, graph: &mut VisualGraph) {
        for domain in &model.domains {
            for parent_id in &domain.parent_ids {
                graph.edges.push(VisualEdge {
                    id: format!("inherit_{}_to_{}", domain.id, parent_id),
                    source: domain.id.clone(),
                    target: parent_id.clone(),
                    label: Some("extends".to_string()),
                    edge_type: EdgeType::Inheritance,
                    style: EdgeStyle {
                        color: "#4caf50".to_string(),
                        width: 2.0,
                        line_style: LineStyle::Solid,
                        arrow_type: ArrowType::Triangle,
                        dashed: false,
                        opacity: 0.9,
                    },
                    is_inferred: false,
                    inference_source: None,
                });
            }

            // 等价关系
            for equiv_id in &domain.equivalent_ids {
                graph.edges.push(VisualEdge {
                    id: format!("equiv_{}_to_{}", domain.id, equiv_id),
                    source: domain.id.clone(),
                    target: equiv_id.clone(),
                    label: Some("equivalentTo".to_string()),
                    edge_type: EdgeType::Equivalence,
                    style: EdgeStyle {
                        color: "#9c27b0".to_string(),
                        width: 1.5,
                        line_style: LineStyle::Dashed,
                        arrow_type: ArrowType::Bidirectional,
                        dashed: true,
                        opacity: 0.7,
                    },
                    is_inferred: false,
                    inference_source: None,
                });
            }

            // 互斥关系
            for disjoint_id in &domain.disjoint_ids {
                graph.edges.push(VisualEdge {
                    id: format!("disjoint_{}_to_{}", domain.id, disjoint_id),
                    source: domain.id.clone(),
                    target: disjoint_id.clone(),
                    label: Some("disjointWith".to_string()),
                    edge_type: EdgeType::Disjoint,
                    style: EdgeStyle {
                        color: "#f44336".to_string(),
                        width: 1.5,
                        line_style: LineStyle::Dotted,
                        arrow_type: ArrowType::None,
                        dashed: true,
                        opacity: 0.6,
                    },
                    is_inferred: false,
                    inference_source: None,
                });
            }
        }
    }

    /// 生成交易生命周期节点
    fn generate_lifecycle_nodes(lifecycle: &TransactionLifecycle, graph: &mut VisualGraph) {
        let lifecycle_node_id = format!("lifecycle_{}", lifecycle.id);

        // 生命周期根节点
        graph.nodes.push(VisualNode {
            id: lifecycle_node_id.clone(),
            label: lifecycle.name.clone(),
            node_type: NodeType::Group,
            position: NodePosition {
                x: 400.0,
                y: 500.0,
                z: None,
            },
            style: NodeStyle {
                background_color: "#fff3e0".to_string(),
                border_color: "#ff9800".to_string(),
                border_width: 3.0,
                text_color: "#333333".to_string(),
                size: 50.0,
                shape: NodeShape::RoundedRectangle,
                icon: Some("lifecycle".to_string()),
                opacity: 1.0,
            },
            data: NodeData {
                ontology_id: lifecycle.id.clone(),
                ontology_type: "transaction_lifecycle".to_string(),
                properties: {
                    let mut props = HashMap::new();
                    props.insert(
                        "transaction_type".to_string(),
                        format!("{:?}", lifecycle.transaction_type),
                    );
                    props
                },
                description: Some(format!(
                    "Transaction type: {:?}",
                    lifecycle.transaction_type
                )),
            },
            interactive: true,
        });

        // 阶段节点
        for (index, phase) in lifecycle.phases.iter().enumerate() {
            let phase_id = format!("{}_phase_{}", lifecycle.id, phase.id);

            graph.nodes.push(VisualNode {
                id: phase_id.clone(),
                label: phase.name.clone(),
                node_type: NodeType::TransactionPhase,
                position: NodePosition {
                    x: 200.0 + (index as f64) * 200.0,
                    y: 650.0,
                    z: None,
                },
                style: NodeStyle {
                    background_color: "#ffe0b2".to_string(),
                    border_color: "#ff9800".to_string(),
                    border_width: 2.0,
                    text_color: "#333333".to_string(),
                    size: 35.0,
                    shape: NodeShape::RoundedRectangle,
                    icon: None,
                    opacity: 0.9,
                },
                data: NodeData {
                    ontology_id: lifecycle.id.clone(),
                    ontology_type: format!("phase_{:?}", phase.phase_type),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert("order".to_string(), phase.order.to_string());
                        props.insert("is_terminal".to_string(), phase.is_terminal.to_string());
                        props
                    },
                    description: None,
                },
                interactive: true,
            });

            // 连接到生命周期根节点
            graph.edges.push(VisualEdge {
                id: format!("edge_{}_to_{}", lifecycle_node_id, phase_id),
                source: lifecycle_node_id.clone(),
                target: phase_id.clone(),
                label: Some("hasPhase".to_string()),
                edge_type: EdgeType::Association,
                style: EdgeStyle {
                    color: "#ff9800".to_string(),
                    width: 1.0,
                    line_style: LineStyle::Solid,
                    arrow_type: ArrowType::Arrow,
                    dashed: false,
                    opacity: 0.6,
                },
                is_inferred: false,
                inference_source: None,
            });
        }

        // 阶段转换边
        for transition in &lifecycle.transitions {
            let from_id = format!("{}_phase_{}", lifecycle.id, transition.from_phase);
            let to_id = format!("{}_phase_{}", lifecycle.id, transition.to_phase);

            graph.edges.push(VisualEdge {
                id: transition.id.clone(),
                source: from_id,
                target: to_id,
                label: Some(transition.trigger_event.clone()),
                edge_type: EdgeType::Transition,
                style: EdgeStyle {
                    color: "#ff5722".to_string(),
                    width: 2.0,
                    line_style: LineStyle::Solid,
                    arrow_type: ArrowType::Arrow,
                    dashed: !transition.is_automatic,
                    opacity: 0.8,
                },
                is_inferred: false,
                inference_source: None,
            });
        }
    }

    /// 生成约束节点
    fn generate_constraint_nodes(model: &OntologyModel, graph: &mut VisualGraph) {
        for (index, constraint) in model.constraints.iter().enumerate() {
            let constraint_id = format!("constraint_{}", constraint.id);

            graph.nodes.push(VisualNode {
                id: constraint_id.clone(),
                label: constraint.name.clone(),
                node_type: NodeType::Constraint,
                position: NodePosition {
                    x: 100.0 + (index as f64) * 180.0,
                    y: 800.0,
                    z: None,
                },
                style: NodeStyle {
                    background_color: "#ffebee".to_string(),
                    border_color: "#f44336".to_string(),
                    border_width: 2.0,
                    text_color: "#333333".to_string(),
                    size: 30.0,
                    shape: NodeShape::Diamond,
                    icon: None,
                    opacity: 0.9,
                },
                data: NodeData {
                    ontology_id: constraint.id.clone(),
                    ontology_type: format!("{:?}", constraint.constraint_type),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert("severity".to_string(), format!("{:?}", constraint.severity));
                        props.insert("expression".to_string(), constraint.expression.clone());
                        if let Some(ref err) = constraint.error_message_template {
                            props.insert("error_message".to_string(), err.clone());
                        }
                        props
                    },
                    description: constraint.description.clone(),
                },
                interactive: true,
            });

            // 连接到目标本体
            graph.edges.push(VisualEdge {
                id: format!(
                    "edge_{}_to_{}",
                    constraint_id, constraint.scope.target_ontology
                ),
                source: constraint_id.clone(),
                target: constraint.scope.target_ontology.clone(),
                label: Some("constrains".to_string()),
                edge_type: EdgeType::Constraint,
                style: EdgeStyle {
                    color: "#f44336".to_string(),
                    width: 1.5,
                    line_style: LineStyle::Dotted,
                    arrow_type: ArrowType::Arrow,
                    dashed: true,
                    opacity: 0.6,
                },
                is_inferred: false,
                inference_source: None,
            });
        }
    }

    /// 生成计算节点
    fn generate_computation_nodes(model: &OntologyModel, graph: &mut VisualGraph) {
        for (index, computation) in model.computations.iter().enumerate() {
            let computation_id = format!("computation_{}", computation.id);

            graph.nodes.push(VisualNode {
                id: computation_id.clone(),
                label: computation.name.clone(),
                node_type: NodeType::Computation,
                position: NodePosition {
                    x: 150.0 + (index as f64) * 200.0,
                    y: 950.0,
                    z: None,
                },
                style: NodeStyle {
                    background_color: "#e8f5e9".to_string(),
                    border_color: "#4caf50".to_string(),
                    border_width: 2.0,
                    text_color: "#333333".to_string(),
                    size: 35.0,
                    shape: NodeShape::Hexagon,
                    icon: None,
                    opacity: 0.9,
                },
                data: NodeData {
                    ontology_id: computation.id.clone(),
                    ontology_type: format!("{:?}", computation.computation_type),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert("formula".to_string(), computation.formula.clone());
                        if let Some(first_trigger) = computation.trigger_conditions.first() {
                            props.insert("trigger".to_string(), first_trigger.clone());
                        }
                        props
                    },
                    description: computation.description.clone(),
                },
                interactive: true,
            });

            // 连接到输入本体
            for input in &computation.inputs {
                graph.edges.push(VisualEdge {
                    id: format!("edge_{}_input_{}", computation_id, input.id),
                    source: input.source_ontology.clone(),
                    target: computation_id.clone(),
                    label: Some(format!("input: {}", input.name)),
                    edge_type: EdgeType::Computation,
                    style: EdgeStyle {
                        color: "#4caf50".to_string(),
                        width: 1.0,
                        line_style: LineStyle::Solid,
                        arrow_type: ArrowType::Arrow,
                        dashed: false,
                        opacity: 0.6,
                    },
                    is_inferred: false,
                    inference_source: None,
                });
            }

            // 连接到输出本体
            for output in &computation.outputs {
                graph.edges.push(VisualEdge {
                    id: format!("edge_{}_output_{}", computation_id, output.id),
                    source: computation_id.clone(),
                    target: output.target_ontology.clone(),
                    label: Some(format!("output: {}", output.name)),
                    edge_type: EdgeType::Computation,
                    style: EdgeStyle {
                        color: "#4caf50".to_string(),
                        width: 1.0,
                        line_style: LineStyle::Solid,
                        arrow_type: ArrowType::Arrow,
                        dashed: false,
                        opacity: 0.6,
                    },
                    is_inferred: false,
                    inference_source: None,
                });
            }
        }
    }

    /// 应用布局
    fn apply_layout(graph: &mut VisualGraph) {
        match graph.layout.algorithm {
            LayoutAlgorithm::Hierarchical => {
                Self::apply_hierarchical_layout(graph);
            }
            LayoutAlgorithm::ForceDirected => {
                Self::apply_force_directed_layout(graph);
            }
            LayoutAlgorithm::Circular => {
                Self::apply_circular_layout(graph);
            }
            _ => {
                // 默认使用层次布局
                Self::apply_hierarchical_layout(graph);
            }
        }
    }

    /// 层次布局
    fn apply_hierarchical_layout(graph: &mut VisualGraph) {
        // 简单的层次布局实现
        let mut level_map: HashMap<String, u32> = HashMap::new();

        // 计算每个节点的层级
        for edge in &graph.edges {
            if edge.edge_type == EdgeType::Inheritance {
                let parent_level = *level_map.get(&edge.target).unwrap_or(&0);
                let child_level = level_map.entry(edge.source.clone()).or_insert(0);
                *child_level = (*child_level).max(parent_level + 1);
            }
        }

        // 根据层级重新定位节点
        let mut level_counts: HashMap<u32, u32> = HashMap::new();
        for node in &mut graph.nodes {
            let level = level_map.get(&node.id).copied().unwrap_or(0);
            let count = level_counts.entry(level).or_insert(0);

            node.position.x = (*count as f64) * 250.0;
            node.position.y = (level as f64) * 150.0;

            *count += 1;
        }
    }

    /// 力导向布局
    fn apply_force_directed_layout(graph: &mut VisualGraph) {
        // 简化的力导向布局实现
        let iterations = 100;
        let mut positions: HashMap<String, NodePosition> = HashMap::new();

        // 初始化随机位置
        for node in &graph.nodes {
            positions.insert(
                node.id.clone(),
                NodePosition {
                    x: rand::random::<f64>() * 800.0,
                    y: rand::random::<f64>() * 600.0,
                    z: None,
                },
            );
        }

        // 迭代计算
        for _ in 0..iterations {
            // 计算斥力
            for i in 0..graph.nodes.len() {
                for j in (i + 1)..graph.nodes.len() {
                    let id_i = graph.nodes[i].id.clone();
                    let id_j = graph.nodes[j].id.clone();

                    if let (Some(pos_i), Some(pos_j)) = (positions.get(&id_i), positions.get(&id_j))
                    {
                        let dx = pos_j.x - pos_i.x;
                        let dy = pos_j.y - pos_i.y;
                        let distance = (dx * dx + dy * dy).sqrt().max(1.0);

                        let force = graph.layout.physics_params.repulsion / (distance * distance);
                        let fx = (dx / distance) * force;
                        let fy = (dy / distance) * force;

                        if let Some(pos_i) = positions.get_mut(&id_i) {
                            pos_i.x -= fx;
                            pos_i.y -= fy;
                        }
                        if let Some(pos_j) = positions.get_mut(&id_j) {
                            pos_j.x += fx;
                            pos_j.y += fy;
                        }
                    }
                }
            }

            // 计算引力（边连接的节点）
            for edge in &graph.edges {
                if let (Some(pos_source), Some(pos_target)) = (
                    positions.get(&edge.source).cloned(),
                    positions.get(&edge.target).cloned(),
                ) {
                    let dx = pos_target.x - pos_source.x;
                    let dy = pos_target.y - pos_source.y;
                    let distance = (dx * dx + dy * dy).sqrt().max(1.0);

                    let force = graph.layout.physics_params.spring_stiffness * distance;
                    let fx = (dx / distance) * force;
                    let fy = (dy / distance) * force;

                    if let Some(pos_source) = positions.get_mut(&edge.source) {
                        pos_source.x += fx;
                        pos_source.y += fy;
                    }
                    if let Some(pos_target) = positions.get_mut(&edge.target) {
                        pos_target.x -= fx;
                        pos_target.y -= fy;
                    }
                }
            }
        }

        // 更新节点位置
        for node in &mut graph.nodes {
            if let Some(pos) = positions.get(&node.id) {
                node.position = pos.clone();
            }
        }
    }

    /// 圆形布局
    fn apply_circular_layout(graph: &mut VisualGraph) {
        let center_x = 400.0;
        let center_y = 300.0;
        let radius = 250.0;
        let node_count = graph.nodes.len() as f64;

        for (index, node) in graph.nodes.iter_mut().enumerate() {
            let angle = (index as f64 / node_count) * 2.0 * std::f64::consts::PI;
            node.position.x = center_x + radius * angle.cos();
            node.position.y = center_y + radius * angle.sin();
        }
    }

    /// 获取领域颜色
    fn get_domain_color(kind: &DomainKind) -> String {
        match kind {
            DomainKind::Entity => "#2196f3".to_string(),
            DomainKind::ValueObject => "#4caf50".to_string(),
            DomainKind::AggregateRoot => "#ff9800".to_string(),
            DomainKind::DomainService => "#9c27b0".to_string(),
            DomainKind::DomainEvent => "#f44336".to_string(),
            DomainKind::Enumeration => "#607d8b".to_string(),
        }
    }

    /// 获取领域形状
    fn get_domain_shape(kind: &DomainKind) -> NodeShape {
        match kind {
            DomainKind::Entity => NodeShape::RoundedRectangle,
            DomainKind::ValueObject => NodeShape::Ellipse,
            DomainKind::AggregateRoot => NodeShape::Rectangle,
            DomainKind::DomainService => NodeShape::Hexagon,
            DomainKind::DomainEvent => NodeShape::Diamond,
            DomainKind::Enumeration => NodeShape::Circle,
        }
    }

    /// 映射关系类型
    fn map_relation_type(relation_type: &RelationType) -> EdgeType {
        match relation_type {
            RelationType::Association => EdgeType::Association,
            RelationType::Aggregation => EdgeType::Aggregation,
            RelationType::Composition => EdgeType::Composition,
            RelationType::Inheritance => EdgeType::Inheritance,
            RelationType::Dependency => EdgeType::Dependency,
            RelationType::Realization => EdgeType::Custom("realization".to_string()),
            RelationType::Custom(s) => EdgeType::Custom(s.clone()),
        }
    }

    /// 获取箭头类型
    fn get_arrow_type(relation_type: &RelationType) -> ArrowType {
        match relation_type {
            RelationType::Association => ArrowType::Arrow,
            RelationType::Aggregation => ArrowType::Diamond,
            RelationType::Composition => ArrowType::FilledDiamond,
            RelationType::Inheritance => ArrowType::Triangle,
            RelationType::Dependency => ArrowType::HollowTriangle,
            RelationType::Realization => ArrowType::HollowTriangle,
            RelationType::Custom(_) => ArrowType::Arrow,
        }
    }

    /// 默认布局配置
    fn default_layout() -> LayoutConfig {
        LayoutConfig {
            algorithm: LayoutAlgorithm::Hierarchical,
            node_spacing: 200.0,
            level_spacing: 150.0,
            physics_enabled: false,
            physics_params: PhysicsParams {
                gravity: 0.1,
                repulsion: 1000.0,
                spring_stiffness: 0.05,
                damping: 0.9,
            },
        }
    }

    /// 默认视图状态
    fn default_view_state() -> ViewState {
        ViewState {
            zoom: 1.0,
            center: NodePosition {
                x: 400.0,
                y: 300.0,
                z: None,
            },
            selected_nodes: vec![],
            highlighted_nodes: vec![],
            collapsed_nodes: vec![],
            filters: vec![],
        }
    }

    /// 导出为 Mermaid 语法（增强版：classDef 颜色 + subgraph 分组 + 节点内嵌详情）
    pub fn export_to_mermaid(graph: &VisualGraph) -> String {
        let mut mermaid = String::from("graph TD\n");

        // ---- 1. 样式定义 ----
        mermaid.push_str("\n    %% 样式定义\n");
        mermaid.push_str(
            "    classDef domain fill:#e3f2fd,stroke:#1976d2,stroke-width:2px,color:#333\n",
        );
        mermaid.push_str(
            "    classDef property fill:#f3e5f5,stroke:#7b1fa2,stroke-width:1px,color:#333\n",
        );
        mermaid.push_str(
            "    classDef constraint fill:#ffebee,stroke:#f44336,stroke-width:1.5px,color:#333\n",
        );
        mermaid.push_str(
            "    classDef computation fill:#e8f5e9,stroke:#4caf50,stroke-width:1.5px,color:#333\n",
        );
        mermaid.push_str(
            "    classDef lifecycle fill:#fff3e0,stroke:#ff9800,stroke-width:2px,color:#333\n",
        );
        mermaid.push_str(
            "    classDef phase fill:#ffe0b2,stroke:#ff9800,stroke-width:1px,color:#333\n",
        );

        // 构建 ID 映射（sanitize 后的 ID）
        let id_map: HashMap<String, String> = graph
            .nodes
            .iter()
            .map(|n| (n.id.clone(), Self::sanitize_mermaid_id(&n.id)))
            .collect();

        // ---- 2. 按类型分组节点 ----
        let domains: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Domain)
            .collect();
        let properties: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Property)
            .collect();
        let constraints: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Constraint)
            .collect();
        let computations: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Computation)
            .collect();
        let lifecycle_nodes: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Group || n.node_type == NodeType::TransactionPhase)
            .collect();

        // ---- 3. 输出 subgraph ----
        Self::render_subgraph(&mut mermaid, "Domains", "📦 Domains", &domains, &id_map);
        Self::render_subgraph(
            &mut mermaid,
            "Properties",
            "🔧 Properties",
            &properties,
            &id_map,
        );
        Self::render_subgraph(
            &mut mermaid,
            "Constraints",
            "🔒 Constraints",
            &constraints,
            &id_map,
        );
        Self::render_subgraph(
            &mut mermaid,
            "Computations",
            "⚡ Computations",
            &computations,
            &id_map,
        );
        Self::render_subgraph(
            &mut mermaid,
            "Lifecycle",
            "🔄 Lifecycle",
            &lifecycle_nodes,
            &id_map,
        );

        // ---- 4. 输出边 ----
        if !graph.edges.is_empty() {
            mermaid.push_str("\n    %% Relationships\n");
            for edge in &graph.edges {
                mermaid.push_str(&format!("    {}\n", Self::render_edge(edge, &id_map)));
            }
        }

        // ---- 5. Domain 个性化颜色（保持原有背景色） ----
        if !domains.is_empty() {
            mermaid.push_str("\n    %% Domain colors\n");
            for node in &domains {
                let sid = &id_map[&node.id];
                mermaid.push_str(&format!(
                    "    style {} fill:{},stroke:{},stroke-width:{}px,color:{}\n",
                    sid,
                    node.style.background_color,
                    node.style.border_color,
                    node.style.border_width,
                    node.style.text_color
                ));
            }
        }

        // ---- 6. 应用统一样式 ----
        Self::apply_class_statements(&mut mermaid, &properties, "property", &id_map);
        Self::apply_class_statements(&mut mermaid, &constraints, "constraint", &id_map);
        Self::apply_class_statements(&mut mermaid, &computations, "computation", &id_map);
        Self::apply_class_statements(&mut mermaid, &lifecycle_nodes, "lifecycle", &id_map);

        mermaid
    }

    fn sanitize_mermaid_id(id: &str) -> String {
        id.replace([' ', '.', '/', '#'], "_")
    }

    fn render_subgraph(
        mermaid: &mut String,
        id: &str,
        title: &str,
        nodes: &[&VisualNode],
        id_map: &HashMap<String, String>,
    ) {
        if nodes.is_empty() {
            return;
        }
        mermaid.push_str(&format!("\n    subgraph {}[\"{}\"]\n", id, title));
        for node in nodes {
            mermaid.push_str(&format!(
                "        {}\n",
                Self::render_node(node, &id_map[&node.id])
            ));
        }
        mermaid.push_str("    end\n");
    }

    fn render_node(node: &VisualNode, sid: &str) -> String {
        let (open, close) = match node.style.shape {
            NodeShape::Rectangle => ("[\"", "\"]"),
            NodeShape::RoundedRectangle | NodeShape::Ellipse => ("(\"", "\")"),
            NodeShape::Circle => ("((\"", "\"))"),
            NodeShape::Diamond => ("{\"", "\"}"),
            NodeShape::Hexagon => ("{{\"", "\"}}"),
            NodeShape::Custom(_) => ("[\"", "\"]"),
        };

        let label = Self::build_enhanced_label(node);
        format!("{}{}{}{}", sid, open, label, close)
    }

    fn build_enhanced_label(node: &VisualNode) -> String {
        match node.node_type {
            NodeType::Domain => {
                let kind = &node.data.ontology_type;
                let kind_zh = match kind.as_str() {
                    "AggregateRoot" => "AggregateRoot",
                    "Entity" => "Entity",
                    "ValueObject" => "ValueObject",
                    _ => kind.as_str(),
                };
                let desc = node
                    .data
                    .description
                    .as_ref()
                    .filter(|d| !d.is_empty())
                    .map(|d| format!("<br/><small>{}</small>", d))
                    .unwrap_or_default();
                format!(
                    "{}<br/><small><i>{}</i></small>{}",
                    node.label, kind_zh, desc
                )
            }
            NodeType::Constraint => {
                let expression = node
                    .data
                    .properties
                    .get("expression")
                    .cloned()
                    .unwrap_or_default();
                let severity = node
                    .data
                    .properties
                    .get("severity")
                    .cloned()
                    .unwrap_or_default();
                let err = node
                    .data
                    .properties
                    .get("error_message")
                    .cloned()
                    .unwrap_or_default();
                let mut parts = vec![node.label.clone()];
                if !expression.is_empty() {
                    parts.push(format!("<small>{}</small>", expression));
                }
                if !severity.is_empty() {
                    let emoji = match severity.as_str() {
                        "Error" => "🔴",
                        "Warning" => "🟡",
                        "Info" => "🔵",
                        _ => "⚠️",
                    };
                    parts.push(format!("{} {}", emoji, severity));
                }
                if !err.is_empty() {
                    parts.push(format!("<small>{}</small>", err));
                }
                parts.join("<br/>")
            }
            NodeType::Computation => {
                let formula = node
                    .data
                    .properties
                    .get("formula")
                    .cloned()
                    .unwrap_or_default();
                let trigger = node
                    .data
                    .properties
                    .get("trigger")
                    .cloned()
                    .unwrap_or_default();
                let mut parts = vec![node.label.clone()];
                if !formula.is_empty() {
                    parts.push(format!("<small>{}</small>", formula));
                }
                if !trigger.is_empty() {
                    parts.push(format!("⚡ {}", trigger));
                }
                parts.join("<br/>")
            }
            NodeType::TransactionPhase => {
                let order = node
                    .data
                    .properties
                    .get("order")
                    .cloned()
                    .unwrap_or_default();
                let is_terminal = node
                    .data
                    .properties
                    .get("is_terminal")
                    .cloned()
                    .unwrap_or_default();
                let term = if is_terminal == "true" {
                    " | terminal"
                } else {
                    ""
                };
                let ty = node.data.ontology_type.replace("phase_", "");
                format!(
                    "{}<br/><small>order: {}{} | {}</small>",
                    node.label, order, term, ty
                )
            }
            NodeType::Property => {
                let prop_type = node
                    .data
                    .properties
                    .get("property_type")
                    .cloned()
                    .unwrap_or_default();
                let required = node
                    .data
                    .properties
                    .get("required")
                    .cloned()
                    .unwrap_or_default();
                let req_tag = if required == "true" { " *" } else { "" };
                let desc = node
                    .data
                    .description
                    .as_ref()
                    .filter(|d| !d.is_empty())
                    .map(|d| format!(" | {}", d))
                    .unwrap_or_default();
                format!(
                    "{}{}<br/><small>{}{}</small>",
                    node.label, req_tag, prop_type, desc
                )
            }
            _ => node.label.clone(),
        }
    }

    fn render_edge(edge: &VisualEdge, id_map: &HashMap<String, String>) -> String {
        let source = id_map.get(&edge.source).unwrap_or(&edge.source);
        let target = id_map.get(&edge.target).unwrap_or(&edge.target);

        let arrow = if edge.style.dashed {
            match edge.style.arrow_type {
                ArrowType::None => "-.-",
                ArrowType::Arrow => "-.->",
                ArrowType::Bidirectional => "<-.->",
                ArrowType::Triangle => "-.->",
                ArrowType::Diamond => "-.o",
                ArrowType::FilledDiamond => "-.*",
                ArrowType::HollowTriangle => "-.->",
            }
        } else {
            match edge.style.arrow_type {
                ArrowType::None => "---",
                ArrowType::Arrow => "-->",
                ArrowType::Bidirectional => "<-->",
                ArrowType::Triangle => "-->",
                ArrowType::Diamond => "--o",
                ArrowType::FilledDiamond => "--*",
                ArrowType::HollowTriangle => "-->",
            }
        };

        if let Some(label) = &edge.label {
            format!("{} {}|{}| {}", source, arrow, label, target)
        } else {
            format!("{} {} {}", source, arrow, target)
        }
    }

    fn apply_class_statements(
        mermaid: &mut String,
        nodes: &[&VisualNode],
        class_name: &str,
        id_map: &HashMap<String, String>,
    ) {
        if nodes.is_empty() {
            return;
        }
        let ids: Vec<String> = nodes
            .iter()
            .map(|n| id_map.get(&n.id).cloned().unwrap_or_else(|| n.id.clone()))
            .collect();
        // Mermaid class 语句限制每行约 20-30 个节点，分批输出
        for chunk in ids.chunks(25) {
            mermaid.push_str(&format!("    class {} {}\n", chunk.join(","), class_name));
        }
    }

    /// 导出为 Cytoscape JSON
    pub fn export_to_cytoscape(graph: &VisualGraph) -> serde_json::Value {
        let elements = serde_json::json!({
            "nodes": graph.nodes.iter().map(|n| {
                serde_json::json!({
                    "data": {
                        "id": n.id,
                        "label": n.label,
                        "type": format!("{:?}", n.node_type),
                    },
                    "position": {
                        "x": n.position.x,
                        "y": n.position.y,
                    },
                    "style": {
                        "background-color": n.style.background_color,
                        "border-color": n.style.border_color,
                        "border-width": n.style.border_width,
                        "width": n.style.size,
                        "height": n.style.size,
                    }
                })
            }).collect::<Vec<_>>(),
            "edges": graph.edges.iter().map(|e| {
                serde_json::json!({
                    "data": {
                        "id": e.id,
                        "source": e.source,
                        "target": e.target,
                        "label": e.label,
                        "type": format!("{:?}", e.edge_type),
                    },
                    "style": {
                        "line-color": e.style.color,
                        "width": e.style.width,
                        "line-style": if e.style.dashed { "dashed" } else { "solid" },
                    }
                })
            }).collect::<Vec<_>>(),
        });

        elements
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn create_test_model() -> OntologyModel {
        OntologyModel {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: None,
            version: "1.0".to_string(),
            domains: vec![
                DomainOntology {
                    id: "order".to_string(),
                    name: "Order".to_string(),
                    description: None,
                    kind: DomainKind::AggregateRoot,
                    parent_ids: vec![],
                    equivalent_ids: vec![],
                    disjoint_ids: vec![],
                    properties: vec![OntologyProperty {
                        id: "total".to_string(),
                        name: "total".to_string(),
                        property_type: PropertyType::DataProperty,
                        required: true,
                        cardinality: Cardinality {
                            min: Some(1),
                            max: Some(1),
                            exact: None,
                        },
                        domain: "order".to_string(),
                        range: "decimal".to_string(),
                        is_functional: true,
                        is_transitive: false,
                        is_symmetric: false,
                        constraints: vec![],
                        semantic_description: Some("Order total amount".to_string()),
                    }],
                    prefab_contract: None,
                },
                DomainOntology {
                    id: "order_item".to_string(),
                    name: "OrderItem".to_string(),
                    description: None,
                    kind: DomainKind::Entity,
                    parent_ids: vec![],
                    equivalent_ids: vec![],
                    disjoint_ids: vec![],
                    properties: vec![],
                    prefab_contract: None,
                },
            ],
            transaction_lifecycle: Some(TransactionLifecycle {
                id: "order-lifecycle".to_string(),
                name: "Order Lifecycle".to_string(),
                transaction_type: TransactionType::Unidirectional,
                phases: vec![
                    TransactionPhase {
                        id: "created".to_string(),
                        name: "Created".to_string(),
                        phase_type: PhaseType::Creation,
                        order: 1,
                        is_terminal: false,
                        entry_conditions: vec![],
                        exit_conditions: vec![],
                        invariants: vec![],
                        related_ontologies: vec!["order".to_string()],
                    },
                    TransactionPhase {
                        id: "confirmed".to_string(),
                        name: "Confirmed".to_string(),
                        phase_type: PhaseType::Confirmation,
                        order: 2,
                        is_terminal: false,
                        entry_conditions: vec![],
                        exit_conditions: vec![],
                        invariants: vec![],
                        related_ontologies: vec!["order".to_string()],
                    },
                ],
                transitions: vec![PhaseTransition {
                    id: "create-to-confirm".to_string(),
                    from_phase: "created".to_string(),
                    to_phase: "confirmed".to_string(),
                    trigger_event: "confirm".to_string(),
                    guard_conditions: vec![],
                    actions: vec![],
                    is_automatic: false,
                    timeout: None,
                }],
                symmetry: TransactionSymmetry {
                    is_symmetric: false,
                    symmetry_type: SymmetryType::Asymmetric,
                    parties: vec![],
                    symmetry_constraints: vec![],
                },
                constraints: vec![],
            }),
            relations: vec![RelationOntology {
                id: "has-items".to_string(),
                name: "hasItems".to_string(),
                relation_type: RelationType::Composition,
                source_ontology: "order".to_string(),
                target_ontology: "order_item".to_string(),
                is_bidirectional: false,
                properties: vec![],
                constraints: vec![],
                semantic_description: Some("Order contains items".to_string()),
            }],
            constraints: vec![],
            computations: vec![],
            namespaces: HashMap::new(),
            metadata: OntologyMetadata::default(),
        }
    }

    #[test]
    fn test_generate_graph() {
        let model = create_test_model();
        let graph = VisualizerEngine::generate_graph(&model);

        assert!(!graph.nodes.is_empty());
        assert!(!graph.edges.is_empty());

        // 检查是否有领域节点
        let domain_nodes: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Domain)
            .collect();
        assert_eq!(domain_nodes.len(), 2);
    }

    #[test]
    fn test_export_to_mermaid() {
        let model = create_test_model();
        let graph = VisualizerEngine::generate_graph(&model);
        let mermaid = VisualizerEngine::export_to_mermaid(&graph);

        assert!(mermaid.starts_with("graph TD"));
        assert!(mermaid.contains("order"));
        assert!(mermaid.contains("order_item"));
        // 验证增强功能
        assert!(mermaid.contains("classDef domain"));
        assert!(mermaid.contains("subgraph Domains"));
        assert!(mermaid.contains("subgraph Lifecycle"));
    }

    #[test]
    fn test_export_to_cytoscape() {
        let model = create_test_model();
        let graph = VisualizerEngine::generate_graph(&model);
        let cytoscape = VisualizerEngine::export_to_cytoscape(&graph);

        assert!(cytoscape.get("nodes").is_some());
        assert!(cytoscape.get("edges").is_some());
    }
}
