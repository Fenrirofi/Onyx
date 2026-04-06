use uuid::Uuid;
use iced::Point;

/// A directed edge from (src_node, src_port) → (dst_node, dst_port)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Connection {
    pub id: Uuid,
    pub src_node: Uuid,
    pub src_port: usize,  // index into outputs
    pub dst_node: Uuid,
    pub dst_port: usize,  // index into inputs
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

/// Temporary state while the user is dragging a new wire
#[derive(Debug, Clone)]
pub struct PendingConnection {
    pub src_node: Uuid,
    pub src_port: usize,
    pub src_pos: Point,     // world-space socket center
    pub current_pos: Point, // world-space cursor tip
}