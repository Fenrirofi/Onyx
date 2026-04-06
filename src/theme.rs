use iced::Color;

/// Global colour palette used throughout the node editor.
/// Import with: `use crate::theme::THEME;`
pub struct EditorTheme {
    pub bg:          Color,
    pub grid:        Color,
    pub node_bg:     Color,
    pub node_border: Color,
    pub accent:      Color,
    pub wire:        Color,
    pub wire_pending: Color,
    pub port_label:  Color,
    pub text:        Color,
}

pub const THEME: EditorTheme = EditorTheme {
    bg:          Color { r: 0.11, g: 0.11, b: 0.13, a: 1.0 },
    grid:        Color { r: 0.18, g: 0.18, b: 0.22, a: 1.0 },
    node_bg:     Color { r: 0.17, g: 0.17, b: 0.20, a: 1.0 },
    node_border: Color { r: 0.28, g: 0.28, b: 0.35, a: 1.0 },
    accent:      Color { r: 0.40, g: 0.70, b: 1.00, a: 1.0 },
    wire:        Color { r: 0.55, g: 0.55, b: 0.65, a: 1.0 },
    wire_pending: Color { r: 0.90, g: 0.70, b: 0.30, a: 1.0 },
    port_label:  Color { r: 0.75, g: 0.75, b: 0.80, a: 1.0 },
    text:        Color { r: 0.92, g: 0.92, b: 0.94, a: 1.0 },
};