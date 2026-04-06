use iced::{
    mouse, Point, Rectangle, Size, Vector,
    widget::canvas::{
        Cache, Frame, Geometry, Path, Program, Stroke, Text,
        path::Builder,
    },
};
use uuid::Uuid;

use crate::node_graph::{
    connection::PendingConnection,
    graph::Graph,
    node::{Node, NODE_WIDTH, NODE_HEADER_HEIGHT, NODE_PREVIEW_HEIGHT, PORT_RADIUS},
};

use crate::app::Message;
use crate::theme::THEME;

// ── Canvas interaction state ──────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct CanvasState {
    pub pan: Vector,
    pub zoom: f32,
    pub interaction: Interaction,
    pub hover_node: Option<Uuid>,
    pub hover_port: Option<(Uuid, PortSide, usize)>,
    pub hovered_wire: Option<Uuid>,
}

impl CanvasState {
    pub fn new() -> Self {
        Self { zoom: 1.0, ..Default::default() }
    }

    pub fn screen_to_world(&self, p: Point) -> Point {
        Point::new(
            (p.x - self.pan.x) / self.zoom,
            (p.y - self.pan.y) / self.zoom,
        )
    }

    pub fn world_to_screen(&self, p: Point) -> Point {
        Point::new(
            p.x * self.zoom + self.pan.x,
            p.y * self.zoom + self.pan.y,
        )
    }
}

#[derive(Debug, Clone, Default)]
pub enum Interaction {
    #[default]
    Idle,
    Panning { last: Point },
    DraggingNode { id: Uuid, offset: Vector, last_world: Point },
    DraggingWire(PendingConnection),
    Selecting { start: Point, current: Point },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortSide {
    Input,
    Output,
}

// ── Geometry caches ───────────────────────────────────────────────────────────

pub struct CanvasCaches {
    pub wires: Cache,
}

impl CanvasCaches {
    pub fn new() -> Self {
        Self { wires: Cache::new() }
    }

    pub fn invalidate_view(&self) {
        self.wires.clear();
    }

    pub fn invalidate_nodes(&self) {
        self.wires.clear();
    }
}

// ── Canvas Program ────────────────────────────────────────────────────────────

pub struct NodeCanvas<'a> {
    pub graph:   &'a Graph,
    pub state:   &'a CanvasState,
    pub pending: Option<&'a PendingConnection>,
    pub caches:  &'a CanvasCaches,
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

impl<'a> Program<Message> for NodeCanvas<'a> {
    type State = FrameKey;

    fn draw(
        &self,
        prev_key: &FrameKey,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
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

        // Kable
        for conn in &self.graph.connections {
            if let (Some(src), Some(dst)) = (
                self.graph.nodes.get(&conn.src_node),
                self.graph.nodes.get(&conn.dst_node),
            ) {
                let src_sp = self.state.world_to_screen(src.output_port_pos(conn.src_port));
                let dst_sp = self.state.world_to_screen(dst.input_port_pos(conn.dst_port));
                let src_off = src_sp.x < -margin || src_sp.x > vp.width + margin
                           || src_sp.y < -margin || src_sp.y > vp.height + margin;
                let dst_off = dst_sp.x < -margin || dst_sp.x > vp.width + margin
                           || dst_sp.y < -margin || dst_sp.y > vp.height + margin;
                if src_off && dst_off { continue; }
                let base_color = src.outputs.get(conn.src_port)
                    .map(|p| p.port_type.color())
                    .unwrap_or(THEME.wire);
                let color = if self.state.hovered_wire == Some(conn.id) {
                    iced::Color { r: (base_color.r + 0.4).min(1.0), g: (base_color.g + 0.4).min(1.0), b: (base_color.b + 0.4).min(1.0), a: 1.0 }
                } else {
                    base_color
                };
                let width_mult = if self.state.hovered_wire == Some(conn.id) { 2.0 } else { 1.0 };
                draw_bezier_wire(&mut main, src_sp, dst_sp, color, self.state.zoom, width_mult);
            }
        }

        // Węzły
        for node in self.graph.nodes_ordered() {
            draw_node(&mut main, node, self.state, vp, margin);
        }

        let mut overlay = Frame::new(renderer, vp);

        // Podświetlenie portu
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

        // Zaznaczanie prostokątem
        if let Interaction::Selecting { start, current } = &self.state.interaction {
            let rect = rect_from_two(
                self.state.world_to_screen(*start),
                self.state.world_to_screen(*current),
            );
            let path = Path::rectangle(rect.position(), rect.size());
            overlay.fill(&path, iced::Color { a: 0.08, ..THEME.accent });
            overlay.stroke(
                &path,
                Stroke::default()
                    .with_color(iced::Color { a: 0.6, ..THEME.accent })
                    .with_width(1.0),
            );
        }

        // Przeciągany kabel
        if let Some(pending) = self.pending {
            let src_sp = self.state.world_to_screen(pending.src_pos);
            let cur_sp = self.state.world_to_screen(pending.current_pos);
            draw_bezier_wire(&mut overlay, src_sp, cur_sp, THEME.wire_pending, self.state.zoom, 1.0);
        }

        vec![bg, main.into_geometry(), overlay.into_geometry()]
    }

