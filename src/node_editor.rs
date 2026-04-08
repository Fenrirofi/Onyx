//! Node editor canvas implementation with drag-and-drop and selection.

use iced::{
    font::Family,
    mouse::{Button, Cursor, Event as MouseEvent, Interaction},
    widget::canvas::{self, Action, Frame, Geometry, Path, Stroke, Text},
    Color, Event, Font, Point, Rectangle, Size, Vector,
};

use crate::app::Message;                  // <-- Import the top-level Message
use crate::constants::{NODE_HEIGHT, NODE_WIDTH};

// -----------------------------------------------------------------------------
//  Messages for the node editor
// -----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub enum NodeEditorMessage {
    DragNode { index: usize, delta: Vector },
    SelectNode(usize),
    AddNode(Point),
    DragEnd,
}

// -----------------------------------------------------------------------------
//  Node Model
// -----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct Node {
    pub position: Point,
    pub label: String,
}

// -----------------------------------------------------------------------------
//  Node Editor (Canvas Program)
// -----------------------------------------------------------------------------
pub struct NodeEditor {
    pub nodes: Vec<Node>,
    pub selected_node: Option<usize>,
}

impl NodeEditor {
    pub fn new() -> Self {
        Self {
            nodes: vec![
                Node {
                    position: Point::new(50.0, 50.0),
                    label: "Node A".into(),
                },
                Node {
                    position: Point::new(200.0, 100.0),
                    label: "Node B".into(),
                },
            ],
            selected_node: None,
        }
    }

    /// Updates the internal state based on a node editor message.
    pub fn update(&mut self, msg: NodeEditorMessage) {
        match msg {
            NodeEditorMessage::DragNode { index, delta } => {
                if let Some(node) = self.nodes.get_mut(index) {
                    node.position = node.position + delta;
                }
            }
            NodeEditorMessage::SelectNode(index) => {
                self.selected_node = Some(index);
            }
            NodeEditorMessage::AddNode(position) => {
                let new_node = Node {
                    position,
                    label: format!("Node {}", self.nodes.len() + 1),
                };
                self.nodes.push(new_node);
            }
            NodeEditorMessage::DragEnd => {}
        }
    }

    /// Draws the background grid.
    fn draw_grid(&self, frame: &mut Frame, bounds: Rectangle) {
        let grid_spacing = 20.0;
        let stroke = Stroke::default()
            .with_width(1.0)
            .with_color(Color::from_rgb8(50, 50, 50));

        let width = bounds.width;
        let height = bounds.height;

        // Vertical lines
        for x in (0..width as i32).step_by(grid_spacing as usize) {
            let xf = x as f32;
            frame.stroke(
                &Path::line(Point::new(xf, 0.0), Point::new(xf, height)),
                stroke.clone(),
            );
        }

        // Horizontal lines
        for y in (0..height as i32).step_by(grid_spacing as usize) {
            let yf = y as f32;
            frame.stroke(
                &Path::line(Point::new(0.0, yf), Point::new(width, yf)),
                stroke.clone(),
            );
        }
    }

    /// Draws a single node.
    fn draw_node(&self, frame: &mut Frame, index: usize, node: &Node) {
        let rect = Path::rectangle(node.position, Size::new(NODE_WIDTH, NODE_HEIGHT));

        let fill_color = if Some(index) == self.selected_node {
            Color::from_rgb8(70, 70, 110) // Selected highlight
        } else {
            Color::from_rgb8(45, 45, 60)  // Default node color
        };

        frame.fill(&rect, fill_color);
        frame.stroke(
            &rect,
            Stroke::default().with_width(2.0).with_color(Color::WHITE),
        );

        let text = Text {
            content: node.label.clone(),
            position: node.position + Vector::new(10.0, 25.0),
            color: Color::WHITE,
            size: iced::Pixels(14.0),
            font: Font {
                family: Family::Name("Switzer"),
                weight: iced::font::Weight::Medium,
                ..Default::default()
            },
            ..Default::default()
        };
        frame.fill_text(text);
    }
}

// -----------------------------------------------------------------------------
//  Canvas Program Implementation
// -----------------------------------------------------------------------------
#[derive(Default)]
pub struct NodeEditorState {
    pub dragged_node: Option<usize>,
    pub drag_start: Point,
}

impl canvas::Program<Message> for NodeEditor {
    type State = NodeEditorState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<Action<Message>> {
        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                MouseEvent::ButtonPressed(Button::Left) => {
                    if let Some(pos) = cursor.position_in(bounds) {
                        // Check if an existing node was clicked (topmost first)
                        for (i, node) in self.nodes.iter().enumerate().rev() {
                            let node_bounds =
                                Rectangle::new(node.position, Size::new(NODE_WIDTH, NODE_HEIGHT));
                            if node_bounds.contains(pos) {
                                state.dragged_node = Some(i);
                                state.drag_start = pos;
                                return Some(Action::publish(Message::NodeEditor(
                                    NodeEditorMessage::SelectNode(i),
                                )));
                            }
                        }

                        // No node hit → add a new one
                        return Some(Action::publish(Message::NodeEditor(
                            NodeEditorMessage::AddNode(pos),
                        )));
                    }
                }

                MouseEvent::ButtonReleased(Button::Left) => {
                    state.dragged_node = None;
                    return Some(Action::publish(Message::NodeEditor(
                        NodeEditorMessage::DragEnd,
                    )));
                }

                MouseEvent::CursorMoved { .. } => {
                    if let (Some(idx), Some(current_pos)) =
                        (state.dragged_node, cursor.position_in(bounds))
                    {
                        let delta = current_pos - state.drag_start;
                        if delta.x != 0.0 || delta.y != 0.0 {
                            state.drag_start = current_pos;
                            return Some(Action::publish(Message::NodeEditor(
                                NodeEditorMessage::DragNode { index: idx, delta },
                            )));
                        }
                    }
                }

                _ => {}
            },

            _ => {}
        }

        None
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> Interaction {
        if state.dragged_node.is_some() {
            Interaction::Grabbing
        } else {
            Interaction::default()
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        // Background grid
        self.draw_grid(&mut frame, bounds);

        // Nodes
        for (i, node) in self.nodes.iter().enumerate() {
            self.draw_node(&mut frame, i, node);
        }

        vec![frame.into_geometry()]
    }
}