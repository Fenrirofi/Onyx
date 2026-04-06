use iced::Color;

/// Type of data flowing through a port
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
            PortType::Float   => Color::from_rgb(0.60, 0.85, 0.60),
            PortType::Vec2    => Color::from_rgb(0.60, 0.70, 1.00),
            PortType::Vec3    => Color::from_rgb(0.80, 0.60, 1.00),
            PortType::Vec4    => Color::from_rgb(1.00, 0.60, 0.80),
            PortType::Color   => Color::from_rgb(1.00, 0.85, 0.30),
            PortType::Texture => Color::from_rgb(1.00, 0.65, 0.30),
            PortType::Any     => Color::from_rgb(0.70, 0.70, 0.70),
        }
    }

    pub fn compatible_with(&self, other: &PortType) -> bool {
        *self == PortType::Any || *other == PortType::Any || self == other
    }
}

/// A single input or output port on a node
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Port {
    pub label: String,
    pub port_type: PortType,
}

impl Port {
    pub fn new(label: impl Into<String>, port_type: PortType) -> Self {
        Self { label: label.into(), port_type }
    }
}
