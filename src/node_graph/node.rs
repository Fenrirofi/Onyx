use iced::{Color, Point, Rectangle};
use uuid::Uuid;

use crate::node_graph::port::{Port, PortType};

// ── Layout constants ──────────────────────────────────────────────────────────
pub const NODE_WIDTH: f32 = 180.0;
pub const NODE_HEADER_HEIGHT: f32 = 28.0;
pub const NODE_PREVIEW_HEIGHT: f32 = 160.0;
pub const PORT_ROW_HEIGHT: f32 = 22.0;
pub const PORT_RADIUS: f32 = 6.0;
pub const PORT_PADDING_X: f32 = 10.0;

// ── Node categories ───────────────────────────────────────────────────────────

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
            NodeCategory::Input   => "Wejście",
            NodeCategory::Texture => "Tekstura",
            NodeCategory::Math    => "Matematyka",
            NodeCategory::Color   => "Kolor",
            NodeCategory::Output  => "Wyjście",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            NodeCategory::Input   => Color::from_rgb(0.25, 0.45, 0.70),
            NodeCategory::Texture => Color::from_rgb(0.55, 0.30, 0.65),
            NodeCategory::Math    => Color::from_rgb(0.25, 0.55, 0.45),
            NodeCategory::Color   => Color::from_rgb(0.70, 0.45, 0.20),
            NodeCategory::Output  => Color::from_rgb(0.65, 0.25, 0.25),
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
            NodeCategory::Texture => vec![
                NodeKind::TextureSample,
                NodeKind::NormalMap,
            ],
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
            NodeCategory::Output => vec![
                NodeKind::PBROutput,
                NodeKind::UnlitOutput,
            ],
        }
    }
}

// ── Node kinds ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum NodeKind {
    // Input
    UVInput,
    VertexColor,
    Time,
    CameraPos,
    // Texture
    TextureSample,
    NormalMap,
    // Math
    Add,
    Multiply,
    Lerp,
    Clamp,
    Power,
    Fresnel,
    Mix,
    Gamma,
    // Color
    ColorConstant,
    HsvToRgb,
    RgbToHsv,
    // Output
    PBROutput,
    UnlitOutput,
}

impl NodeKind {
    pub fn label(&self) -> &'static str {
        match self {
            NodeKind::UVInput      => "UV Input",
            NodeKind::VertexColor  => "Vertex Color",
            NodeKind::Time         => "Time",
            NodeKind::CameraPos    => "Camera Position",
            NodeKind::TextureSample => "Texture Sample",
            NodeKind::NormalMap    => "Normal Map",
            NodeKind::Add          => "Add",
            NodeKind::Multiply     => "Multiply",
            NodeKind::Lerp         => "Lerp",
            NodeKind::Clamp        => "Clamp",
            NodeKind::Power        => "Power",
            NodeKind::Fresnel      => "Fresnel",
            NodeKind::Mix          => "Mix",
            NodeKind::Gamma        => "Gamma",
            NodeKind::ColorConstant => "Color",
            NodeKind::HsvToRgb     => "HSV to RGB",
            NodeKind::RgbToHsv     => "RGB to HSV",
            NodeKind::PBROutput    => "PBR Output",
            NodeKind::UnlitOutput  => "Unlit Output",
        }
    }

    pub fn category(&self) -> NodeCategory {
        match self {
            NodeKind::UVInput | NodeKind::VertexColor | NodeKind::Time | NodeKind::CameraPos
                => NodeCategory::Input,
            NodeKind::TextureSample | NodeKind::NormalMap
                => NodeCategory::Texture,
            NodeKind::Add | NodeKind::Multiply | NodeKind::Lerp | NodeKind::Clamp
            | NodeKind::Power | NodeKind::Fresnel | NodeKind::Mix | NodeKind::Gamma
                => NodeCategory::Math,
            NodeKind::ColorConstant | NodeKind::HsvToRgb | NodeKind::RgbToHsv
                => NodeCategory::Color,
            NodeKind::PBROutput | NodeKind::UnlitOutput
                => NodeCategory::Output,
        }
    }

    pub fn header_color(&self) -> Color {
        self.category().color()
    }

    /// Return (inputs, outputs) port definitions
    pub fn ports(&self) -> (Vec<Port>, Vec<Port>) {
        match self {
            NodeKind::UVInput => (
                vec![],
                vec![Port::new("UV", PortType::Vec2)],
            ),
            NodeKind::VertexColor => (
                vec![],
                vec![Port::new("Color", PortType::Color)],
            ),
            NodeKind::Time => (
                vec![],
                vec![Port::new("Time", PortType::Float)],
            ),
            NodeKind::CameraPos => (
                vec![],
                vec![Port::new("Position", PortType::Vec3)],
            ),
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
                vec![
                    Port::new("A", PortType::Any),
                    Port::new("B", PortType::Any),
                ],
                vec![Port::new("Out", PortType::Any)],
            ),
            NodeKind::Multiply => (
                vec![
                    Port::new("A", PortType::Any),
                    Port::new("B", PortType::Any),
                ],
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
            NodeKind::ColorConstant => (
                vec![],
                vec![Port::new("Color", PortType::Color)],
            ),
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
            NodeKind::UnlitOutput => (
                vec![Port::new("Color", PortType::Color)],
                vec![],
            ),
        }
    }
}

// ── Node instance ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Node {
    pub id: Uuid,
    pub kind: NodeKind,
    /// [x, y] stored as array for easy serde
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
        Rectangle::new(self.pos(), iced::Size::new(NODE_WIDTH, self.height()))
    }

    /// World-space center of input port `i`
    pub fn input_port_pos(&self, i: usize) -> Point {
        let y = self.position[1]
            + NODE_HEADER_HEIGHT + NODE_PREVIEW_HEIGHT
            + i as f32 * PORT_ROW_HEIGHT + PORT_ROW_HEIGHT * 0.5;
        Point::new(self.position[0], y)
    }

    /// World-space center of output port `i`
    pub fn output_port_pos(&self, i: usize) -> Point {
        let y = self.position[1]
            + NODE_HEADER_HEIGHT + NODE_PREVIEW_HEIGHT
            + i as f32 * PORT_ROW_HEIGHT + PORT_ROW_HEIGHT * 0.5;
        Point::new(self.position[0] + NODE_WIDTH, y)
    }

    /// Returns the port index if `p` is within hit radius of any output port.
    /// `zoom` scales the hit radius so snapping works at any zoom level.
    pub fn hit_test_output_port(&self, p: Point) -> Option<usize> {
        self.hit_test_output_port_zoomed(p, 1.0)
    }

    pub fn hit_test_output_port_zoomed(&self, p: Point, zoom: f32) -> Option<usize> {
        // Minimum 12 px on screen, scaled back to world units
        let hit_r = (PORT_RADIUS * 2.0).max(16.0 / zoom.sqrt());
        for i in 0..self.outputs.len() {
            let center = self.output_port_pos(i);
            if (p.x - center.x).powi(2) + (p.y - center.y).powi(2) <= hit_r * hit_r {
                return Some(i);
            }
        }
        None
    }

    /// Returns the port index if `p` is within hit radius of any input port.
    /// `zoom` scales the hit radius so snapping works at any zoom level.
    pub fn hit_test_input_port(&self, p: Point) -> Option<usize> {
        self.hit_test_input_port_zoomed(p, 1.0)
    }

    pub fn hit_test_input_port_zoomed(&self, p: Point, zoom: f32) -> Option<usize> {
        // Minimum 12 px on screen, scaled back to world units
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