// ============================================================
// SCALONY PLIK – ONYX NODE EDITOR Z PEŁNYM SYSTEMEM WĘZŁÓW
// DLA ICED 0.15.0
// ============================================================
use iced::font::Family;
use iced::mouse::{Cursor, Interaction};
use iced::widget::canvas::{self, Frame, Geometry, Path, Program, Stroke, Text as CanvasText};
use iced::widget::mouse_area;
use iced::{
    Alignment, Background, Border, Color, Element, Event, Font, Length, Padding, Point, Rectangle,
    Size, Subscription, Task, Theme, Vector, event, keyboard, mouse,
    widget::{
        Space, button, canvas::Canvas, column, container, pane_grid, row, scrollable, stack, text,
        text_input,
    },
};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================
// STAŁE KOLORÓW (NOWY STYL PANELI)
// ============================================================
const BG_PRIM: Color = Color::from_rgb8(38, 38, 38);
const BG_SECO: Color = Color::from_rgb8(34, 34, 34);
const SEPA_CO: Color = Color::from_rgb8(27, 27, 27);
const CT_PRIM: Color = Color::from_rgb8(179, 179, 179);

// ============================================================
// STAŁE I MOTYW (ORYGINALNY THEME, ALE ZMIENIAMY PANELE)
// ============================================================
pub struct ThemeColors {
    pub bg: Color,
    pub grid_minor: Color,
    pub grid_major: Color,
    pub node_bg: Color,
    pub node_border: Color,
    pub accent: Color,
    pub wire: Color,
    pub wire_pending: Color,
    pub port_label: Color,
    pub panel_bg: Color,
    pub panel_border: Color,
    pub text: Color,
    pub text_dim: Color,
    pub danger: Color,
}

pub const THEME: ThemeColors = ThemeColors {
    bg: Color {
        r: 0.11,
        g: 0.11,
        b: 0.13,
        a: 1.0,
    },
    grid_minor: Color {
        r: 0.22,
        g: 0.22,
        b: 0.26,
        a: 1.0,
    },
    grid_major: Color {
        r: 0.30,
        g: 0.30,
        b: 0.36,
        a: 1.0,
    },
    node_bg: Color {
        r: 0.17,
        g: 0.18,
        b: 0.21,
        a: 1.0,
    },
    node_border: Color {
        r: 0.55,
        g: 0.55,
        b: 0.60,
        a: 1.0,
    },
    accent: Color {
        r: 0.40,
        g: 0.70,
        b: 1.00,
        a: 1.0,
    },
    wire: Color {
        r: 0.65,
        g: 0.65,
        b: 0.70,
        a: 1.0,
    },
    wire_pending: Color {
        r: 0.90,
        g: 0.75,
        b: 0.30,
        a: 1.0,
    },
    port_label: Color {
        r: 0.80,
        g: 0.80,
        b: 0.85,
        a: 1.0,
    },
    panel_bg: Color {
        r: 0.14,
        g: 0.14,
        b: 0.17,
        a: 1.0,
    },
    panel_border: Color {
        r: 0.22,
        g: 0.22,
        b: 0.28,
        a: 1.0,
    },
    text: Color {
        r: 0.92,
        g: 0.92,
        b: 0.94,
        a: 1.0,
    },
    text_dim: Color {
        r: 0.55,
        g: 0.55,
        b: 0.60,
        a: 1.0,
    },
    danger: Color {
        r: 0.90,
        g: 0.35,
        b: 0.35,
        a: 1.0,
    },
};

pub const NODE_WIDTH: f32 = 180.0;
pub const NODE_HEADER_HEIGHT: f32 = 28.0;
pub const NODE_PREVIEW_HEIGHT: f32 = 160.0;
pub const PORT_ROW_HEIGHT: f32 = 22.0;
pub const PORT_RADIUS: f32 = 6.0;
pub const PORT_PADDING_X: f32 = 10.0;

// ============================================================
// PORTY, WĘZŁY, POŁĄCZENIA, GRAF
// ============================================================
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PortType {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Color,
    Texture,
    Any,
}

impl PortType {
    pub fn color(&self) -> Color {
        match self {
            PortType::Float => Color::from_rgb(0.60, 0.85, 0.60),
            PortType::Vec2 => Color::from_rgb(0.60, 0.70, 1.00),
            PortType::Vec3 => Color::from_rgb(0.80, 0.60, 1.00),
            PortType::Vec4 => Color::from_rgb(1.00, 0.60, 0.80),
            PortType::Color => Color::from_rgb(1.00, 0.85, 0.30),
            PortType::Texture => Color::from_rgb(1.00, 0.65, 0.30),
            PortType::Any => Color::from_rgb(0.70, 0.70, 0.70),
        }
    }