    fn update(
        &self,
        key: &mut FrameKey,
        _event: &iced::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<iced::widget::Action<Message>> {
        *key = FrameKey::from(self.state);
        None
    }

    fn mouse_interaction(
        &self,
        _state: &FrameKey,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if !cursor.is_over(bounds) { return mouse::Interaction::default(); }
        match &self.state.interaction {
            Interaction::Panning { .. }      => mouse::Interaction::Grabbing,
            Interaction::DraggingNode { .. } => mouse::Interaction::Grabbing,
            Interaction::DraggingWire(_)     => mouse::Interaction::Crosshair,
            Interaction::Selecting { .. }    => mouse::Interaction::Crosshair,
            Interaction::Idle => {
                if self.state.hover_port.is_some() { mouse::Interaction::Crosshair }
                else if self.state.hover_node.is_some() { mouse::Interaction::Grab }
                else { mouse::Interaction::default() }
            }
        }
    }
}

// ── Drawing helpers ───────────────────────────────────────────────────────────

fn draw_bezier_wire(
    frame: &mut Frame,
    src: Point,
    dst: Point,
    color: iced::Color,
    zoom: f32,
    width_mult: f32,
) {
    let ctrl_offset = ((dst.x - src.x).abs() * 0.5).max(60.0 * zoom);
    let c1 = Point::new(src.x + ctrl_offset, src.y);
    let c2 = Point::new(dst.x - ctrl_offset, dst.y);

    let path = Path::new(|b| { b.move_to(src); b.bezier_curve_to(c1, c2, dst); });
    let w = 2.0 * zoom.sqrt().max(0.8) * width_mult;

    frame.stroke(&path, Stroke::default().with_color(iced::Color { a: 0.15, ..color }).with_width(w * 3.5));
    frame.stroke(&path, Stroke::default().with_color(color).with_width(w));
}

/// Rysuje pojedynczy węzeł – z POPRAWIONYM tekstem (stały rozmiar czcionki)
fn draw_node(frame: &mut Frame, node: &Node, state: &CanvasState, vp: Size, margin: f32) {
    let zoom  = state.zoom;
    let pos_s = state.world_to_screen(node.pos());
    let w     = NODE_WIDTH * zoom;
    let h     = node.height() * zoom;

    if pos_s.x + w < -margin || pos_s.x > vp.width  + margin
    || pos_s.y + h < -margin || pos_s.y > vp.height + margin {
        return;
    }

    let header_h  = NODE_HEADER_HEIGHT * zoom;

    // LOD
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
            frame.stroke(&body, Stroke::default()
                .with_color(iced::Color { a: 1.0, ..THEME.accent })
                .with_width(1.5));
        }
        return;
    }

    let preview_h = NODE_PREVIEW_HEIGHT * zoom;
    let corner    = 6.0 * zoom;

