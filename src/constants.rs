//! Global constants used throughout the application.

use iced::Color;

/// Primary background color (panels, main area).
pub const BG_PRIMARY: Color = Color::from_rgb8(38, 38, 38);

/// Secondary background color (content areas).
pub const BG_SECONDARY: Color = Color::from_rgb8(34, 34, 34);

/// Color used for separators and borders.
pub const SEPARATOR_COLOR: Color = Color::from_rgb8(27, 27, 27);

/// Primary text color.
pub const TEXT_PRIMARY: Color = Color::from_rgb8(179, 179, 179);

/// Default width of a node in the node editor.
pub const NODE_WIDTH: f32 = 120.0;

/// Default height of a node in the node editor.
pub const NODE_HEIGHT: f32 = 60.0;