    pub fn compatible_with(&self, other: &PortType) -> bool {
        *self == PortType::Any || *other == PortType::Any || self == other
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Port {
    pub label: String,
    pub port_type: PortType,
}

impl Port {
    pub fn new(label: impl Into<String>, port_type: PortType) -> Self {
        Self {
            label: label.into(),
            port_type,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum NodeCategory {
    Input,
    Texture,
    Math,
    Color,
    Output,
}

impl NodeCategory {
    pub fn all() -> Vec<NodeCategory> {
        vec![
            NodeCategory::Input,
            NodeCategory::Texture,
            NodeCategory::Math,
            NodeCategory::Color,
            NodeCategory::Output,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            NodeCategory::Input => "Wejście",
            NodeCategory::Texture => "Tekstura",
            NodeCategory::Math => "Matematyka",
            NodeCategory::Color => "Kolor",
            NodeCategory::Output => "Wyjście",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            NodeCategory::Input => Color::from_rgb(0.25, 0.45, 0.70),
            NodeCategory::Texture => Color::from_rgb(0.55, 0.30, 0.65),
            NodeCategory::Math => Color::from_rgb(0.25, 0.55, 0.45),
            NodeCategory::Color => Color::from_rgb(0.70, 0.45, 0.20),
            NodeCategory::Output => Color::from_rgb(0.65, 0.25, 0.25),
        }
    }

    pub fn nodes(&self) -> Vec<NodeKind> {
        match self {
            NodeCategory::Input => vec![
                NodeKind::UVInput,
                NodeKind::VertexColor,
                NodeKind::Time,
                NodeKind::CameraPos,
            ],
            NodeCategory::Texture => vec![NodeKind::TextureSample, NodeKind::NormalMap],
            NodeCategory::Math => vec![
                NodeKind::Add,
                NodeKind::Multiply,
                NodeKind::Lerp,
                NodeKind::Clamp,
                NodeKind::Power,
                NodeKind::Fresnel,
                NodeKind::Mix,
                NodeKind::Gamma,
            ],
            NodeCategory::Color => vec![
                NodeKind::ColorConstant,
                NodeKind::HsvToRgb,
                NodeKind::RgbToHsv,
            ],
            NodeCategory::Output => vec![NodeKind::PBROutput, NodeKind::UnlitOutput],
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum NodeKind {
    UVInput,
    VertexColor,
    Time,
    CameraPos,
    TextureSample,
    NormalMap,
    Add,
    Multiply,
    Lerp,
    Clamp,
    Power,
    Fresnel,
    Mix,
    Gamma,
    ColorConstant,
    HsvToRgb,
    RgbToHsv,
    PBROutput,
    UnlitOutput,
}

impl NodeKind {
    pub fn label(&self) -> &'static str {
        match self {
            NodeKind::UVInput => "UV Input",
            NodeKind::VertexColor => "Vertex Color",
            NodeKind::Time => "Time",
            NodeKind::CameraPos => "Camera Position",
            NodeKind::TextureSample => "Texture Sample",
            NodeKind::NormalMap => "Normal Map",
            NodeKind::Add => "Add",
            NodeKind::Multiply => "Multiply",
            NodeKind::Lerp => "Lerp",
            NodeKind::Clamp => "Clamp",
            NodeKind::Power => "Power",
            NodeKind::Fresnel => "Fresnel",
            NodeKind::Mix => "Mix",
            NodeKind::Gamma => "Gamma",
            NodeKind::ColorConstant => "Color",
            NodeKind::HsvToRgb => "HSV to RGB",
            NodeKind::RgbToHsv => "RGB to HSV",
            NodeKind::PBROutput => "PBR Output",
            NodeKind::UnlitOutput => "Unlit Output",
        }
    }

    pub fn category(&self) -> NodeCategory {
        match self {
            NodeKind::UVInput | NodeKind::VertexColor | NodeKind::Time | NodeKind::CameraPos => {
                NodeCategory::Input
            }
            NodeKind::TextureSample | NodeKind::NormalMap => NodeCategory::Texture,
            NodeKind::Add
            | NodeKind::Multiply
            | NodeKind::Lerp
            | NodeKind::Clamp
            | NodeKind::Power
            | NodeKind::Fresnel
            | NodeKind::Mix
            | NodeKind::Gamma => NodeCategory::Math,
            NodeKind::ColorConstant | NodeKind::HsvToRgb | NodeKind::RgbToHsv => {
                NodeCategory::Color
            }
            NodeKind::PBROutput | NodeKind::UnlitOutput => NodeCategory::Output,
        }
    }

    pub fn header_color(&self) -> Color {
        self.category().color()
    }

    pub fn ports(&self) -> (Vec<Port>, Vec<Port>) {
        match self {
            NodeKind::UVInput => (vec![], vec![Port::new("UV", PortType::Vec2)]),
            NodeKind::VertexColor => (vec![], vec![Port::new("Color", PortType::Color)]),
            NodeKind::Time => (vec![], vec![Port::new("Time", PortType::Float)]),
            NodeKind::CameraPos => (vec![], vec![Port::new("Position", PortType::Vec3)]),
            NodeKind::TextureSample => (
                vec![
                    Port::new("UV", PortType::Vec2),
                    Port::new("Sampler", PortType::Texture),
                ],
                vec![
                    Port::new("RGBA", PortType::Color),
                    Port::new("RGB", PortType::Vec3),
                    Port::new("A", PortType::Float),
                ],
            ),
            NodeKind::NormalMap => (
                vec![
                    Port::new("Texture", PortType::Color),
                    Port::new("Scale", PortType::Float),
                ],
                vec![Port::new("Normal", PortType::Vec3)],
            ),
            NodeKind::Add => (
                vec![Port::new("A", PortType::Any), Port::new("B", PortType::Any)],
                vec![Port::new("Out", PortType::Any)],
            ),
            NodeKind::Multiply => (
                vec![Port::new("A", PortType::Any), Port::new("B", PortType::Any)],
                vec![Port::new("Out", PortType::Any)],
            ),
            NodeKind::Lerp => (
                vec![
                    Port::new("A", PortType::Any),
                    Port::new("B", PortType::Any),
                    Port::new("T", PortType::Float),
                ],
                vec![Port::new("Out", PortType::Any)],
            ),
            NodeKind::Clamp => (
                vec![
                    Port::new("In", PortType::Float),
                    Port::new("Min", PortType::Float),
                    Port::new("Max", PortType::Float),
                ],
                vec![Port::new("Out", PortType::Float)],
            ),
            NodeKind::Power => (
                vec![
                    Port::new("Base", PortType::Float),
                    Port::new("Exp", PortType::Float),
                ],
                vec![Port::new("Out", PortType::Float)],
            ),
            NodeKind::Fresnel => (
                vec![
                    Port::new("Normal", PortType::Vec3),
                    Port::new("Power", PortType::Float),
                ],
                vec![Port::new("Out", PortType::Float)],
            ),
            NodeKind::Mix => (
                vec![
                    Port::new("A", PortType::Any),
                    Port::new("B", PortType::Any),
                    Port::new("Factor", PortType::Float),
                ],
                vec![Port::new("Out", PortType::Any)],
            ),
            NodeKind::Gamma => (
                vec![Port::new("In", PortType::Color)],
                vec![Port::new("Out", PortType::Color)],
            ),
            NodeKind::ColorConstant => (vec![], vec![Port::new("Color", PortType::Color)]),
            NodeKind::HsvToRgb => (
                vec![Port::new("HSV", PortType::Vec3)],
                vec![Port::new("RGB", PortType::Vec3)],
            ),
            NodeKind::RgbToHsv => (
                vec![Port::new("RGB", PortType::Vec3)],
                vec![Port::new("HSV", PortType::Vec3)],
            ),
            NodeKind::PBROutput => (
                vec![
                    Port::new("Albedo", PortType::Color),
                    Port::new("Normal", PortType::Vec3),
                    Port::new("Metallic", PortType::Float),
                    Port::new("Roughness", PortType::Float),
                    Port::new("Emission", PortType::Color),
                ],
                vec![],
            ),
            NodeKind::UnlitOutput => (vec![Port::new("Color", PortType::Color)], vec![]),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Node {
    pub id: Uuid,
    pub kind: NodeKind,
    pub position: [f32; 2],
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
    pub selected: bool,
}

impl Node {
    pub fn new(kind: NodeKind, position: Point) -> Self {
        let (inputs, outputs) = kind.ports();
        Self {
            id: Uuid::new_v4(),
            kind,
            position: [position.x, position.y],
            inputs,
            outputs,
            selected: false,
        }
    }

    pub fn pos(&self) -> Point {
        Point::new(self.position[0], self.position[1])
    }

    pub fn height(&self) -> f32 {
        let port_count = self.inputs.len().max(self.outputs.len());
        NODE_HEADER_HEIGHT + NODE_PREVIEW_HEIGHT + port_count as f32 * PORT_ROW_HEIGHT + 8.0
    }

    pub fn bounds(&self) -> Rectangle {
        Rectangle::new(self.pos(), Size::new(NODE_WIDTH, self.height()))
    }

    pub fn input_port_pos(&self, i: usize) -> Point {
        let y = self.position[1]
            + NODE_HEADER_HEIGHT
            + NODE_PREVIEW_HEIGHT
            + i as f32 * PORT_ROW_HEIGHT
            + PORT_ROW_HEIGHT * 0.5;
        Point::new(self.position[0], y)
    }

    pub fn output_port_pos(&self, i: usize) -> Point {
        let y = self.position[1]
            + NODE_HEADER_HEIGHT
            + NODE_PREVIEW_HEIGHT
            + i as f32 * PORT_ROW_HEIGHT
            + PORT_ROW_HEIGHT * 0.5;
        Point::new(self.position[0] + NODE_WIDTH, y)
    }

    pub fn hit_test_output_port_zoomed(&self, p: Point, zoom: f32) -> Option<usize> {
        let hit_r = (PORT_RADIUS * 2.0).max(16.0 / zoom.sqrt());
        for i in 0..self.outputs.len() {
            let center = self.output_port_pos(i);
            if (p.x - center.x).powi(2) + (p.y - center.y).powi(2) <= hit_r * hit_r {
                return Some(i);
            }
        }
        None
    }

    pub fn hit_test_input_port_zoomed(&self, p: Point, zoom: f32) -> Option<usize> {
        let hit_r = (PORT_RADIUS * 2.0).max(16.0 / zoom.sqrt());
        for i in 0..self.inputs.len() {
            let center = self.input_port_pos(i);
            if (p.x - center.x).powi(2) + (p.y - center.y).powi(2) <= hit_r * hit_r {
                return Some(i);
            }
        }
        None
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Connection {
    pub id: Uuid,
    pub src_node: Uuid,
    pub src_port: usize,
    pub dst_node: Uuid,
    pub dst_port: usize,
}

impl Connection {
    pub fn new(src_node: Uuid, src_port: usize, dst_node: Uuid, dst_port: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            src_node,
            src_port,
            dst_node,
            dst_port,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PendingConnection {
    pub src_node: Uuid,
    pub src_port: usize,
    pub src_pos: Point,
    pub current_pos: Point,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Graph {
    pub nodes: HashMap<Uuid, Node>,
    pub connections: Vec<Connection>,
    node_order: Vec<Uuid>,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, kind: NodeKind, position: Point) -> Uuid {
        let node = Node::new(kind, position);
        let id = node.id;
        self.nodes.insert(id, node);
        self.node_order.push(id);
        id
    }

    pub fn remove_node(&mut self, id: Uuid) {
        self.nodes.remove(&id);
        self.node_order.retain(|n| *n != id);
        self.connections
            .retain(|c| c.src_node != id && c.dst_node != id);
    }

    pub fn duplicate_node(&mut self, id: Uuid) -> Option<Uuid> {
        let node = self.nodes.get(&id)?.clone();
        let offset = Point::new(node.position[0] + 30.0, node.position[1] + 30.0);
        let new_id = self.add_node(node.kind.clone(), offset);
        Some(new_id)
    }

    pub fn nodes_ordered(&self) -> impl Iterator<Item = &Node> {
        self.node_order.iter().filter_map(|id| self.nodes.get(id))
    }

    pub fn bring_to_front(&mut self, id: Uuid) {
        if self.nodes.contains_key(&id) {
            self.node_order.retain(|n| *n != id);
            self.node_order.push(id);
        }
    }

    pub fn select_only(&mut self, id: Uuid) {
        for node in self.nodes.values_mut() {
            node.selected = node.id == id;
        }
    }

    pub fn deselect_all(&mut self) {
        for node in self.nodes.values_mut() {
            node.selected = false;
        }
    }

    pub fn select_rect(&mut self, rect: Rectangle) {
        for node in self.nodes.values_mut() {
            let nb = node.bounds();
            node.selected = rect.intersects(&nb);
        }
    }

    pub fn selected_ids(&self) -> Vec<Uuid> {
        self.nodes
            .values()
            .filter(|n| n.selected)
            .map(|n| n.id)
            .collect()
    }

    pub fn move_selected(&mut self, delta: Vector) {
        for node in self.nodes.values_mut() {
            if node.selected {
                node.position[0] += delta.x;
                node.position[1] += delta.y;
            }
        }
    }

    pub fn add_connection(
        &mut self,
        src_node: Uuid,
        src_port: usize,
        dst_node: Uuid,
        dst_port: usize,
    ) -> bool {
        let src_type = {
            let node = match self.nodes.get(&src_node) {
                Some(n) => n,
                None => return false,
            };
            match node.outputs.get(src_port) {
                Some(p) => p.port_type.clone(),
                None => return false,
            }
        };
        let dst_type = {
            let node = match self.nodes.get(&dst_node) {
                Some(n) => n,
                None => return false,
            };
            match node.inputs.get(dst_port) {
                Some(p) => p.port_type.clone(),
                None => return false,
            }
        };

        if !src_type.compatible_with(&dst_type) {
            return false;
        }

        if src_node == dst_node {
            return false;
        }

        self.connections
            .retain(|c| !(c.dst_node == dst_node && c.dst_port == dst_port));
        self.connections
            .push(Connection::new(src_node, src_port, dst_node, dst_port));
        true
    }

    pub fn remove_connection(&mut self, id: Uuid) {
        self.connections.retain(|c| c.id != id);
    }

    pub fn get_connection(&self, id: Uuid) -> Option<&Connection> {
        self.connections.iter().find(|c| c.id == id)
    }

    pub fn connection_at_point(&self, p: Point, threshold: f32) -> Option<Uuid> {
        for conn in &self.connections {
            let src_node = self.nodes.get(&conn.src_node)?;
            let dst_node = self.nodes.get(&conn.dst_node)?;

            let src_pos = src_node.output_port_pos(conn.src_port);
            let dst_pos = dst_node.input_port_pos(conn.dst_port);

            if point_near_cubic_bezier(p, src_pos, dst_pos, threshold) {
                return Some(conn.id);
            }
        }
        None
    }

    pub fn connection_at_point_excluding(
        &self,
        p: Point,
        threshold: f32,
        exclude_node: Uuid,
    ) -> Option<Uuid> {
        for conn in &self.connections {
            if conn.src_node == exclude_node || conn.dst_node == exclude_node {
                continue;
            }
            let src_node = self.nodes.get(&conn.src_node)?;
            let dst_node = self.nodes.get(&conn.dst_node)?;
            let src_pos = src_node.output_port_pos(conn.src_port);
            let dst_pos = dst_node.input_port_pos(conn.dst_port);
            if point_near_cubic_bezier(p, src_pos, dst_pos, threshold) {
                return Some(conn.id);
            }
        }
        None
    }

    pub fn auto_layout(&mut self, selected_only: bool) {
        use std::collections::{HashMap, HashSet, VecDeque};

        let target_ids: Vec<Uuid> = if selected_only {
            self.selected_ids()
        } else {
            self.nodes.keys().copied().collect()
        };

        if target_ids.is_empty() {
            return;
        }

        let target_set: HashSet<Uuid> = target_ids.iter().copied().collect();

        let mut successors: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut predecessors: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        for id in &target_ids {
            successors.insert(*id, Vec::new());
            predecessors.insert(*id, Vec::new());
        }

        for conn in &self.connections {
            if target_set.contains(&conn.src_node) && target_set.contains(&conn.dst_node) {
                successors
                    .entry(conn.src_node)
                    .or_default()
                    .push(conn.dst_node);
                predecessors
                    .entry(conn.dst_node)
                    .or_default()
                    .push(conn.src_node);
            }
        }

        let mut in_degree: HashMap<Uuid, usize> = target_ids
            .iter()
            .map(|id| (*id, predecessors[id].len()))
            .collect();

        let mut layer: HashMap<Uuid, usize> = HashMap::new();
        let mut queue: VecDeque<Uuid> = target_ids
            .iter()
            .filter(|id| in_degree[*id] == 0)
            .copied()
            .collect();

        for &id in &queue {
            layer.insert(id, 0);
        }

        while let Some(id) = queue.pop_front() {
            let current_layer = layer[&id];
            if let Some(succs) = successors.get(&id) {
                for &s in succs {
                    let entry = layer.entry(s).or_insert(0);
                    *entry = (*entry).max(current_layer + 1);
                    let deg = in_degree.entry(s).or_insert(0);
                    if *deg > 0 {
                        *deg -= 1;
                    }
                    if *deg == 0 {
                        queue.push_back(s);
                    }
                }
            }
        }

        let max_layer = layer.values().copied().max().unwrap_or(0);
        for id in &target_ids {
            layer.entry(*id).or_insert(max_layer + 1);
        }

        let mut by_layer: HashMap<usize, Vec<Uuid>> = HashMap::new();
        for id in &target_ids {
            by_layer.entry(layer[id]).or_default().push(*id);
        }

        let mut layer_order: HashMap<usize, Vec<Uuid>> = HashMap::new();
        let mut sorted_layers: Vec<usize> = by_layer.keys().copied().collect();
        sorted_layers.sort();

        for &l in &sorted_layers {
            let mut nodes = by_layer[&l].clone();
            nodes.sort_by_key(|id: &Uuid| id.as_u128());
            layer_order.insert(l, nodes);
        }

        const MAX_ITER: usize = 10;
        let empty_vec = Vec::new();

        for _ in 0..MAX_ITER {
            for i in 0..sorted_layers.len() {
                let current_layer = sorted_layers[i];
                let nodes = layer_order[&current_layer].clone();
                if nodes.is_empty() {
                    continue;
                }

                let mut barycenter: HashMap<Uuid, f32> = HashMap::new();
                for &node in &nodes {
                    let preds = predecessors.get(&node).unwrap_or(&empty_vec);
                    if preds.is_empty() {
                        barycenter.insert(node, 0.0);
                    } else {
                        let sum: usize = preds
                            .iter()
                            .filter_map(|p| {
                                let pred_layer = layer[p];
                                layer_order
                                    .get(&pred_layer)
                                    .and_then(|vec: &Vec<Uuid>| vec.iter().position(|&id| id == *p))
                            })
                            .sum();
                        barycenter.insert(node, sum as f32 / preds.len() as f32);
                    }
                }

                let mut sorted_nodes = nodes;
                sorted_nodes.sort_by(|a, b| {
                    barycenter[a]
                        .partial_cmp(&barycenter[b])
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                layer_order.insert(current_layer, sorted_nodes);
            }

            for i in (0..sorted_layers.len()).rev() {
                let current_layer = sorted_layers[i];
                let nodes = layer_order[&current_layer].clone();
                if nodes.is_empty() {
                    continue;
                }

                let mut barycenter: HashMap<Uuid, f32> = HashMap::new();
                for &node in &nodes {
                    let succs = successors.get(&node).unwrap_or(&empty_vec);
                    if succs.is_empty() {
                        barycenter.insert(node, 0.0);
                    } else {
                        let sum: usize = succs
                            .iter()
                            .filter_map(|s| {
                                let succ_layer = layer[s];
                                layer_order
                                    .get(&succ_layer)
                                    .and_then(|vec: &Vec<Uuid>| vec.iter().position(|&id| id == *s))
                            })
                            .sum();
                        barycenter.insert(node, sum as f32 / succs.len() as f32);
                    }
                }

                let mut sorted_nodes = nodes;
                sorted_nodes.sort_by(|a, b| {
                    barycenter[a]
                        .partial_cmp(&barycenter[b])
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                layer_order.insert(current_layer, sorted_nodes);
            }
        }

        const H_GAP: f32 = 60.0;
        const V_GAP: f32 = 30.0;
        const START_X: f32 = 60.0;
        const START_Y: f32 = 60.0;
        let col_w = NODE_WIDTH + H_GAP;

        let mut positions: Vec<(Uuid, f32, f32)> = Vec::new();

        for &col in &sorted_layers {
            let nodes_in_col = &layer_order[&col];
            let x = START_X + col as f32 * col_w;
            let mut y = START_Y;
            for &id in nodes_in_col {
                let h = self.nodes[&id].height();
                positions.push((id, x, y));
                y += h + V_GAP;
            }
        }

        for (id, x, y) in positions {
            if let Some(node) = self.nodes.get_mut(&id) {
                node.position[0] = x;
                node.position[1] = y;
            }
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    pub fn from_json(s: &str) -> Option<Self> {
        serde_json::from_str(s).ok()
    }
}

fn point_near_cubic_bezier(p: Point, p0: Point, p3: Point, threshold: f32) -> bool {
    let dx = p3.x - p0.x;
    let ctrl_offset = (dx.abs() * 0.5).max(60.0);
    let p1 = Point::new(p0.x + ctrl_offset, p0.y);
    let p2 = Point::new(p3.x - ctrl_offset, p3.y);

    for i in 0..=20 {
        let t = i as f32 / 20.0;
        let it = 1.0 - t;
        let bx = it * it * it * p0.x
            + 3.0 * it * it * t * p1.x
            + 3.0 * it * t * t * p2.x
            + t * t * t * p3.x;
        let by = it * it * it * p0.y
            + 3.0 * it * it * t * p1.y
            + 3.0 * it * t * t * p2.y
            + t * t * t * p3.y;
        let ddx = p.x - bx;
        let ddy = p.y - by;
        if ddx * ddx + ddy * ddy < threshold * threshold {
            return true;
        }
    }
    false
}

// ============================================================
// STAN KANWY I RYSOWANIE
// ============================================================
#[derive(Debug, Clone, Default)]
pub struct CanvasState {
    pub pan: Vector,
    pub zoom: f32,
    pub interaction: InteractionState,
    pub hover_node: Option<Uuid>,
    pub hover_port: Option<(Uuid, PortSide, usize)>,
    pub hovered_wire: Option<Uuid>,
}

impl CanvasState {
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            ..Default::default()
        }
    }

    pub fn screen_to_world(&self, p: Point) -> Point {
        Point::new(
            (p.x - self.pan.x) / self.zoom,
            (p.y - self.pan.y) / self.zoom,
        )
    }

    pub fn world_to_screen(&self, p: Point) -> Point {
        Point::new(p.x * self.zoom + self.pan.x, p.y * self.zoom + self.pan.y)
    }
}

#[derive(Debug, Clone, Default)]
pub enum InteractionState {
    #[default]
    Idle,
    Panning {
        last: Point,
    },
    DraggingNode {
        id: Uuid,
        offset: Vector,
        last_world: Point,
    },
    DraggingWire(PendingConnection),
    Selecting {
        start: Point,
        current: Point,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortSide {
    Input,
    Output,
}

#[derive(Debug)]
pub struct CanvasCaches {
    pub wires: canvas::Cache,
}

impl CanvasCaches {
    pub fn new() -> Self {
        Self {
            wires: canvas::Cache::new(),
        }
    }

    pub fn invalidate_view(&self) {
        self.wires.clear();
    }

    pub fn invalidate_nodes(&self) {
        self.wires.clear();
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FrameKey {
    pan_x2: i32,
    pan_y2: i32,
    zoom_k: u32,
}

impl FrameKey {
    fn from(state: &CanvasState) -> Self {
        Self {
            pan_x2: (state.pan.x * 2.0) as i32,
            pan_y2: (state.pan.y * 2.0) as i32,
            zoom_k: (state.zoom * 1000.0) as u32,
        }
    }
}

pub struct NodeCanvasProgram<'a> {
    pub graph: &'a Graph,
    pub state: &'a CanvasState,
    pub pending: Option<&'a PendingConnection>,
    pub caches: &'a CanvasCaches,
}

impl<'a> Program<Message> for NodeCanvasProgram<'a> {
    type State = FrameKey;

    fn draw(
        &self,
        prev_key: &FrameKey,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let cur_key = FrameKey::from(self.state);
        if *prev_key != cur_key {
            self.caches.invalidate_view();
        }

        let vp = bounds.size();
        let margin = 120.0;

        let bg = self.caches.wires.draw(renderer, vp, |frame| {
            frame.fill_rectangle(Point::ORIGIN, vp, THEME.bg);
        });

        let mut main = Frame::new(renderer, vp);

        for conn in &self.graph.connections {
            if let (Some(src), Some(dst)) = (
                self.graph.nodes.get(&conn.src_node),
                self.graph.nodes.get(&conn.dst_node),
            ) {
                let src_sp = self
                    .state
                    .world_to_screen(src.output_port_pos(conn.src_port));
                let dst_sp = self
                    .state
                    .world_to_screen(dst.input_port_pos(conn.dst_port));
                let src_off = src_sp.x < -margin
                    || src_sp.x > vp.width + margin
                    || src_sp.y < -margin
                    || src_sp.y > vp.height + margin;
                let dst_off = dst_sp.x < -margin
                    || dst_sp.x > vp.width + margin
                    || dst_sp.y < -margin
                    || dst_sp.y > vp.height + margin;
                if src_off && dst_off {
                    continue;
                }
                let base_color = src
                    .outputs
                    .get(conn.src_port)
                    .map(|p| p.port_type.color())
                    .unwrap_or(THEME.wire);
                let color = if self.state.hovered_wire == Some(conn.id) {
                    Color {
                        r: (base_color.r + 0.4).min(1.0),
                        g: (base_color.g + 0.4).min(1.0),
                        b: (base_color.b + 0.4).min(1.0),
                        a: 1.0,
                    }
                } else {
                    base_color
                };
                let width_mult = if self.state.hovered_wire == Some(conn.id) {
                    2.0
                } else {
                    1.0
                };
                draw_bezier_wire(
                    &mut main,
                    src_sp,
                    dst_sp,
                    color,
                    self.state.zoom,
                    width_mult,
                );
            }
        }

        for node in self.graph.nodes_ordered() {
            draw_node(&mut main, node, self.state, vp, margin);
        }

        let mut overlay = Frame::new(renderer, vp);

        if let Some((node_id, side, port_idx)) = self.state.hover_port {
            if let Some(node) = self.graph.nodes.get(&node_id) {
                let port_pos = match side {
                    PortSide::Input => node.input_port_pos(port_idx),
                    PortSide::Output => node.output_port_pos(port_idx),
                };
                let screen_pos = self.state.world_to_screen(port_pos);
                draw_port_highlight(&mut overlay, screen_pos, self.state.zoom);
            }
        }

        if let InteractionState::Selecting { start, current } = &self.state.interaction {
            let rect = rect_from_two(
                self.state.world_to_screen(*start),
                self.state.world_to_screen(*current),
            );
            let path = Path::rectangle(rect.position(), rect.size());
            overlay.fill(
                &path,
                Color {
                    a: 0.08,
                    ..THEME.accent
                },
            );
            overlay.stroke(
                &path,
                Stroke::default()
                    .with_color(Color {
                        a: 0.6,
                        ..THEME.accent
                    })
                    .with_width(1.0),
            );
        }

        if let Some(pending) = self.pending {
            let src_sp = self.state.world_to_screen(pending.src_pos);
            let cur_sp = self.state.world_to_screen(pending.current_pos);
            draw_bezier_wire(
                &mut overlay,
                src_sp,
                cur_sp,
                THEME.wire_pending,
                self.state.zoom,
                1.0,
            );
        }

        vec![bg, main.into_geometry(), overlay.into_geometry()]
    }

    fn update(
        &self,
        key: &mut FrameKey,
        event: &Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<canvas::Action<Message>> {
        *key = FrameKey::from(self.state);

        match event {
            Event::Mouse(mouse_event) => {
                if let Some(position) = cursor.position_in(bounds) {
                    let canvas_msg = match mouse_event {
                        mouse::Event::CursorMoved { .. } => CanvasMsg::MouseMoved(position),
                        mouse::Event::ButtonPressed(button) => match button {
                            mouse::Button::Left => CanvasMsg::LeftPressed(position),
                            mouse::Button::Right => CanvasMsg::RightPressed(position),
                            mouse::Button::Middle => CanvasMsg::MiddlePressed(position),
                            _ => return None,
                        },
                        mouse::Event::ButtonReleased(button) => match button {
                            mouse::Button::Left => CanvasMsg::LeftReleased(position),
                            mouse::Button::Right => CanvasMsg::RightReleased(position),
                            mouse::Button::Middle => CanvasMsg::MiddleReleased(position),
                            _ => return None,
                        },
                        mouse::Event::WheelScrolled { delta } => {
                            CanvasMsg::Scrolled(position, *delta)
                        }
                        _ => return None,
                    };
                    return Some(canvas::Action::publish(Message::CanvasEvent(canvas_msg)));
                }
            }
            _ => {}
        }
        None
    }

    fn mouse_interaction(
        &self,
        _state: &FrameKey,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Interaction {
        if !cursor.is_over(bounds) {
            return Interaction::default();
        }
        match &self.state.interaction {
            InteractionState::Panning { .. } => Interaction::Grabbing,
            InteractionState::DraggingNode { .. } => Interaction::Grabbing,
            InteractionState::DraggingWire(_) => Interaction::Crosshair,
            InteractionState::Selecting { .. } => Interaction::Crosshair,
            InteractionState::Idle => {
                if self.state.hover_port.is_some() {
                    Interaction::Crosshair
                } else if self.state.hover_node.is_some() {
                    Interaction::Grab
                } else {
                    Interaction::default()
                }
            }
        }
    }
}

fn draw_bezier_wire(
    frame: &mut Frame,
    src: Point,
    dst: Point,
    color: Color,
    zoom: f32,
    width_mult: f32,
) {
    let ctrl_offset = ((dst.x - src.x).abs() * 0.5).max(60.0 * zoom);
    let c1 = Point::new(src.x + ctrl_offset, src.y);
    let c2 = Point::new(dst.x - ctrl_offset, dst.y);

    let path = Path::new(|b| {
        b.move_to(src);
        b.bezier_curve_to(c1, c2, dst);
    });
    let w = 2.0 * zoom.sqrt().max(0.8) * width_mult;

    frame.stroke(
        &path,
        Stroke::default()
            .with_color(Color { a: 0.15, ..color })
            .with_width(w * 3.5),
    );
    frame.stroke(&path, Stroke::default().with_color(color).with_width(w));
}

fn draw_node(frame: &mut Frame, node: &Node, state: &CanvasState, vp: Size, margin: f32) {
    let zoom = state.zoom;
    let pos_s = state.world_to_screen(node.pos());
    let w = NODE_WIDTH * zoom;
    let h = node.height() * zoom;

    if pos_s.x + w < -margin
        || pos_s.x > vp.width + margin
        || pos_s.y + h < -margin
        || pos_s.y > vp.height + margin
    {
        return;
    }

    let header_h = NODE_HEADER_HEIGHT * zoom;

    if w * h < 16.0 {
        frame.fill(
            &Path::circle(Point::new(pos_s.x + w * 0.5, pos_s.y + h * 0.5), 2.0),
            node.kind.header_color(),
        );
        return;
    }
    if w * h < 400.0 {
        let corner = (6.0 * zoom).min(w * 0.5);
        let body = Path::new(|b| rounded_rect(b, pos_s.x, pos_s.y, w, h, corner));
        frame.fill(&body, THEME.node_bg);
        frame.fill(
            &Path::new(|b| rounded_rect_top(b, pos_s.x, pos_s.y, w, header_h.min(h), corner)),
            node.kind.header_color(),
        );
        if node.selected {
            frame.stroke(
                &body,
                Stroke::default()
                    .with_color(Color {
                        a: 1.0,
                        ..THEME.accent
                    })
                    .with_width(1.5),
            );
        }
        return;
    }

    let preview_h = NODE_PREVIEW_HEIGHT * zoom;
    let corner = 6.0 * zoom;

    if zoom > 0.3 {
        frame.fill(
            &Path::new(|b| rounded_rect(b, pos_s.x + 3.0, pos_s.y + 4.0, w, h, corner)),
            Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.40,
            },
        );
    }

    let body = Path::new(|b| rounded_rect(b, pos_s.x, pos_s.y, w, h, corner));
    frame.fill(&body, THEME.node_bg);

    frame.fill(
        &Path::new(|b| rounded_rect_top(b, pos_s.x, pos_s.y, w, header_h, corner)),
        node.kind.header_color(),
    );

    let px = pos_s.x + 6.0 * zoom;
    let py = pos_s.y + header_h + 6.0 * zoom;
    let pw = w - 12.0 * zoom;
    let ph = preview_h - 12.0 * zoom;

    if w > 90.0 && pw > 4.0 && ph > 4.0 {
        let preview_path = Path::new(|b| rounded_rect(b, px, py, pw, ph, 3.0 * zoom));
        frame.fill(
            &preview_path,
            Color {
                r: 0.08,
                g: 0.08,
                b: 0.10,
                a: 1.0,
            },
        );
        draw_node_preview(frame, node, px, py, pw, ph, zoom);
        frame.stroke(
            &preview_path,
            Stroke::default()
                .with_color(Color {
                    r: 0.22,
                    g: 0.22,
                    b: 0.28,
                    a: 1.0,
                })
                .with_width(0.5 * zoom.max(1.0)),
        );
    }

    let (outline_color, outline_w) = if node.selected {
        (
            Color {
                a: 1.0,
                ..THEME.accent
            },
            2.0 * zoom,
        )
    } else {
        (
            Color {
                a: 0.3,
                ..THEME.node_border
            },
            zoom.max(0.5),
        )
    };
    frame.stroke(
        &body,
        Stroke::default()
            .with_color(outline_color)
            .with_width(outline_w),
    );

    if w > 40.0 {
        let title_size = 13.0 * zoom;
        let text_height_est = title_size * 1.2;
        let text_y = pos_s.y + (header_h - text_height_est) * 0.5;
        frame.fill_text(CanvasText {
            content: node.kind.label().to_string(),
            position: Point::new(pos_s.x + 10.0 * zoom, text_y),
            color: Color::WHITE,
            size: iced::Pixels(title_size),
            ..CanvasText::default()
        });
    }

    if zoom > 0.25 {
        let port_label_size = 11.0 * zoom;
        for (i, port) in node.inputs.iter().enumerate() {
            let c = state.world_to_screen(node.input_port_pos(i));
            if c.x < -margin || c.x > vp.width + margin || c.y < -margin || c.y > vp.height + margin
            {
                continue;
            }
            draw_port(frame, c, port.port_type.color(), zoom);
            let text_y = c.y - port_label_size * 0.6;
            if w > 80.0 {
                frame.fill_text(CanvasText {
                    content: port.label.clone(),
                    position: Point::new(c.x + 8.0 * zoom, text_y),
                    color: THEME.port_label,
                    size: iced::Pixels(port_label_size),
                    ..CanvasText::default()
                });
            }
        }
        for (i, port) in node.outputs.iter().enumerate() {
            let c = state.world_to_screen(node.output_port_pos(i));
            if c.x < -margin || c.x > vp.width + margin || c.y < -margin || c.y > vp.height + margin
            {
                continue;
            }
            draw_port(frame, c, port.port_type.color(), zoom);
            if w > 80.0 {
                let text_y = c.y - port_label_size * 0.6;
                let label_w = port.label.len() as f32 * port_label_size * 0.55;
                frame.fill_text(CanvasText {
                    content: port.label.clone(),
                    position: Point::new(c.x - 8.0 * zoom - label_w, text_y),
                    color: THEME.port_label,
                    size: iced::Pixels(port_label_size),
                    ..CanvasText::default()
                });
            }
        }
    }
}

fn draw_node_preview(frame: &mut Frame, node: &Node, x: f32, y: f32, w: f32, h: f32, zoom: f32) {
    match &node.kind {
        NodeKind::ColorConstant => {
            let col = node
                .outputs
                .first()
                .map(|p| p.port_type.color())
                .unwrap_or(Color::WHITE);
            let steps = 24u32;
            for i in 0..steps {
                let t = i as f32 / steps as f32;
                let bx = x + t * w;
                let bw = w / steps as f32 + 1.0;
                frame.fill_rectangle(
                    Point::new(bx, y),
                    Size::new(bw, h),
                    Color {
                        r: col.r * t,
                        g: col.g * t,
                        b: col.b * t,
                        a: 1.0,
                    },
                );
            }
        }
        NodeKind::TextureSample | NodeKind::NormalMap => {
            let cols = 8u32;
            let rows = 5u32;
            let cw = w / cols as f32;
            let ch = h / rows as f32;
            for row in 0..rows {
                for col in 0..cols {
                    let dark = (row + col) % 2 == 0;
                    let fill = if dark {
                        Color {
                            r: 0.15,
                            g: 0.15,
                            b: 0.18,
                            a: 1.0,
                        }
                    } else {
                        Color {
                            r: 0.30,
                            g: 0.30,
                            b: 0.35,
                            a: 1.0,
                        }
                    };
                    frame.fill_rectangle(
                        Point::new(x + col as f32 * cw, y + row as f32 * ch),
                        Size::new(cw + 0.5, ch + 0.5),
                        fill,
                    );
                }
            }
            if zoom > 0.5 {
                frame.fill_text(CanvasText {
                    content: "TEX".to_string(),
                    position: Point::new(x + w * 0.5 - 5.5, y + h * 0.5 - 5.5),
                    color: Color {
                        r: 0.6,
                        g: 0.6,
                        b: 0.7,
                        a: 0.5,
                    },
                    size: iced::Pixels(11.0),
                    ..CanvasText::default()
                });
            }
        }
        NodeKind::Time
        | NodeKind::Clamp
        | NodeKind::Power
        | NodeKind::Gamma
        | NodeKind::Fresnel => {
            let steps = 24u32;
            for i in 0..steps {
                let t = i as f32 / steps as f32;
                let bx = x + t * w;
                let bw = w / steps as f32 + 1.0;
                frame.fill_rectangle(
                    Point::new(bx, y),
                    Size::new(bw, h),
                    Color {
                        r: t,
                        g: t,
                        b: t,
                        a: 1.0,
                    },
                );
            }
        }
        NodeKind::Add | NodeKind::Multiply | NodeKind::Lerp | NodeKind::Mix => {
            let steps = (w as u32).max(2);
            let mid_y = y + h * 0.5;
            let amp = h * 0.38;
            let mut prev = Point::new(x, mid_y);
            for i in 1..=steps {
                let t = i as f32 / steps as f32;
                let sx = x + t * w;
                let sy = mid_y - (t * std::f32::consts::TAU * 1.5).sin() * amp;
                let cur = Point::new(sx, sy);
                frame.stroke(
                    &Path::line(prev, cur),
                    Stroke::default()
                        .with_color(Color {
                            r: 0.4,
                            g: 0.7,
                            b: 1.0,
                            a: 0.85,
                        })
                        .with_width(1.5 * zoom.sqrt()),
                );
                prev = cur;
            }
        }
        NodeKind::UVInput => {
            let cols = 24u32;
            let rows = 16u32;
            let cw = w / cols as f32;
            let ch = h / rows as f32;
            for row in 0..rows {
                for col in 0..cols {
                    let u = col as f32 / cols as f32;
                    let v = 1.0 - row as f32 / rows as f32;
                    frame.fill_rectangle(
                        Point::new(x + col as f32 * cw, y + row as f32 * ch),
                        Size::new(cw + 0.5, ch + 0.5),
                        Color {
                            r: u,
                            g: v,
                            b: 0.2,
                            a: 1.0,
                        },
                    );
                }
            }
        }
        NodeKind::VertexColor | NodeKind::HsvToRgb | NodeKind::RgbToHsv => {
            let steps = 24u32;
            for i in 0..steps {
                let t = i as f32 / steps as f32;
                let c = hue_to_rgb(t * 360.0);
                let bx = x + t * w;
                let bw = w / steps as f32 + 1.0;
                frame.fill_rectangle(Point::new(bx, y), Size::new(bw, h * 0.55), c);
                frame.fill_rectangle(
                    Point::new(bx, y + h * 0.55),
                    Size::new(bw, h * 0.45),
                    Color {
                        r: c.r * t,
                        g: c.g * t,
                        b: c.b * t,
                        a: 1.0,
                    },
                );
            }
        }
        NodeKind::CameraPos => {
            let mx = x + w * 0.5;
            let my = y + h * 0.5;
            let arm = h * 0.32;
            let col = Color {
                r: 0.5,
                g: 0.8,
                b: 1.0,
                a: 0.7,
            };
            let stroke = Stroke::default()
                .with_color(col)
                .with_width(1.5 * zoom.sqrt());
            frame.stroke(
                &Path::line(Point::new(mx - arm, my), Point::new(mx + arm, my)),
                stroke,
            );
            frame.stroke(
                &Path::line(Point::new(mx, my - arm), Point::new(mx, my + arm)),
                stroke,
            );
            frame.stroke(&Path::circle(Point::new(mx, my), arm * 0.45), stroke);
        }
        NodeKind::PBROutput | NodeKind::UnlitOutput => {
            let mx = x + w * 0.5;
            let my = y + h * 0.5;
            let r = h * 0.30;
            let col = Color {
                r: 0.8,
                g: 0.4,
                b: 0.4,
                a: 0.7,
            };
            frame.stroke(
                &Path::circle(Point::new(mx, my), r),
                Stroke::default()
                    .with_color(col)
                    .with_width(1.5 * zoom.sqrt()),
            );
            frame.stroke(
                &Path::circle(Point::new(mx, my), r * 0.55),
                Stroke::default()
                    .with_color(col)
                    .with_width(1.0 * zoom.sqrt()),
            );
            frame.fill(&Path::circle(Point::new(mx, my), r * 0.2), col);
        }
    }
}

fn hue_to_rgb(hue: f32) -> Color {
    let h = hue / 60.0;
    let i = h.floor() as u32 % 6;
    let f = h - h.floor();
    let q = 1.0 - f;
    let (r, g, b) = match i {
        0 => (1.0, f, 0.0),
        1 => (q, 1.0, 0.0),
        2 => (0.0, 1.0, f),
        3 => (0.0, q, 1.0),
        4 => (f, 0.0, 1.0),
        _ => (1.0, 0.0, q),
    };
    Color { r, g, b, a: 1.0 }
}

fn draw_port(frame: &mut Frame, center: Point, color: Color, zoom: f32) {
    let r = PORT_RADIUS * zoom;
    let fill = Color { a: 0.3, ..color };
    let ring = Color { a: 0.7, ..color };

    frame.fill(&Path::circle(center, (r - 1.5 * zoom).max(0.5)), fill);
    frame.stroke(
        &Path::circle(center, r),
        Stroke::default().with_color(ring).with_width(1.5 * zoom),
    );
}

fn draw_port_highlight(frame: &mut Frame, center: Point, zoom: f32) {
    let r = (PORT_RADIUS + 4.0) * zoom;
    let path = Path::circle(center, r);
    frame.fill(
        &path,
        Color {
            a: 0.2,
            ..THEME.accent
        },
    );
    frame.stroke(
        &path,
        Stroke::default()
            .with_color(THEME.accent)
            .with_width(2.0 * zoom),
    );
}

fn rounded_rect(b: &mut canvas::path::Builder, x: f32, y: f32, w: f32, h: f32, r: f32) {
    b.move_to(Point::new(x + r, y));
    b.line_to(Point::new(x + w - r, y));
    b.arc_to(Point::new(x + w, y), Point::new(x + w, y + r), r);
    b.line_to(Point::new(x + w, y + h - r));
    b.arc_to(Point::new(x + w, y + h), Point::new(x + w - r, y + h), r);
    b.line_to(Point::new(x + r, y + h));
    b.arc_to(Point::new(x, y + h), Point::new(x, y + h - r), r);
    b.line_to(Point::new(x, y + r));
    b.arc_to(Point::new(x, y), Point::new(x + r, y), r);
    b.close();
}

fn rounded_rect_top(b: &mut canvas::path::Builder, x: f32, y: f32, w: f32, h: f32, r: f32) {
    b.move_to(Point::new(x + r, y));
    b.line_to(Point::new(x + w - r, y));
    b.arc_to(Point::new(x + w, y), Point::new(x + w, y + r), r);
    b.line_to(Point::new(x + w, y + h));
    b.line_to(Point::new(x, y + h));
    b.line_to(Point::new(x, y + r));
    b.arc_to(Point::new(x, y), Point::new(x + r, y), r);
    b.close();
}

fn rect_from_two(a: Point, b: Point) -> Rectangle {
    let x = a.x.min(b.x);
    let y = a.y.min(b.y);
    Rectangle::new(
        Point::new(x, y),
        Size::new((a.x - b.x).abs(), (a.y - b.y).abs()),
    )
}

// ============================================================
// MINIMAPA
// ============================================================
pub struct MinimapCanvas<'a> {
    pub graph: &'a Graph,
    pub viewport: Rectangle,
    pub bounds: Rectangle,
}

impl<'a> MinimapCanvas<'a> {
    pub fn new(graph: &'a Graph, viewport: Rectangle) -> Self {
        let bounds = Self::compute_graph_bounds(graph);
        Self {
            graph,
            viewport,
            bounds,
        }
    }

    pub fn compute_graph_bounds(graph: &Graph) -> Rectangle {
        if graph.nodes.is_empty() {
            return Rectangle::new(Point::ORIGIN, Size::new(100.0, 100.0));
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for node in graph.nodes.values() {
            let b = node.bounds();
            min_x = min_x.min(b.x);
            min_y = min_y.min(b.y);
            max_x = max_x.max(b.x + b.width);
            max_y = max_y.max(b.y + b.height);
        }

        let pad = 50.0;
        Rectangle::new(
            Point::new(min_x - pad, min_y - pad),
            Size::new(max_x - min_x + 2.0 * pad, max_y - min_y + 2.0 * pad),
        )
    }
}

impl<'a> Program<Message> for MinimapCanvas<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        frame.fill_rectangle(Point::ORIGIN, bounds.size(), BG_SECO); // Użyto BG_SECO dla minimapy

        let scale_x = bounds.width / self.bounds.width;
        let scale_y = bounds.height / self.bounds.height;
        let scale = scale_x.min(scale_y);
        let offset_x = (bounds.width - self.bounds.width * scale) * 0.5;
        let offset_y = (bounds.height - self.bounds.height * scale) * 0.5;

        let world_to_minimap = |p: Point| -> Point {
            Point::new(
                (p.x - self.bounds.x) * scale + offset_x,
                (p.y - self.bounds.y) * scale + offset_y,
            )
        };

        for conn in &self.graph.connections {
            if let (Some(src), Some(dst)) = (
                self.graph.nodes.get(&conn.src_node),
                self.graph.nodes.get(&conn.dst_node),
            ) {
                let src_pos = world_to_minimap(src.output_port_pos(conn.src_port));
                let dst_pos = world_to_minimap(dst.input_port_pos(conn.dst_port));
                frame.stroke(
                    &Path::line(src_pos, dst_pos),
                    Stroke::default()
                        .with_color(Color::from_rgb(0.5, 0.5, 0.6))
                        .with_width(0.8),
                );
            }
        }

        for node in self.graph.nodes.values() {
            let node_bounds = node.bounds();
            let pos = world_to_minimap(node_bounds.position());
            let size = Size::new(node_bounds.width * scale, node_bounds.height * scale);

            if size.width < 0.5 || size.height < 0.5 {
                continue;
            }

            let rect = Path::rectangle(pos, size);
            frame.fill(&rect, node.kind.header_color());

            if node.selected {
                frame.stroke(
                    &rect,
                    Stroke::default().with_color(THEME.accent).with_width(1.2),
                );
            }
        }

        let viewport_pos = world_to_minimap(self.viewport.position());
        let viewport_size = Size::new(self.viewport.width * scale, self.viewport.height * scale);
        let viewport_rect = Path::rectangle(viewport_pos, viewport_size);
        frame.stroke(
            &viewport_rect,
            Stroke::default().with_color(Color::WHITE).with_width(1.5),
        );

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<canvas::Action<Message>> {
        if let Event::Mouse(mouse_event) = event {
            if let Some(position) = cursor.position_in(bounds) {
                match mouse_event {
                    mouse::Event::ButtonPressed(mouse::Button::Left) => {
                        let scale_x = bounds.width / self.bounds.width;
                        let scale_y = bounds.height / self.bounds.height;
                        let scale = scale_x.min(scale_y);
                        let offset_x = (bounds.width - self.bounds.width * scale) * 0.5;
                        let offset_y = (bounds.height - self.bounds.height * scale) * 0.5;

                        let world_x = self.bounds.x + (position.x - offset_x) / scale;
                        let world_y = self.bounds.y + (position.y - offset_y) / scale;

                        return Some(canvas::Action::publish(Message::MinimapJump(Point::new(
                            world_x, world_y,
                        ))));
                    }
                    _ => {}
                }
            }
        }
        None
    }
}

// ============================================================
// MENU SZYBKIEGO POŁĄCZENIA
// ============================================================
#[derive(Debug, Clone)]
pub struct QuickConnectMenu {
    pub pending: PendingConnection,
    pub screen_pos: Point,
    pub src_type: PortType,
    pub world_pos: Point,
    pub candidates: Vec<(NodeKind, usize)>,
}

fn build_quick_connect_candidates(src_type: &PortType) -> Vec<(NodeKind, usize)> {
    let all: &[NodeKind] = &[
        NodeKind::Add,
        NodeKind::Multiply,
        NodeKind::Lerp,
        NodeKind::Clamp,
        NodeKind::Power,
        NodeKind::Fresnel,
        NodeKind::Mix,
        NodeKind::Gamma,
        NodeKind::TextureSample,
        NodeKind::NormalMap,
        NodeKind::HsvToRgb,
        NodeKind::RgbToHsv,
        NodeKind::PBROutput,
        NodeKind::UnlitOutput,
    ];

    let mut result = Vec::new();
    for kind in all {
        let (inputs, _) = kind.ports();
        for (idx, port) in inputs.iter().enumerate() {
            if src_type.compatible_with(&port.port_type) {
                result.push((kind.clone(), idx));
                break;
            }
        }
    }
    result
}

// ============================================================
// APLIKACJA GŁÓWNA
// ============================================================
#[derive(Debug, Clone)]
pub enum PanelType {
    NodeGraph,
    Preview3D,
    Preview2D,
}

#[derive(Debug)]
pub struct Pane {
    pub panel_type: PanelType,
}

impl Pane {
    pub fn new(panel_type: PanelType) -> Self {
        Self { panel_type }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    PaneResized(pane_grid::ResizeEvent),
    CanvasEvent(CanvasMsg),
    AddNode(NodeKind, Point),
    DeleteSelected,
    DuplicateSelected,
    SelectAll,
    ToggleNodePicker,
    SearchChanged(String),
    FitAll,
    AutoLayout,
    ResetZoom,
    SaveGraph,
    NewGraph,
    Undo,
    Redo,
    QuickConnectPick(NodeKind),
    QuickConnectCancel,
    KeyPressed(keyboard::Key),
    MinimapJump(Point),
    ToggleMinimap,
    ResizeSplitterPressed(ResizeSide), // <-- bez pozycji, pobieramy z last_mouse_pos
    ResizeSplitterMoved(f32),
    ResizeSplitterReleased,
}

#[derive(Debug, Clone)]
pub enum CanvasMsg {
    MouseMoved(Point),
    LeftPressed(Point),
    LeftReleased(Point),
    RightPressed(Point),
    RightReleased(Point),
    MiddlePressed(Point),
    MiddleReleased(Point),
    Scrolled(Point, mouse::ScrollDelta),
}

#[derive(Debug)]
pub struct OnyxApp {
    pub panes: pane_grid::State<Pane>,
    pub graph: Graph,
    pub canvas_state: CanvasState,
    pub pending: Option<PendingConnection>,
    pub caches: CanvasCaches,
    pub show_node_picker: bool,
    pub search_query: String,
    pub canvas_bounds: Rectangle,
    pub status: String,
    pub quick_connect: Option<QuickConnectMenu>,
    pub ctrl_held: bool,
    pub shift_held: bool,
    pub show_minimap: bool,
    pub left_panel_width: f32,
    pub right_panel_width: f32,
    pub resize_state: Option<ResizeState>,
    pub last_mouse_pos: Point, // <-- NOWE: śledzenie pozycji myszy
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResizeSide {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct ResizeState {
    pub side: ResizeSide,
    pub start_mouse_x: f32,
    pub start_width: f32,
}

impl Default for OnyxApp {
    fn default() -> Self {
        Self::new()
    }
}

impl OnyxApp {
    pub fn new() -> Self {
        let (mut panes, node_pane) = pane_grid::State::new(Pane::new(PanelType::NodeGraph));

        let (preview_3d_pane, _) = panes
            .split(
                pane_grid::Axis::Horizontal,
                node_pane,
                Pane::new(PanelType::Preview3D),
            )
            .expect("Horizontal split");

        let _ = panes.split(
            pane_grid::Axis::Vertical,
            preview_3d_pane,
            Pane::new(PanelType::Preview2D),
        );

        let mut graph = Graph::new();

        let uv = graph.add_node(NodeKind::UVInput, Point::new(60.0, 200.0));
        let tex = graph.add_node(NodeKind::TextureSample, Point::new(280.0, 180.0));
        let gam = graph.add_node(NodeKind::Gamma, Point::new(510.0, 160.0));
        let fres = graph.add_node(NodeKind::Fresnel, Point::new(280.0, 400.0));
        let mix = graph.add_node(NodeKind::Mix, Point::new(730.0, 250.0));
        let pbr = graph.add_node(NodeKind::PBROutput, Point::new(960.0, 160.0));

        graph.add_connection(uv, 0, tex, 0);
        graph.add_connection(tex, 1, gam, 0);
        graph.add_connection(gam, 0, mix, 0);
        graph.add_connection(fres, 0, mix, 2);
        graph.add_connection(mix, 0, pbr, 0);

        let mut cs = CanvasState::new();
        cs.pan = Vector::new(20.0, 20.0);

        Self {
            panes,
            graph,
            canvas_state: cs,
            pending: None,
            caches: CanvasCaches::new(),
            show_node_picker: false,
            search_query: String::new(),
            canvas_bounds: Rectangle::new(Point::ORIGIN, Size::new(1200.0, 900.0)),
            status: "Gotowy. Kliknij PPM na płótnie aby dodać węzeł.".into(),
            quick_connect: None,
            ctrl_held: false,
            shift_held: false,
            show_minimap: true,
            left_panel_width: 250.0,
            right_panel_width: 300.0,
            resize_state: None,
            last_mouse_pos: Point::ORIGIN, // <-- inicjalizacja
        }
    }

    pub fn title(&self) -> String {
        "Onyx Node Editor".to_string()
    }

    pub fn theme(&self) -> Theme {
        Theme::custom(
            "onyx",
            iced::theme::palette::Seed {
                background: Color::from_rgb(0.11, 0.11, 0.13),
                text: Color::from_rgb(0.92, 0.92, 0.94),
                primary: Color::from_rgb(0.40, 0.70, 1.00),
                success: Color::from_rgb(0.30, 0.80, 0.50),
                warning: Color::from_rgb(0.90, 0.75, 0.30),
                danger: Color::from_rgb(0.90, 0.35, 0.35),
            },
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        event::listen().map(|e| match e {
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => Message::KeyPressed(key),
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                // Aktualizujemy pozycję myszy i wysyłamy komunikat o ruchu
                // (używamy Message::ResizeSplitterMoved tylko jeśli jesteśmy w trakcie przeciągania,
                // ale wysyłamy zawsze, a w update sprawdzamy stan)
                Message::ResizeSplitterMoved(position.x)
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                Message::ResizeSplitterReleased
            }
            _ => Message::KeyPressed(keyboard::Key::Named(keyboard::key::Named::F35)), // ignorowane
        })
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PaneResized(event) => {
                self.panes.resize(event.split, event.ratio);
            }
            Message::KeyPressed(key) => self.handle_key(key),
            Message::CanvasEvent(ev) => self.handle_canvas_event(ev),
            Message::AddNode(kind, world_pos) => {
                let id = self.graph.add_node(kind.clone(), world_pos);
                self.graph.deselect_all();
                if let Some(n) = self.graph.nodes.get_mut(&id) {
                    n.selected = true;
                }
                self.caches.invalidate_nodes();
                self.status = format!("Dodano węzeł: {}", kind.label());
                self.show_node_picker = false;
            }
            Message::DeleteSelected => {
                let selected = self.graph.selected_ids();
                let count = selected.len();
                for id in selected {
                    self.graph.remove_node(id);
                }
                self.caches.invalidate_nodes();
                self.status = format!("Usunięto {} węzł(ów).", count);
            }
            Message::DuplicateSelected => {
                let selected = self.graph.selected_ids();
                self.graph.deselect_all();
                for id in selected {
                    if let Some(new_id) = self.graph.duplicate_node(id) {
                        if let Some(n) = self.graph.nodes.get_mut(&new_id) {
                            n.selected = true;
                        }
                    }
                }
                self.caches.invalidate_nodes();
                self.status = "Zduplikowano zaznaczenie.".into();
            }
            Message::SelectAll => {
                for n in self.graph.nodes.values_mut() {
                    n.selected = true;
                }
                self.caches.invalidate_nodes();
            }
            Message::ToggleNodePicker => {
                self.show_node_picker = !self.show_node_picker;
                self.search_query.clear();
            }
            Message::SearchChanged(q) => {
                self.search_query = q;
            }
            Message::MinimapJump(world_pos) => {
                let view_center = Point::new(
                    self.canvas_bounds.width * 0.5,
                    self.canvas_bounds.height * 0.5,
                );
                let current_center = self.canvas_state.screen_to_world(view_center);
                let delta = Vector::new(
                    world_pos.x - current_center.x,
                    world_pos.y - current_center.y,
                );
                self.canvas_state.pan.x += delta.x * self.canvas_state.zoom;
                self.canvas_state.pan.y += delta.y * self.canvas_state.zoom;
                self.caches.invalidate_view();
                self.status = format!("Skok do ({:.0}, {:.0})", world_pos.x, world_pos.y);
            }
            Message::ToggleMinimap => {
                self.show_minimap = !self.show_minimap;
            }
            Message::FitAll => self.fit_all_nodes(),
            Message::AutoLayout => {
                let selected = !self.graph.selected_ids().is_empty();
                self.graph.auto_layout(selected);
                self.fit_all_nodes();
                self.status = if selected {
                    "Auto-layout zastosowany do zaznaczonych węzłów.".into()
                } else {
                    "Auto-layout zastosowany do całego grafu.".into()
                };
            }
            Message::QuickConnectPick(kind) => {
                if let Some(qc) = self.quick_connect.take() {
                    let port_idx = qc
                        .candidates
                        .iter()
                        .find(|(k, _)| *k == kind)
                        .map(|(_, idx)| *idx)
                        .unwrap_or(0);
                    let new_id = self.graph.add_node(kind, qc.world_pos);
                    let ok = self.graph.add_connection(
                        qc.pending.src_node,
                        qc.pending.src_port,
                        new_id,
                        port_idx,
                    );
                    self.status = if ok {
                        "Dodano i połączono węzeł.".into()
                    } else {
                        "Dodano węzeł (połączenie niekompatybilne).".into()
                    };
                    self.caches.invalidate_nodes();
                }
            }
            Message::QuickConnectCancel => {
                self.quick_connect = None;
            }
            Message::ResetZoom => {
                self.canvas_state.zoom = 1.0;
                self.caches.invalidate_view();
                self.status = "Zoom zresetowany do 100%.".into();
            }
            Message::SaveGraph => {
                let json = self.graph.to_json();
                self.status = format!("Graf zapisany ({} znaków).", json.len());
            }
            Message::NewGraph => {
                self.graph = Graph::new();
                self.canvas_state = CanvasState::new();
                self.caches.invalidate_view();
                self.status = "Nowy graf.".into();
            }
            Message::Undo | Message::Redo => {
                self.status = "Undo/Redo jeszcze niezaimplementowane.".into();
            }

            Message::ResizeSplitterPressed(side) => {
                // Używamy ostatniej znanej pozycji myszy jako punktu startowego
                let mouse_x = self.last_mouse_pos.x;
                let start_width = match side {
                    ResizeSide::Left => self.left_panel_width,
                    ResizeSide::Right => self.right_panel_width,
                };
                self.resize_state = Some(ResizeState {
                    side,
                    start_mouse_x: mouse_x,
                    start_width,
                });
            }

            Message::ResizeSplitterMoved(mouse_x) => {
                // Aktualizujemy globalną pozycję myszy
                self.last_mouse_pos.x = mouse_x;

                if let Some(state) = &self.resize_state {
                    let delta = mouse_x - state.start_mouse_x;
                    let new_width = match state.side {
                        ResizeSide::Left => (state.start_width + delta).max(150.0).min(500.0),
                        // Dla prawego panelu: przesunięcie w lewo (ujemna delta) zwiększa szerokość
                        ResizeSide::Right => (state.start_width - delta).max(200.0).min(600.0),
                    };
                    match state.side {
                        ResizeSide::Left => self.left_panel_width = new_width,
                        ResizeSide::Right => self.right_panel_width = new_width,
                    }
                }
            }

            Message::ResizeSplitterReleased => {
                self.resize_state = None;
            }
        }
        Task::none()
    }

    fn handle_key(&mut self, key: keyboard::Key) {
        use keyboard::Key::Named;
        use keyboard::key::Named::*;

        match key {
            Named(Delete) | Named(Backspace) => {
                let _ = self.update(Message::DeleteSelected);
            }
            Named(Escape) => {
                self.show_node_picker = false;
                self.quick_connect = None;
                self.graph.deselect_all();
                self.pending = None;
                self.canvas_state.interaction = InteractionState::Idle;
            }
            keyboard::Key::Character(c) if c.as_str() == "d" => {
                let _ = self.update(Message::DuplicateSelected);
            }
            keyboard::Key::Character(c) if c.as_str() == "f" => {
                self.fit_all_nodes();
            }
            keyboard::Key::Character(c) if c.as_str() == "a" => {
                let _ = self.update(Message::SelectAll);
            }
            keyboard::Key::Character(c) if c.as_str() == " " => {
                self.show_node_picker = !self.show_node_picker;
            }
            _ => {}
        }
    }

    fn handle_canvas_event(&mut self, ev: CanvasMsg) {
        match ev {
            CanvasMsg::LeftPressed(screen_pos) => {
                let world = self.canvas_state.screen_to_world(screen_pos);
                self.handle_left_press(world, screen_pos);
            }
            CanvasMsg::LeftReleased(screen_pos) => {
                let world = self.canvas_state.screen_to_world(screen_pos);
                self.handle_left_release(world);
            }
            CanvasMsg::RightPressed(screen_pos) => {
                let world = self.canvas_state.screen_to_world(screen_pos);

                let wire_threshold = 6.0 / self.canvas_state.zoom;
                if let Some(conn_id) = self.graph.connection_at_point(world, wire_threshold) {
                    self.graph.remove_connection(conn_id);
                    self.caches.invalidate_nodes();
                    self.status = "Połączenie usunięte.".into();
                    return;
                }

                let hit = self.hit_test_node(world);
                if hit.is_none() {
                    self.show_node_picker = !self.show_node_picker;
                    self.search_query.clear();
                }
            }
            CanvasMsg::RightReleased(_) => {
                // można zignorować
            }
            CanvasMsg::MiddlePressed(screen_pos) => {
                self.canvas_state.interaction = InteractionState::Panning { last: screen_pos };
            }
            CanvasMsg::MiddleReleased(_) => {
                if matches!(
                    self.canvas_state.interaction,
                    InteractionState::Panning { .. }
                ) {
                    self.canvas_state.interaction = InteractionState::Idle;
                }
            }
            CanvasMsg::MouseMoved(screen_pos) => {
                let world = self.canvas_state.screen_to_world(screen_pos);
                self.handle_mouse_move(world, screen_pos);
            }
            CanvasMsg::Scrolled(screen_pos, delta) => {
                self.handle_scroll(screen_pos, delta);
            }
        }
    }

    fn handle_left_press(&mut self, world: Point, _screen: Point) {
        if let Some((node_id, port_idx, src_pos)) = self.hit_test_output_port(world) {
            self.graph.bring_to_front(node_id);
            let pc = PendingConnection {
                src_node: node_id,
                src_port: port_idx,
                src_pos,
                current_pos: world,
            };
            self.canvas_state.interaction = InteractionState::DraggingWire(pc.clone());
            self.pending = Some(pc);
            self.caches.invalidate_nodes();
            return;
        }

        if let Some(node_id) = self.hit_test_node(world) {
            self.graph.bring_to_front(node_id);
            if !self.graph.nodes[&node_id].selected {
                self.graph.select_only(node_id);
            }
            let node_pos = self.graph.nodes[&node_id].pos();
            self.canvas_state.interaction = InteractionState::DraggingNode {
                id: node_id,
                offset: Vector::new(world.x - node_pos.x, world.y - node_pos.y),
                last_world: world,
            };
            self.caches.invalidate_nodes();
            return;
        }

        self.graph.deselect_all();
        self.caches.invalidate_nodes();
        self.canvas_state.interaction = InteractionState::Selecting {
            start: world,
            current: world,
        };
    }

    fn handle_left_release(&mut self, world: Point) {
        if self.quick_connect.is_some() {
            self.quick_connect = None;
            return;
        }

        let interaction =
            std::mem::replace(&mut self.canvas_state.interaction, InteractionState::Idle);

        match interaction {
            InteractionState::DraggingWire(pending) => {
                self.pending = None;
                if let Some((dst_node, dst_port)) = self.hit_test_input_port(world) {
                    let ok = self.graph.add_connection(
                        pending.src_node,
                        pending.src_port,
                        dst_node,
                        dst_port,
                    );
                    self.status = if ok {
                        "Połączono porty.".into()
                    } else {
                        "Niekompatybilne typy portów — połączenie odrzucone.".into()
                    };
                } else {
                    let src_type = self
                        .graph
                        .nodes
                        .get(&pending.src_node)
                        .and_then(|n| n.outputs.get(pending.src_port))
                        .map(|p| p.port_type.clone());

                    if let Some(src_type) = src_type {
                        let candidates = build_quick_connect_candidates(&src_type);
                        if !candidates.is_empty() {
                            let screen_pos = self.canvas_state.world_to_screen(world);
                            self.quick_connect = Some(QuickConnectMenu {
                                pending,
                                screen_pos,
                                src_type,
                                world_pos: world,
                                candidates,
                            });
                        }
                    }
                }
                self.caches.invalidate_nodes();
            }
            InteractionState::Selecting { start, current } => {
                let rect = selection_rect(start, current);
                if rect.width >= 4.0 || rect.height >= 4.0 {
                    self.graph.select_rect(rect);
                }
                self.caches.invalidate_nodes();
            }
            InteractionState::DraggingNode { id, .. } => {
                if let Some(wire_id) = self.canvas_state.hovered_wire.take() {
                    self.try_insert_node_on_wire(id, wire_id);
                }
                self.canvas_state.hovered_wire = None;
                self.caches.invalidate_nodes();
            }
            _ => {}
        }
    }

    fn handle_mouse_move(&mut self, world: Point, screen: Point) {
        self.canvas_state.hover_node = self.hit_test_node(world);
        self.canvas_state.hover_port = self.hit_test_any_port(world);

        match &mut self.canvas_state.interaction {
            InteractionState::DraggingNode { id, last_world, .. } => {
                let id = *id;
                let delta = Vector::new(world.x - last_world.x, world.y - last_world.y);
                *last_world = world;
                self.graph.move_selected(delta);

                let threshold = 8.0 / self.canvas_state.zoom;
                let node_center = self.graph.nodes.get(&id).map(|n: &Node| {
                    let b = n.bounds();
                    Point::new(b.x + b.width * 0.5, b.y + b.height * 0.5)
                });
                self.canvas_state.hovered_wire = node_center
                    .and_then(|c| self.graph.connection_at_point_excluding(c, threshold, id));

                self.caches.invalidate_nodes();
            }
            InteractionState::DraggingWire(pending) => {
                let snap_r = 20.0 / self.canvas_state.zoom;
                let snapped = self
                    .graph
                    .nodes_ordered()
                    .flat_map(|n| {
                        n.inputs
                            .iter()
                            .enumerate()
                            .map(move |(i, _)| (n.input_port_pos(i), n.id, i))
                    })
                    .filter(|(p, _, _)| {
                        (p.x - world.x).powi(2) + (p.y - world.y).powi(2) <= snap_r * snap_r
                    })
                    .min_by(|(a, _, _), (b, _, _)| {
                        let da = (a.x - world.x).powi(2) + (a.y - world.y).powi(2);
                        let db = (b.x - world.x).powi(2) + (b.y - world.y).powi(2);
                        da.partial_cmp(&db).unwrap()
                    })
                    .map(|(p, _, _)| p);
                pending.current_pos = snapped.unwrap_or(world);
                self.pending = Some(pending.clone());
            }
            InteractionState::Selecting { current, .. } => {
                *current = world;
            }
            InteractionState::Panning { last } => {
                let delta = Vector::new(screen.x - last.x, screen.y - last.y);
                self.canvas_state.pan.x += delta.x;
                self.canvas_state.pan.y += delta.y;
                *last = screen;
                self.caches.invalidate_view();
            }
            InteractionState::Idle => {}
        }
    }

    fn handle_scroll(&mut self, screen: Point, delta: mouse::ScrollDelta) {
        let scroll_y = match delta {
            mouse::ScrollDelta::Lines { y, .. } => y,
            mouse::ScrollDelta::Pixels { y, .. } => y / 50.0,
        };
        let factor = if scroll_y > 0.0 { 1.1 } else { 1.0 / 1.1 };
        let new_zoom = (self.canvas_state.zoom * factor).clamp(0.1, 4.0);
        let world_before = self.canvas_state.screen_to_world(screen);
        self.canvas_state.zoom = new_zoom;
        let world_after = self.canvas_state.screen_to_world(screen);
        self.canvas_state.pan.x += (world_after.x - world_before.x) * new_zoom;
        self.canvas_state.pan.y += (world_after.y - world_before.y) * new_zoom;
        self.caches.invalidate_view();
    }

    fn hit_test_node(&self, world: Point) -> Option<Uuid> {
        for node in self
            .graph
            .nodes_ordered()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            if node.bounds().contains(world) {
                return Some(node.id);
            }
        }
        None
    }

    fn hit_test_output_port(&self, world: Point) -> Option<(Uuid, usize, Point)> {
        let zoom = self.canvas_state.zoom;
        for node in self
            .graph
            .nodes_ordered()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            if let Some(i) = node.hit_test_output_port_zoomed(world, zoom) {
                return Some((node.id, i, node.output_port_pos(i)));
            }
        }
        None
    }

    fn hit_test_input_port(&self, world: Point) -> Option<(Uuid, usize)> {
        let zoom = self.canvas_state.zoom;
        for node in self
            .graph
            .nodes_ordered()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            if let Some(i) = node.hit_test_input_port_zoomed(world, zoom) {
                return Some((node.id, i));
            }
        }
        None
    }

    fn hit_test_any_port(&self, world: Point) -> Option<(Uuid, PortSide, usize)> {
        let zoom = self.canvas_state.zoom;
        for node in self
            .graph
            .nodes_ordered()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            if let Some(i) = node.hit_test_output_port_zoomed(world, zoom) {
                return Some((node.id, PortSide::Output, i));
            }
            if let Some(i) = node.hit_test_input_port_zoomed(world, zoom) {
                return Some((node.id, PortSide::Input, i));
            }
        }
        None
    }

    fn try_insert_node_on_wire(&mut self, node_id: Uuid, wire_id: Uuid) {
        let conn = match self.graph.get_connection(wire_id) {
            Some(c) => c.clone(),
            None => return,
        };

        let src_out_type = self
            .graph
            .nodes
            .get(&conn.src_node)
            .and_then(|n| n.outputs.get(conn.src_port))
            .map(|p| p.port_type.clone());
        let dst_in_type = self
            .graph
            .nodes
            .get(&conn.dst_node)
            .and_then(|n| n.inputs.get(conn.dst_port))
            .map(|p| p.port_type.clone());

        let (src_out_type, dst_in_type) = match (src_out_type, dst_in_type) {
            (Some(a), Some(b)) => (a, b),
            _ => return,
        };

        let node_in_port = self.graph.nodes.get(&node_id).and_then(|n| {
            n.inputs
                .iter()
                .enumerate()
                .find(|(_, p)| src_out_type.compatible_with(&p.port_type))
                .map(|(i, _)| i)
        });

        let node_out_port = self.graph.nodes.get(&node_id).and_then(|n| {
            n.outputs
                .iter()
                .enumerate()
                .find(|(_, p)| p.port_type.compatible_with(&dst_in_type))
                .map(|(i, _)| i)
        });

        match (node_in_port, node_out_port) {
            (Some(in_idx), Some(out_idx)) => {
                self.graph.remove_connection(wire_id);
                self.graph
                    .add_connection(conn.src_node, conn.src_port, node_id, in_idx);
                self.graph
                    .add_connection(node_id, out_idx, conn.dst_node, conn.dst_port);
                self.status = "Węzeł wstawiony w połączenie.".into();
            }
            _ => {
                self.status = "Węzeł niekompatybilny z tym połączeniem.".into();
            }
        }
    }

    fn fit_all_nodes(&mut self) {
        if self.graph.nodes.is_empty() {
            return;
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for node in self.graph.nodes.values() {
            let b = node.bounds();
            min_x = min_x.min(b.x);
            min_y = min_y.min(b.y);
            max_x = max_x.max(b.x + b.width);
            max_y = max_y.max(b.y + b.height);
        }

        let pad = 80.0;
        let content_w = max_x - min_x + pad * 2.0;
        let content_h = max_y - min_y + pad * 2.0;
        let view_w = self.canvas_bounds.width;
        let view_h = self.canvas_bounds.height;

        let zoom = (view_w / content_w).min(view_h / content_h).min(1.5);
        self.canvas_state.zoom = zoom;
        self.canvas_state.pan = Vector::new(
            (view_w - content_w * zoom) * 0.5 - (min_x - pad) * zoom,
            (view_h - content_h * zoom) * 0.5 - (min_y - pad) * zoom,
        );
        self.status = format!("Widok dopasowany — zoom {:.0}%.", zoom * 100.0);
        self.caches.invalidate_view();
    }

    pub fn view(&self) -> Element<'_, Message> {
        // Lewy panel
        let left_panel = self.view_left_panel();
        // Środkowy obszar z pane_grid
        let center = pane_grid(&self.panes, |_id, pane, _is_maximized| {
            pane_grid::Content::new(self.draw_panel(pane))
        })
        .on_resize(10, |event| Message::PaneResized(event))
        .width(Length::Fill)
        .height(Length::Fill);
        // Prawy panel
        let right_panel = self.view_right_panel();

        // Splittery
        let left_splitter = self.create_splitter(ResizeSide::Left);
        let right_splitter = self.create_splitter(ResizeSide::Right);

        // Główny wiersz z panelami i splitterami
        let main_row = row![
            container(left_panel).width(Length::Fixed(self.left_panel_width)),
            left_splitter,
            container(center).width(Length::Fill),
            right_splitter,
            container(right_panel).width(Length::Fixed(self.right_panel_width)),
        ]
        .height(Length::Fill);

        let status_bar = self.view_statusbar();

        column![main_row, status_bar].spacing(0).into()
    }

    fn view_left_panel(&self) -> Element<'_, Message> {
        // Przykład: używamy istniejącego sidebaru (biblioteka węzłów)
        // ale bez zależności od `show_node_picker` – zawsze widoczny.
        // Jeśli chcesz ukrywać panel, dodaj warunek.
        let query = self.search_query.to_lowercase();

        let search = text_input("Szukaj węzłów...", &self.search_query)
            .on_input(Message::SearchChanged)
            .padding([6, 10])
            .size(13);

        let mut items: Vec<Element<Message>> = vec![];

        for category in NodeCategory::all() {
            let nodes: Vec<_> = category
                .nodes()
                .into_iter()
                .filter(|k| query.is_empty() || k.label().to_lowercase().contains(&query))
                .collect();

            if nodes.is_empty() {
                continue;
            }

            let cat_color = category.color();
            let cat_header = container(text(category.label()).size(11).color(Color::WHITE))
                .padding([3, 8])
                .width(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(cat_color)),
                    border: iced::Border {
                        radius: 3.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                });

            items.push(cat_header.into());

            for kind in nodes {
                let label = kind.label();
                let world_pos = self.canvas_state.screen_to_world(Point::new(
                    self.canvas_bounds.width * 0.5 - NODE_WIDTH * 0.5,
                    self.canvas_bounds.height * 0.5,
                ));
                let kind_clone = kind.clone();
                let btn = button(text(label).size(12))
                    .padding([5, 12])
                    .width(Length::Fill)
                    .on_press(Message::AddNode(kind_clone, world_pos))
                    .style(button::secondary);

                items.push(btn.into());
            }

            items.push(Space::new().height(Length::Fixed(4.0)).into());
        }

        let content = scrollable(column(items).spacing(2).padding([4, 6])).height(Length::Fill);

        let panel = container(
            column![
                container(text("Węzły").size(14).color(THEME.text))
                    .padding([8, 0])
                    .width(Length::Fill),
                search,
                Space::new().height(Length::Fixed(6.0)),
                content,
            ]
            .spacing(0),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(BG_SECO)),
            border: iced::Border {
                color: SEPA_CO,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        });

        panel.into()
    }

    fn create_splitter(&self, side: ResizeSide) -> Element<'_, Message> {
        let is_active = matches!(&self.resize_state, Some(state) if state.side == side);
    
        // Ustaw preferowaną szerokość widocznego paska (np. 4.0)
        let visible_width = 1.5;
        // Obszar klikalny może być nieco szerszy dla wygody (np. 8.0)
        let hit_area_width = 2.;
    
        container(
            mouse_area(
                container(Space::new())
                    .width(Length::Fixed(visible_width))
                    .height(Length::Fill)
                    .style(move |_theme| {
                        let base_color = if is_active {
                            Color::from_rgb(0.7, 0.8, 1.0)
                        } else {
                            Color::TRANSPARENT
                        };
                        container::Style {
                            background: Some(iced::Background::Color(base_color)),
                            ..Default::default()
                        }
                    })
            )
            .on_press(Message::ResizeSplitterPressed(side.clone()))
            .on_release(Message::ResizeSplitterReleased)
            .interaction(iced::mouse::Interaction::ResizingHorizontally)
        )
        .width(Length::Fixed(hit_area_width)) // obszar łapania szerszy niż pasek
        .height(Length::Fill)
        .into()
    }

    fn view_right_panel(&self) -> Element<'_, Message> {
        // Inspektor właściwości – wyświetl informacje o zaznaczonych węzłach
        let selected_ids = self.graph.selected_ids();
        let content: Element<_> = if selected_ids.len() == 1 {
            let id = selected_ids[0];
            if let Some(node) = self.graph.nodes.get(&id) {
                // Przykładowe informacje o węźle
                column![
                    text(format!("Węzeł: {}", node.kind.label())).size(14),
                    text(format!("ID: {}", id)),
                    text(format!(
                        "Pozycja: ({:.1}, {:.1})",
                        node.pos().x,
                        node.pos().y
                    )),
                    text(format!("Wejścia: {}", node.inputs.len())),
                    text(format!("Wyjścia: {}", node.outputs.len())),
                ]
                .spacing(8)
                .padding(10)
                .into()
            } else {
                text("Brak danych").into()
            }
        } else if selected_ids.len() > 1 {
            text(format!("Zaznaczonych węzłów: {}", selected_ids.len())).into()
        } else {
            text("Nic nie zaznaczono").into()
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(BG_SECO)),
                border: iced::Border {
                    color: SEPA_CO,
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .into()
    }

    fn draw_panel(&self, pane: &Pane) -> Element<'_, Message> {
        let content: Element<'_, Message> = match pane.panel_type {
            PanelType::NodeGraph => self.view_node_graph_panel(),
            PanelType::Preview3D => container(button("Preview 3D placeholder")).into(),
            PanelType::Preview2D => container(text("Preview 2D placeholder")).into(),
        };
        self.panel_style(pane.panel_type.clone(), content)
    }

    fn panel_style<'a>(
        &self,
        variant: PanelType,
        content: Element<'a, Message>,
    ) -> Element<'a, Message> {
        let title = match variant {
            PanelType::NodeGraph => "Node Editor",
            PanelType::Preview3D => "Preview 3D",
            PanelType::Preview2D => "Preview 2D",
        };

        let header = container(
            text(title)
                .size(14)
                .font(Font {
                    family: Family::Name("Switzer"),
                    weight: iced::font::Weight::Medium,
                    ..Default::default()
                })
                .color(CT_PRIM),
        )
        .width(Length::Fill)
        .padding(Padding::default().left(22).bottom(6).top(6))
        .style(|_theme| container::Style {
            background: Some(BG_PRIM.into()),
            ..Default::default()
        });

        iced::widget::column![
            header,
            container(
                container(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(|_| container::Style {
                        background: Some(Background::Color(BG_SECO)),

                        border: Border {
                            width: 1.5,
                            color: SEPA_CO,
                            ..Default::default()
                        },
                        ..Default::default()
                    })
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(2)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(BG_PRIM)),
                ..Default::default()
            })
        ]
        .into()
    }

    fn view_node_graph_panel(&self) -> Element<'_, Message> {
        let toolbar = self.view_toolbar();
        let canvas = self.view_canvas();
        let sidebar = self.view_sidebar();

        let main_row = row![
            canvas,
            if self.show_node_picker {
                sidebar
            } else {
                Space::new().width(0).into()
            },
        ]
        .spacing(0)
        .height(Length::Fill);

        let canvas_layer: Element<_> = if let Some(qc) = &self.quick_connect {
            stack![main_row, self.view_quick_connect_menu(qc)].into()
        } else {
            main_row.into()
        };

        let viewport = Rectangle::new(
            self.canvas_state.screen_to_world(Point::ORIGIN),
            Size::new(
                self.canvas_bounds.width / self.canvas_state.zoom,
                self.canvas_bounds.height / self.canvas_state.zoom,
            ),
        );

        let minimap: Element<'_, Message> = if self.show_minimap {
            container(
                Canvas::new(MinimapCanvas {
                    graph: &self.graph,
                    viewport,
                    bounds: MinimapCanvas::compute_graph_bounds(&self.graph),
                })
                .width(Length::Fixed(200.0))
                .height(Length::Fixed(150.0)),
            )
            .padding(8)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(Color::from_rgba(
                    0.1, 0.1, 0.15, 0.85,
                ))),
                border: iced::Border {
                    color: Color::from_rgb(0.3, 0.3, 0.4),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..container::Style::default()
            })
            .align_x(iced::Alignment::End)
            .align_y(iced::Alignment::End)
            .into()
        } else {
            Space::new().into()
        };

        let canvas_with_minimap = stack![canvas_layer, minimap];

        // Zwracamy tylko toolbar i obszar roboczy – status przeniesiony do głównego widoku
        column![toolbar, canvas_with_minimap].spacing(0).into()
    }

    fn view_toolbar(&self) -> Element<'_, Message> {
        let btn = |label: &'static str, msg: Message| {
            button(text(label).size(12)).padding([4, 10]).on_press(msg)
        };

        let zoom_label = text(format!("{:.0}%", self.canvas_state.zoom * 100.0))
            .size(12)
            .color(Color::from_rgb(0.7, 0.7, 0.8));

        let separator = || {
            container(Space::new().width(1.0).height(20.0)).style(|_| container::Style {
                background: Some(iced::Background::Color(SEPA_CO)),
                ..Default::default()
            })
        };

        let toolbar = container(
            row![
                btn("Nowy", Message::NewGraph),
                btn("Zapisz JSON", Message::SaveGraph),
                Space::new().width(Length::Fixed(12.0)),
                separator(),
                Space::new().width(Length::Fixed(12.0)),
                btn("Cofnij", Message::Undo),
                btn("Ponów", Message::Redo),
                Space::new().width(Length::Fixed(12.0)),
                separator(),
                Space::new().width(Length::Fixed(12.0)),
                btn("Usuń [Del]", Message::DeleteSelected),
                btn("Duplikuj [D]", Message::DuplicateSelected),
                btn("Zaznacz wszystko [A]", Message::SelectAll),
                Space::new().width(Length::Fill),
                btn(
                    if self.show_minimap {
                        "◉ Mapa"
                    } else {
                        "○ Mapa"
                    },
                    Message::ToggleMinimap
                ),
                btn("Dopasuj [F]", Message::FitAll),
                btn("Auto-layout", Message::AutoLayout),
                btn("100%", Message::ResetZoom),
                zoom_label,
                Space::new().width(Length::Fixed(12.0)),
                btn(
                    if self.show_node_picker {
                        "✕ Węzły"
                    } else {
                        "+ Węzły [Space]"
                    },
                    Message::ToggleNodePicker
                ),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        )
        .padding([6, 10])
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(BG_PRIM)),
            border: iced::Border {
                color: SEPA_CO,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        });

        toolbar.into()
    }

    fn view_canvas(&self) -> Element<'_, Message> {
        Canvas::new(NodeCanvasProgram {
            graph: &self.graph,
            state: &self.canvas_state,
            pending: self.pending.as_ref(),
            caches: &self.caches,
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn view_sidebar(&self) -> Element<'_, Message> {
        let query = self.search_query.to_lowercase();

        let search = text_input("Szukaj węzłów...", &self.search_query)
            .on_input(Message::SearchChanged)
            .padding([6, 10])
            .size(13);

        let mut items: Vec<Element<Message>> = vec![];

        for category in NodeCategory::all() {
            let nodes: Vec<_> = category
                .nodes()
                .into_iter()
                .filter(|k| query.is_empty() || k.label().to_lowercase().contains(&query))
                .collect();

            if nodes.is_empty() {
                continue;
            }

            let cat_color = category.color();
            let cat_header = container(text(category.label()).size(11).color(Color::WHITE))
                .padding([3, 8])
                .width(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(cat_color)),
                    border: iced::Border {
                        radius: 3.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                });

            items.push(cat_header.into());

            for kind in nodes {
                let label = kind.label();
                let world_pos = self.canvas_state.screen_to_world(Point::new(
                    self.canvas_bounds.width * 0.5 - NODE_WIDTH * 0.5,
                    self.canvas_bounds.height * 0.5,
                ));
                let kind_clone = kind.clone();
                let btn = button(text(label).size(12))
                    .padding([5, 12])
                    .width(Length::Fill)
                    .on_press(Message::AddNode(kind_clone, world_pos))
                    .style(button::secondary);

                items.push(btn.into());
            }

            items.push(Space::new().height(Length::Fixed(4.0)).into());
        }

        let content = scrollable(column(items).spacing(2).padding([4, 6])).height(Length::Fill);

        let panel = container(
            column![
                container(text("  Węzły").size(13).color(THEME.text))
                    .padding([8, 0])
                    .width(Length::Fill),
                search,
                Space::new().height(Length::Fixed(6.0)),
                content,
            ]
            .spacing(0),
        )
        .width(Length::Fixed(210.0))
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(BG_SECO)),
            border: iced::Border {
                color: SEPA_CO,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        });

        panel.into()
    }

    fn view_statusbar(&self) -> Element<'_, Message> {
        let sel_count = self.graph.selected_ids().len();
        let node_count = self.graph.nodes.len();
        let conn_count = self.graph.connections.len();

        let info = format!(
            "Węzły: {}  |  Połączenia: {}  |  Zaznaczone: {}  |  Zoom: {:.0}%",
            node_count,
            conn_count,
            sel_count,
            self.canvas_state.zoom * 100.0,
        );

        container(
            row![
                text(&self.status)
                    .size(11)
                    .color(Color::from_rgb(0.7, 0.8, 0.7)),
                Space::new().width(Length::Fill),
                text(info).size(11).color(Color::from_rgb(0.5, 0.5, 0.6)),
            ]
            .align_y(Alignment::Center),
        )
        .padding([4, 10])
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(BG_PRIM)),
            border: iced::Border {
                color: SEPA_CO,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
    }