    // Shadow
    if zoom > 0.3 {
        frame.fill(
            &Path::new(|b| rounded_rect(b, pos_s.x + 3.0, pos_s.y + 4.0, w, h, corner)),
            iced::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.40 },
        );
    }

    // Body
    let body = Path::new(|b| rounded_rect(b, pos_s.x, pos_s.y, w, h, corner));
    frame.fill(&body, THEME.node_bg);

    // Header
    frame.fill(
        &Path::new(|b| rounded_rect_top(b, pos_s.x, pos_s.y, w, header_h, corner)),
        node.kind.header_color(),
    );

    // Preview
    let px = pos_s.x + 6.0 * zoom;
    let py = pos_s.y + header_h + 6.0 * zoom;
    let pw = w - 12.0 * zoom;
    let ph = preview_h - 12.0 * zoom;

    if w > 60.0 && pw > 4.0 && ph > 4.0 {
        let preview_path = Path::new(|b| rounded_rect(b, px, py, pw, ph, 3.0 * zoom));
        frame.fill(&preview_path, iced::Color { r: 0.08, g: 0.08, b: 0.10, a: 1.0 });
        draw_node_preview(frame, node, px, py, pw, ph, zoom);
        frame.stroke(
            &preview_path,
            Stroke::default()
                .with_color(iced::Color { r: 0.22, g: 0.22, b: 0.28, a: 1.0 })
                .with_width(0.5 * zoom.max(1.0)),
        );
    }

    // Outline
    let (outline_color, outline_w) = if node.selected {
        (iced::Color { a: 1.0, ..THEME.accent }, 2.0 * zoom)
    } else {
        (iced::Color { a: 0.3, ..THEME.node_border }, zoom.max(0.5))
    };
    frame.stroke(&body, Stroke::default().with_color(outline_color).with_width(outline_w));

    // ─── TYTUŁ WĘZŁA – STAŁY ROZMIAR CZCIONKI (nie skalowany z zoomem) ───
    if w > 40.0 {
        const TITLE_SIZE: f32 = 13.0;
        frame.fill_text(Text {
            content:  node.kind.label().to_string(),
            position: Point::new(pos_s.x + 10.0, pos_s.y + header_h * 0.5 - TITLE_SIZE * 0.5),
            color:    iced::Color::WHITE,
            size:     iced::Pixels(TITLE_SIZE),
            ..Text::default()
        });
    }

    // ─── PORTY I ICH ETYKIETY – STAŁY ROZMIAR CZCIONKI ───
    if zoom > 0.25 {
        const PORT_LABEL_SIZE: f32 = 11.0;
        for (i, port) in node.inputs.iter().enumerate() {
            let c = state.world_to_screen(node.input_port_pos(i));
            if c.x < -margin || c.x > vp.width + margin || c.y < -margin || c.y > vp.height + margin {
                continue;
            }
            draw_port(frame, c, port.port_type.color(), zoom);
            if w > 80.0 {
                frame.fill_text(Text {
                    content:  port.label.clone(),
                    position: Point::new(c.x + 10.0, c.y - PORT_LABEL_SIZE * 0.5),
                    color:    THEME.port_label,
                    size:     iced::Pixels(PORT_LABEL_SIZE),
                    ..Text::default()
                });
            }
        }
        for (i, port) in node.outputs.iter().enumerate() {
            let c = state.world_to_screen(node.output_port_pos(i));
            if c.x < -margin || c.x > vp.width + margin || c.y < -margin || c.y > vp.height + margin {
                continue;
            }
            draw_port(frame, c, port.port_type.color(), zoom);
            if w > 80.0 {
                let label_w = port.label.len() as f32 * PORT_LABEL_SIZE * 0.55;
                frame.fill_text(Text {
                    content:  port.label.clone(),
                    position: Point::new(c.x - 10.0 - label_w, c.y - PORT_LABEL_SIZE * 0.5),
                    color:    THEME.port_label,
                    size:     iced::Pixels(PORT_LABEL_SIZE),
                    ..Text::default()
                });
            }
        }
    }
}

