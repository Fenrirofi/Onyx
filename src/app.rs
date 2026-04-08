//! Main application struct and logic.

use iced::{
    font::Family,
    widget::{button, canvas, container, pane_grid, text, Column},
    Background, Border, Element, Font, Length, Padding,
};

use crate::constants::{BG_PRIMARY, BG_SECONDARY, SEPARATOR_COLOR, TEXT_PRIMARY};
use crate::node_editor::{NodeEditor, NodeEditorMessage};
use crate::pane::{Pane, PanelType};

// -----------------------------------------------------------------------------
//  Top-level Messages
// -----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub enum Message {
    PaneResized(pane_grid::ResizeEvent),
    NodeEditor(NodeEditorMessage),
}

// -----------------------------------------------------------------------------
//  OnyxApp Struct
// -----------------------------------------------------------------------------
pub struct OnyxApp {
    panes: pane_grid::State<Pane>,
    node_editor: NodeEditor,
}

impl Default for OnyxApp {
    fn default() -> Self {
        Self::new()
    }
}

impl OnyxApp {
    pub fn new() -> Self {
        // Create initial pane grid with a NodeGraph panel
        let (mut panes, node_pane) = pane_grid::State::new(Pane::new(PanelType::NodeGraph));

        // Split horizontally to add a 3D preview pane
        let (preview_3d_pane, _) = panes
            .split(
                pane_grid::Axis::Horizontal,
                node_pane,
                Pane::new(PanelType::Preview3D),
            )
            .expect("Horizontal split");

        // Split vertically to add a 2D preview pane below the 3D one
        let _ = panes.split(
            pane_grid::Axis::Vertical,
            preview_3d_pane,
            Pane::new(PanelType::Preview2D),
        );

        Self {
            panes,
            node_editor: NodeEditor::new(),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::PaneResized(event) => {
                self.panes.resize(event.split, event.ratio);
            }
            Message::NodeEditor(msg) => {
                self.node_editor.update(msg);
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let grid = pane_grid(&self.panes, |_id, pane, _is_maximized| {
            pane_grid::Content::new(self.draw_panel(pane))
        })
        .on_resize(10, |event| Message::PaneResized(event));

        container(grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn draw_panel(&self, pane: &Pane) -> Element<'_, Message> {
        let content: Element<_> = match pane.panel_type {
            PanelType::NodeGraph => canvas(&self.node_editor)
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),

            PanelType::Preview3D => container(button("Preview 3D placeholder")).into(),

            PanelType::Preview2D => container(text("Preview 2D placeholder")).into(),
        };

        self.panel_style(pane.panel_type, content)
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
                    weight: iced::font::Weight::Normal,
                    ..Default::default()
                })
                .color(TEXT_PRIMARY),
        )
        .width(Length::Fill)
        .padding(Padding::default().left(22).bottom(6).top(6))
        .style(|_theme| container::Style {
            background: Some(BG_PRIMARY.into()),
            ..Default::default()
        });

        Column::new()
            .push(header)
            .push(
                container(
                    container(content)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .style(|_| container::Style {
                            background: Some(Background::Color(BG_SECONDARY)),
                            border: Border {
                                width: 1.5,
                                color: SEPARATOR_COLOR,
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(4)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Background::Color(BG_PRIMARY)),
                    ..Default::default()
                }),
            )
            .into()
    }
}