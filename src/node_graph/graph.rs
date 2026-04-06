use std::collections::HashMap;
use uuid::Uuid;
use iced::{Point, Vector, Rectangle};

use crate::node_graph::node::{Node, NodeKind};
use crate::node_graph::connection::Connection;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Graph {
    pub nodes: HashMap<Uuid, Node>,
    pub connections: Vec<Connection>,
    // Przechowujemy kolejność ID dla determinizmu i warstw
    node_order: Vec<Uuid>,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

    // ── Node management ───────────────────────────────────────────────────────

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
        self.connections.retain(|c| c.src_node != id && c.dst_node != id);
    }

    pub fn duplicate_node(&mut self, id: Uuid) -> Option<Uuid> {
        let node = self.nodes.get(&id)?.clone();
        // POPRAWKA: pozycja to [f32; 2], więc używamy [0] i [1]
        let offset = Point::new(node.position[0] + 30.0, node.position[1] + 30.0);
        let new_id = self.add_node(node.kind.clone(), offset);
        Some(new_id)
    }

    pub fn nodes_ordered(&self) -> impl Iterator<Item = &Node> {
        self.node_order.iter().filter_map(|id| self.nodes.get(id))
    }

    /// Bezpieczna iteracja mutowalna przez domknięcie
    pub fn for_each_node_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Node),
    {
        for id in &self.node_order {
            if let Some(node) = self.nodes.get_mut(id) {
                f(node);
            }
        }
    }

    pub fn bring_to_front(&mut self, id: Uuid) {
        if self.nodes.contains_key(&id) {
            self.node_order.retain(|n| *n != id);
            self.node_order.push(id);
        }
    }

    // ── Selection ─────────────────────────────────────────────────────────────

    pub fn select_only(&mut self, id: Uuid) {
        for node in self.nodes.values_mut() {
            node.selected = node.id == id;
        }
    }

    pub fn toggle_select(&mut self, id: Uuid) {
        if let Some(n) = self.nodes.get_mut(&id) {
            n.selected = !n.selected;
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
        self.nodes.values()
            .filter(|n| n.selected)
            .map(|n| n.id)
            .collect()
    }

    pub fn move_selected(&mut self, delta: Vector) {
        for node in self.nodes.values_mut() {
            if node.selected {
                // POPRAWKA: używamy indeksów tablicy zamiast .x / .y
                node.position[0] += delta.x;
                node.position[1] += delta.y;
            }
        }
    }

    // ── Connection management ─────────────────────────────────────────────────

    pub fn add_connection(
        &mut self,
        src_node: Uuid, src_port: usize,
        dst_node: Uuid, dst_port: usize,
    ) -> bool {
        let src_type = {
            let node = match self.nodes.get(&src_node) { Some(n) => n, None => return false };
            match node.outputs.get(src_port) { Some(p) => p.port_type.clone(), None => return false }
        };
        let dst_type = {
            let node = match self.nodes.get(&dst_node) { Some(n) => n, None => return false };
            match node.inputs.get(dst_port) { Some(p) => p.port_type.clone(), None => return false }
        };

        if !src_type.compatible_with(&dst_type) {
            return false;
        }

        if src_node == dst_node {
            return false;
        }

        self.connections.retain(|c| !(c.dst_node == dst_node && c.dst_port == dst_port));
        self.connections.push(Connection::new(src_node, src_port, dst_node, dst_port));
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

    /// Jak connection_at_point, ale pomija kable podłączone do `exclude_node`.
    pub fn connection_at_point_excluding(&self, p: Point, threshold: f32, exclude_node: Uuid) -> Option<Uuid> {
        for conn in &self.connections {
            if conn.src_node == exclude_node || conn.dst_node == exclude_node { continue; }
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

    // ── Serialization ─────────────────────────────────────────────────────────

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
        let bx = it*it*it*p0.x + 3.0*it*it*t*p1.x + 3.0*it*t*t*p2.x + t*t*t*p3.x;
        let by = it*it*it*p0.y + 3.0*it*it*t*p1.y + 3.0*it*t*t*p2.y + t*t*t*p3.y;
        let ddx = p.x - bx;
        let ddy = p.y - by;
        if ddx*ddx + ddy*ddy < threshold * threshold {
            return true;
        }
    }
    false
}
// ── Auto-layout (DAG layered layout) ─────────────────────────────────────────

impl Graph {
    /// Układa węzły w równych kolumnach według topologicznej kolejności.
    /// Węzły bez wejść → kolumna 0, ich następniki → kolumna 1, itd.
    pub fn auto_layout(&mut self) {
        use std::collections::{HashMap, VecDeque};

        if self.nodes.is_empty() { return; }

        let ids: Vec<Uuid> = self.node_order.clone();

        // Zbuduj mapę: dst_node → lista src_node (krawędzie grafu)
        let mut predecessors: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut successors:   HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        for id in &ids {
            predecessors.insert(*id, vec![]);
            successors.insert(*id, vec![]);
        }
        for conn in &self.connections {
            predecessors.entry(conn.dst_node).or_default().push(conn.src_node);
            successors.entry(conn.src_node).or_default().push(conn.dst_node);
        }

        // ── Kahn's topological sort + przypisanie warstw ──────────────────
        let mut in_degree: HashMap<Uuid, usize> = ids.iter()
            .map(|id| (*id, predecessors[id].len()))
            .collect();

        let mut layer: HashMap<Uuid, usize> = HashMap::new();
        let mut queue: VecDeque<Uuid> = ids.iter()
            .filter(|id| in_degree[id] == 0)
            .copied()
            .collect();

        for id in queue.iter() { layer.insert(*id, 0); }

        while let Some(id) = queue.pop_front() {
            let cur_layer = layer[&id];
            if let Some(succs) = successors.get(&id) {
                for &s in succs {
                    // Węzeł trafia do warstwy co najmniej cur_layer+1
                    let entry = layer.entry(s).or_insert(0);
                    *entry = (*entry).max(cur_layer + 1);

                    let deg = in_degree.entry(s).or_insert(0);
                    if *deg > 0 { *deg -= 1; }
                    if *deg == 0 { queue.push_back(s); }
                }
            }
        }

        // Węzły bez przypisanej warstwy (cykle / izolowane) → ostatnia+1
        let max_layer = layer.values().copied().max().unwrap_or(0);
        for id in &ids {
            layer.entry(*id).or_insert(max_layer + 1);
        }

        // ── Pogrupuj według warstwy i posortuj dla stabilności ───────────
        let mut by_layer: HashMap<usize, Vec<Uuid>> = HashMap::new();
        for id in &ids {
            by_layer.entry(layer[id]).or_default().push(*id);
        }

        // ── Oblicz pozycje ────────────────────────────────────────────────
        const H_GAP: f32 = 60.0;   // pozioma przerwa między kolumnami
        const V_GAP: f32 = 30.0;   // pionowa przerwa między węzłami
        const START_X: f32 = 60.0;
        const START_Y: f32 = 60.0;
        const NODE_W: f32 = crate::node_graph::node::NODE_WIDTH;

        // Szerokość najgrubszego węzła w warstwie (wszystkie mają NODE_W)
        let col_w = NODE_W + H_GAP;

        let mut col_heights: HashMap<usize, f32> = HashMap::new();
        let mut positions: Vec<(Uuid, f32, f32)> = vec![];

        let mut sorted_layers: Vec<usize> = by_layer.keys().copied().collect();
        sorted_layers.sort();

        for col in sorted_layers {
            let nodes_in_col = &by_layer[&col];
            let x = START_X + col as f32 * col_w;
            let mut y = START_Y;

            for &id in nodes_in_col {
                let h = self.nodes[&id].height();
                positions.push((id, x, y));
                y += h + V_GAP;
            }
            col_heights.insert(col, y);
        }

        // ── Zastosuj pozycje ──────────────────────────────────────────────
        for (id, x, y) in positions {
            if let Some(node) = self.nodes.get_mut(&id) {
                node.position[0] = x;
                node.position[1] = y;
            }
        }
    }
}