/// Rysuje zawartość strefy preview – bez zmian (czcionki są już stałe)
fn draw_node_preview(
    frame: &mut Frame,
    node: &Node,
    x: f32, y: f32, w: f32, h: f32,
    zoom: f32,
) {
    use crate::node_graph::node::NodeKind;

    match &node.kind {
        NodeKind::ColorConstant => {
            let col = node.outputs.first()
                .map(|p| p.port_type.color())
                .unwrap_or(iced::Color::WHITE);
            let steps = 24u32;
            for i in 0..steps {
                let t  = i as f32 / steps as f32;
                let bx = x + t * w;
                let bw = w / steps as f32 + 1.0;
                frame.fill_rectangle(
                    Point::new(bx, y), Size::new(bw, h),
                    iced::Color { r: col.r * t, g: col.g * t, b: col.b * t, a: 1.0 },
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
                        iced::Color { r: 0.15, g: 0.15, b: 0.18, a: 1.0 }
                    } else {
                        iced::Color { r: 0.30, g: 0.30, b: 0.35, a: 1.0 }
                    };
                    frame.fill_rectangle(
                        Point::new(x + col as f32 * cw, y + row as f32 * ch),
                        Size::new(cw + 0.5, ch + 0.5),
                        fill,
                    );
                }
            }
            if zoom > 0.5 {
                frame.fill_text(Text {
                    content:  "TEX".to_string(),
                    position: Point::new(x + w * 0.5 - 5.5, y + h * 0.5 - 5.5),
                    color:    iced::Color { r: 0.6, g: 0.6, b: 0.7, a: 0.5 },
                    size:     iced::Pixels(11.0),
                    ..Text::default()
                });
            }
        }
        NodeKind::Time | NodeKind::Clamp | NodeKind::Power | NodeKind::Gamma | NodeKind::Fresnel => {
            let steps = 24u32;
            for i in 0..steps {
                let t  = i as f32 / steps as f32;
                let bx = x + t * w;
                let bw = w / steps as f32 + 1.0;
                frame.fill_rectangle(
                    Point::new(bx, y), Size::new(bw, h),
                    iced::Color { r: t, g: t, b: t, a: 1.0 },
                );
            }
        }
        NodeKind::Add | NodeKind::Multiply | NodeKind::Lerp | NodeKind::Mix => {
            let steps = (w as u32).max(2);
            let mid_y = y + h * 0.5;
            let amp   = h * 0.38;
            let mut prev = Point::new(x, mid_y);
            for i in 1..=steps {
                let t   = i as f32 / steps as f32;
                let sx  = x + t * w;
                let sy  = mid_y - (t * std::f32::consts::TAU * 1.5).sin() * amp;
                let cur = Point::new(sx, sy);
                frame.stroke(
                    &Path::line(prev, cur),
                    Stroke::default()
                        .with_color(iced::Color { r: 0.4, g: 0.7, b: 1.0, a: 0.85 })
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
                        iced::Color { r: u, g: v, b: 0.2, a: 1.0 },
                    );
                }
            }
        }
        NodeKind::VertexColor | NodeKind::HsvToRgb | NodeKind::RgbToHsv => {
            let steps = 24u32;
            for i in 0..steps {
                let t   = i as f32 / steps as f32;
                let c   = hue_to_rgb(t * 360.0);
                let bx  = x + t * w;
                let bw  = w / steps as f32 + 1.0;
                frame.fill_rectangle(Point::new(bx, y),           Size::new(bw, h * 0.55), c);
                frame.fill_rectangle(
                    Point::new(bx, y + h * 0.55), Size::new(bw, h * 0.45),
                    iced::Color { r: c.r * t, g: c.g * t, b: c.b * t, a: 1.0 },
                );
            }
        }
        NodeKind::CameraPos => {
            let mx  = x + w * 0.5;
            let my  = y + h * 0.5;
            let arm = h * 0.32;
            let col = iced::Color { r: 0.5, g: 0.8, b: 1.0, a: 0.7 };
            let stroke = Stroke::default().with_color(col).with_width(1.5 * zoom.sqrt());
            frame.stroke(&Path::line(Point::new(mx - arm, my), Point::new(mx + arm, my)), stroke);
            frame.stroke(&Path::line(Point::new(mx, my - arm), Point::new(mx, my + arm)), stroke);
            frame.stroke(&Path::circle(Point::new(mx, my), arm * 0.45), stroke);
        }
        NodeKind::PBROutput | NodeKind::UnlitOutput => {
            let mx = x + w * 0.5;
            let my = y + h * 0.5;
            let r  = h * 0.30;
            let col = iced::Color { r: 0.8, g: 0.4, b: 0.4, a: 0.7 };
            frame.stroke(
                &Path::circle(Point::new(mx, my), r),
                Stroke::default().with_color(col).with_width(1.5 * zoom.sqrt()),
            );
            frame.stroke(
                &Path::circle(Point::new(mx, my), r * 0.55),
                Stroke::default().with_color(col).with_width(1.0 * zoom.sqrt()),
            );
            frame.fill(&Path::circle(Point::new(mx, my), r * 0.2), col);
        }
    }
}

fn hue_to_rgb(hue: f32) -> iced::Color {
    let h = hue / 60.0;
    let i = h.floor() as u32 % 6;
    let f = h - h.floor();
    let q = 1.0 - f;
    let (r, g, b) = match i {
        0 => (1.0, f,   0.0),
        1 => (q,   1.0, 0.0),
        2 => (0.0, 1.0, f  ),
        3 => (0.0, q,   1.0),
        4 => (f,   0.0, 1.0),
        _ => (1.0, 0.0, q  ),
    };
    iced::Color { r, g, b, a: 1.0 }
}

fn draw_port(frame: &mut Frame, center: Point, color: iced::Color, zoom: f32) {
    let r     = PORT_RADIUS * zoom;
    let fill  = iced::Color { a: 0.3, ..color };
    let ring  = iced::Color { a: 0.7, ..color };

    frame.fill(&Path::circle(center, (r - 1.5 * zoom).max(0.5)), fill);
    frame.stroke(
        &Path::circle(center, r),
        Stroke::default().with_color(ring).with_width(1.5 * zoom),
    );
}

fn draw_port_highlight(frame: &mut Frame, center: Point, zoom: f32) {
    let r = (PORT_RADIUS + 4.0) * zoom;
    let path = Path::circle(center, r);
    frame.fill(&path, iced::Color { a: 0.2, ..THEME.accent });
    frame.stroke(&path, Stroke::default().with_color(THEME.accent).with_width(2.0 * zoom));
}

// ── Path builders ─────────────────────────────────────────────────────────────

fn rounded_rect(b: &mut Builder, x: f32, y: f32, w: f32, h: f32, r: f32) {
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

fn rounded_rect_top(b: &mut Builder, x: f32, y: f32, w: f32, h: f32, r: f32) {
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
    Rectangle::new(Point::new(x, y), Size::new((a.x - b.x).abs(), (a.y - b.y).abs()))
}