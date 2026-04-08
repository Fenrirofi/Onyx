//! Pane definitions and panel types used in the grid layout.

/// Type of panel displayed in a pane.
#[derive(Debug, Clone, Copy)]
pub enum PanelType {
    NodeGraph,
    Preview3D,
    Preview2D,
}

/// A pane holding a specific panel type.
#[derive(Debug)]
pub struct Pane {
    pub panel_type: PanelType,
}

impl Pane {
    pub fn new(panel_type: PanelType) -> Self {
        Self { panel_type }
    }
}