    fn view_quick_connect_menu(&self, qc: &QuickConnectMenu) -> Element<'_, Message> {
        let mx = qc.screen_pos.x + 8.0;
        let my = qc.screen_pos.y + 8.0;

        let port_color = qc.src_type.color();

        let header = container(
            row![
                Space::new().width(Length::Fixed(2.0)),
                text(format!("→  {:?}", qc.src_type))
                    .size(11)
                    .color(Color::from_rgb(port_color.r, port_color.g, port_color.b)),
            ]
            .align_y(Alignment::Center)
            .spacing(6),
        )
        .padding([6, 10])
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.16, 0.16, 0.20))),
            ..Default::default()
        });

        let mut items = column![header].spacing(1);

        for (kind, _port_idx) in &qc.candidates {
            let kind_clone = kind.clone();
            let cat_color = kind.category().color();
            let btn = button(
                row![
                    Space::new().width(Length::Fixed(4.0)),
                    container(Space::new().width(Length::Fixed(3.0)))
                        .height(Length::Fixed(14.0))
                        .style(move |_| container::Style {
                            background: Some(iced::Background::Color(cat_color)),
                            border: iced::Border {
                                radius: 2.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    Space::new().width(Length::Fixed(7.0)),
                    text(kind.label()).size(12),
                ]
                .align_y(Alignment::Center)
                .spacing(0),
            )
            .padding([5, 10])
            .width(Length::Fill)
            .on_press(Message::QuickConnectPick(kind_clone))
            .style(|_, status| button::Style {
                background: Some(iced::Background::Color(match status {
                    button::Status::Hovered | button::Status::Pressed => {
                        Color::from_rgb(0.25, 0.30, 0.40)
                    }
                    _ => Color::from_rgb(0.15, 0.15, 0.19),
                })),
                border: iced::Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
                text_color: Color::from_rgb(0.88, 0.88, 0.92),
                ..Default::default()
            });

            items = items.push(btn);
        }

        let cancel_btn = button(
            text("✕  Anuluj")
                .size(11)
                .color(Color::from_rgb(0.5, 0.5, 0.6)),
        )
        .padding([4, 10])
        .width(Length::Fill)
        .on_press(Message::QuickConnectCancel)
        .style(|_, _| button::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.13, 0.13, 0.16))),
            ..Default::default()
        });

        items = items.push(Space::new().height(Length::Fixed(2.0)));
        items = items.push(cancel_btn);

        let menu = container(items)
            .width(Length::Fixed(200.0))
            .style(|_| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.14, 0.14, 0.18))),
                border: iced::Border {
                    color: Color::from_rgb(0.30, 0.35, 0.50),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.6),
                    offset: iced::Vector::new(4.0, 6.0),
                    blur_radius: 16.0,
                },
                ..Default::default()
            });

        container(menu)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(iced::Padding {
                top: my.max(0.0),
                left: mx.max(0.0),
                right: 0.0,
                bottom: 0.0,
            })
            .into()
    }
}

// ============================================================
// POMOCNICZE
// ============================================================
fn selection_rect(a: Point, b: Point) -> Rectangle {
    let x = a.x.min(b.x);
    let y = a.y.min(b.y);
    Rectangle::new(
        Point::new(x, y),
        Size::new((a.x - b.x).abs(), (a.y - b.y).abs()),
    )
}

// ============================================================
// MAIN
// ============================================================
fn main() -> iced::Result {
    iced::application(OnyxApp::default, OnyxApp::update, OnyxApp::view)
        .subscription(OnyxApp::subscription)
        .theme(OnyxApp::theme)
        .settings(iced::Settings {
            antialiasing: true,
            ..iced::Settings::default()
        })
        .window(iced::window::Settings {
            size: iced::Size::new(1400.0, 900.0),
            min_size: Some(iced::Size::new(800.0, 600.0)),
            ..iced::window::Settings::default()
        })
        .font(include_bytes!("../assets/fonts/Switzer-Regular.otf").as_slice())
        .run()
}
