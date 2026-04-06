use iced::{
    Alignment, Color, Event, Length, Point, Rectangle, Size, Subscription, Task, Theme, Vector,
    event, keyboard, mouse,
    widget::{
        Space, button, canvas::Canvas, column, container, row, scrollable, stack, text, text_input,
    },
};
use uuid::Uuid;

use crate::node_graph::{
    canvas::{CanvasCaches, CanvasState, Interaction, NodeCanvas, PortSide},
    connection::PendingConnection,
    graph::Graph,
    node::{NODE_WIDTH, NodeCategory, NodeKind},
    port::PortType,
};
use crate::theme::THEME;

// Typ elementu z naszym rendererem
pub type Element<'a> = iced::Element<'a, Message, Theme, iced::Renderer>;

#[derive(Debug, Clone)]
pub enum Message {
    CanvasEvent(CanvasMsg),
    AddNode(NodeKind, Point),
    DeleteSelected,
    DuplicateSelected,
    SelectAll,
    ToggleNodePicker,
    SearchChanged(String),
    FitAll,
    AutoLayout,
    ResetZoom,
    SaveGraph,
    NewGraph,
    Undo,
    Redo,
    QuickConnectPick(NodeKind),
    QuickConnectCancel,
    KeyPressed(keyboard::Key),
}

#[derive(Debug, Clone)]
pub enum CanvasMsg {
    MouseMoved(Point),
    LeftPressed(Point),
    LeftReleased(Point),
    RightPressed(Point),
    MiddlePressed(Point),
    MiddleReleased(Point),
    Scrolled(Point, mouse::ScrollDelta),
}

pub struct NodeEditorApp {
    pub graph: Graph,
    pub canvas_state: CanvasState,
    pub pending: Option<PendingConnection>,
    pub caches: CanvasCaches,
    pub show_node_picker: bool,
    pub search_query: String,
    pub canvas_bounds: Rectangle,
    pub status: String,
    pub quick_connect: Option<QuickConnectMenu>,
    pub ctrl_held: bool,
    pub shift_held: bool,
}

#[derive(Debug, Clone)]
pub struct QuickConnectMenu {
    pub pending: PendingConnection,
    pub screen_pos: Point,
    pub src_type: PortType,
    pub world_pos: Point,
    pub candidates: Vec<(NodeKind, usize)>,
}

impl NodeEditorApp {
    pub fn new() -> (Self, Task<Message>) {
        let mut graph = Graph::new();

        let uv = graph.add_node(NodeKind::UVInput, Point::new(60.0, 200.0));
        let tex = graph.add_node(NodeKind::TextureSample, Point::new(280.0, 180.0));
        let gam = graph.add_node(NodeKind::Gamma, Point::new(510.0, 160.0));
        let fres = graph.add_node(NodeKind::Fresnel, Point::new(280.0, 400.0));
        let mix = graph.add_node(NodeKind::Mix, Point::new(730.0, 250.0));
        let pbr = graph.add_node(NodeKind::PBROutput, Point::new(960.0, 160.0));

        graph.add_connection(uv, 0, tex, 0);
        graph.add_connection(tex, 1, gam, 0);
        graph.add_connection(gam, 0, mix, 0);
        graph.add_connection(fres, 0, mix, 2);
        graph.add_connection(mix, 0, pbr, 0);

        let mut cs = CanvasState::new();
        cs.pan = Vector::new(20.0, 20.0);

        (
            Self {
                graph,
                canvas_state: cs,
                pending: None,
                caches: CanvasCaches::new(),
                show_node_picker: false,
                search_query: String::new(),
                canvas_bounds: Rectangle::new(Point::ORIGIN, Size::new(1200.0, 900.0)),
                status: "Gotowy. Kliknij PPM na płótnie aby dodać węzeł.".into(),
                quick_connect: None,
                ctrl_held: false,
                shift_held: false,
            },
            Task::none(),
        )
    }

    pub fn title(&self) -> String {
        "mxp — Node Editor".to_string()
    }

    pub fn theme(&self) -> Theme {
        Theme::custom(
            "mxp",
            iced::theme::palette::Seed {
                background: Color::from_rgb(0.11, 0.11, 0.13),
                text: Color::from_rgb(0.92, 0.92, 0.94),
                primary: Color::from_rgb(0.40, 0.70, 1.00),
                success: Color::from_rgb(0.30, 0.80, 0.50),
                warning: Color::from_rgb(0.90, 0.35, 0.35),
                danger: Color::from_rgb(0.90, 0.35, 0.35),
            },
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        event::listen().map(|e| match e {
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => Message::KeyPressed(key),
            _ => Message::KeyPressed(keyboard::Key::Named(keyboard::key::Named::F35)),
        })
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::KeyPressed(key) => self.handle_key(key),
            Message::CanvasEvent(ev) => self.handle_canvas_event(ev),
            Message::AddNode(kind, world_pos) => {
                let id = self.graph.add_node(kind.clone(), world_pos);
                self.graph.deselect_all();
                if let Some(n) = self.graph.nodes.get_mut(&id) {
                    n.selected = true;
                }
                self.caches.invalidate_nodes();
                self.status = format!("Dodano węzeł: {}", kind.label());
                self.show_node_picker = false;
            }
            Message::DeleteSelected => {
                let selected = self.graph.selected_ids();
                let count = selected.len();
                for id in selected {
                    self.graph.remove_node(id);
                }
                self.caches.invalidate_nodes();
                self.status = format!("Usunięto {} węzł(ów).", count);
            }
            Message::DuplicateSelected => {
                let selected = self.graph.selected_ids();
                self.graph.deselect_all();
                for id in selected {
                    if let Some(new_id) = self.graph.duplicate_node(id) {
                        if let Some(n) = self.graph.nodes.get_mut(&new_id) {
                            n.selected = true;
                        }
                    }
                }
                self.caches.invalidate_nodes();
                self.status = "Zduplikowano zaznaczenie.".into();
            }
            Message::SelectAll => {
                for n in self.graph.nodes.values_mut() {
                    n.selected = true;
                }
                self.caches.invalidate_nodes();
            }
            Message::ToggleNodePicker => {
                self.show_node_picker = !self.show_node_picker;
                self.search_query.clear();
            }
            Message::SearchChanged(q) => {
                self.search_query = q;
            }
            Message::FitAll => self.fit_all_nodes(),
            Message::AutoLayout => {
                self.graph.auto_layout();
                self.fit_all_nodes();
                self.status = "Auto-layout zastosowany.".into();
            }
            Message::QuickConnectPick(kind) => {
                if let Some(qc) = self.quick_connect.take() {
                    let port_idx = qc
                        .candidates
                        .iter()
                        .find(|(k, _)| *k == kind)
                        .map(|(_, idx)| *idx)
                        .unwrap_or(0);
                    let new_id = self.graph.add_node(kind, qc.world_pos);
                    self.graph.add_connection(
                        qc.pending.src_node,
                        qc.pending.src_port,
                        new_id,
                        port_idx,
                    );
                    self.status = "Dodano i połączono węzeł.".into();
                    self.caches.invalidate_nodes();
                }
            }
            Message::QuickConnectCancel => {
                self.quick_connect = None;
            }
            Message::ResetZoom => {
                self.canvas_state.zoom = 1.0;
                self.caches.invalidate_view();
                self.status = "Zoom zresetowany do 100%.".into();
            }
            Message::SaveGraph => {
                let json = self.graph.to_json();
                self.status = format!("Graf zapisany ({} znaków).", json.len());
            }
            Message::NewGraph => {
                self.graph = Graph::new();
                self.canvas_state = CanvasState::new();
                self.caches.invalidate_view();
                self.status = "Nowy graf.".into();
            }
            Message::Undo | Message::Redo => {
                self.status = "Undo/Redo jeszcze niezaimplementowane.".into();
            }
        }
        Task::none()
    }

    fn handle_key(&mut self, key: keyboard::Key) {
        use keyboard::Key::Named;
        use keyboard::key::Named::*;
        match key {
            Named(Delete) | Named(Backspace) => {
                let _ = self.update(Message::DeleteSelected);
            }
            Named(Escape) => {
                self.show_node_picker = false;
                self.quick_connect = None;
                self.graph.deselect_all();
                self.pending = None;
                self.canvas_state.interaction = Interaction::Idle;
            }
            keyboard::Key::Character(c) if c.as_str() == "d" => {
                let _ = self.update(Message::DuplicateSelected);
            }
            keyboard::Key::Character(c) if c.as_str() == "f" => {
                self.fit_all_nodes();
            }
            keyboard::Key::Character(c) if c.as_str() == "a" => {
                let _ = self.update(Message::SelectAll);
            }
            keyboard::Key::Character(c) if c.as_str() == " " => {
                self.show_node_picker = !self.show_node_picker;
            }
            _ => {}
        }
    }

    fn handle_canvas_event(&mut self, ev: CanvasMsg) {
        match ev {
            CanvasMsg::LeftPressed(screen_pos) => {
                let world = self.canvas_state.screen_to_world(screen_pos);
                self.handle_left_press(world, screen_pos);
            }
            CanvasMsg::LeftReleased(screen_pos) => {
                let world = self.canvas_state.screen_to_world(screen_pos);
                self.handle_left_release(world);
            }
            CanvasMsg::RightPressed(screen_pos) => {
                let world = self.canvas_state.screen_to_world(screen_pos);
                let wire_threshold = 6.0 / self.canvas_state.zoom;
                if let Some(conn_id) = self.graph.connection_at_point(world, wire_threshold) {
                    self.graph.remove_connection(conn_id);
                    self.caches.invalidate_nodes();
                    self.status = "Połączenie usunięte.".into();
                    return;
                }
                let hit = self.hit_test_node(world);
                if hit.is_none() {
                    self.show_node_picker = !self.show_node_picker;
                    self.search_query.clear();
                }
            }
            CanvasMsg::MiddlePressed(screen_pos) => {
                self.canvas_state.interaction = Interaction::Panning { last: screen_pos };
            }
            CanvasMsg::MiddleReleased(_) => {
                if matches!(self.canvas_state.interaction, Interaction::Panning { .. }) {
                    self.canvas_state.interaction = Interaction::Idle;
                }
            }
            CanvasMsg::MouseMoved(screen_pos) => {
                let world = self.canvas_state.screen_to_world(screen_pos);
                self.handle_mouse_move(world, screen_pos);
            }
            CanvasMsg::Scrolled(screen_pos, delta) => {
                self.handle_scroll(screen_pos, delta);
            }
        }
    }

    fn handle_left_press(&mut self, world: Point, _screen: Point) {
        if let Some((node_id, port_idx, src_pos)) = self.hit_test_output_port(world) {
            self.graph.bring_to_front(node_id);
            let pc = PendingConnection {
                src_node: node_id,
                src_port: port_idx,
                src_pos,
                current_pos: world,
            };
            self.canvas_state.interaction = Interaction::DraggingWire(pc.clone());
            self.pending = Some(pc);
            self.caches.invalidate_nodes();
            return;
        }
        if let Some(node_id) = self.hit_test_node(world) {
            self.graph.bring_to_front(node_id);
            if !self.graph.nodes[&node_id].selected {
                self.graph.select_only(node_id);
            }
            let node_pos = self.graph.nodes[&node_id].pos();
            self.canvas_state.interaction = Interaction::DraggingNode {
                id: node_id,
                offset: Vector::new(world.x - node_pos.x, world.y - node_pos.y),
                last_world: world,
            };
            self.caches.invalidate_nodes();
            return;
        }
        self.graph.deselect_all();
        self.caches.invalidate_nodes();
        self.canvas_state.interaction = Interaction::Selecting {
            start: world,
            current: world,
        };
    }

    fn handle_left_release(&mut self, world: Point) {
        if self.quick_connect.is_some() {
            self.quick_connect = None;
            return;
        }
        let interaction = std::mem::replace(&mut self.canvas_state.interaction, Interaction::Idle);
        match interaction {
            Interaction::DraggingWire(pending) => {
                self.pending = None;
                if let Some((dst_node, dst_port)) = self.hit_test_input_port(world) {
                    self.graph.add_connection(
                        pending.src_node,
                        pending.src_port,
                        dst_node,
                        dst_port,
                    );
                    self.status = "Połączono porty.".into();
                } else {
                    let src_type = self
                        .graph
                        .nodes
                        .get(&pending.src_node)
                        .and_then(|n| n.outputs.get(pending.src_port))
                        .map(|p| p.port_type.clone());
                    if let Some(src_type) = src_type {
                        let candidates = build_quick_connect_candidates(&src_type);
                        if !candidates.is_empty() {
                            let screen_pos = self.canvas_state.world_to_screen(world);
                            self.quick_connect = Some(QuickConnectMenu {
                                pending,
                                screen_pos,
                                src_type,
                                world_pos: world,
                                candidates,
                            });
                        }
                    }
                }
                self.caches.invalidate_nodes();
            }
            Interaction::Selecting { start, current } => {
                let rect = selection_rect(start, current);
                if rect.width >= 4.0 || rect.height >= 4.0 {
                    self.graph.select_rect(rect);
                }
                self.caches.invalidate_nodes();
            }
            Interaction::DraggingNode { id, .. } => {
                if let Some(wire_id) = self.canvas_state.hovered_wire.take() {
                    self.try_insert_node_on_wire(id, wire_id);
                }
                self.canvas_state.hovered_wire = None;
                self.caches.invalidate_nodes();
            }
            _ => {}
        }
    }

    fn handle_mouse_move(&mut self, world: Point, screen: Point) {
        self.canvas_state.hover_node = self.hit_test_node(world);
        self.canvas_state.hover_port = self.hit_test_any_port(world);
        match &mut self.canvas_state.interaction {
            Interaction::DraggingNode { id, last_world, .. } => {
                let id = *id;
                let delta = Vector::new(world.x - last_world.x, world.y - last_world.y);
                *last_world = world;
                self.graph.move_selected(delta);
                let threshold = 8.0 / self.canvas_state.zoom;
                let node_center = self.graph.nodes.get(&id).map(|n| {
                    let b = n.bounds();
                    Point::new(b.x + b.width * 0.5, b.y + b.height * 0.5)
                });
                self.canvas_state.hovered_wire = node_center
                    .and_then(|c| self.graph.connection_at_point_excluding(c, threshold, id));
                self.caches.invalidate_nodes();
            }
            Interaction::DraggingWire(pending) => {
                let snap_r = 20.0 / self.canvas_state.zoom;
                let snapped = self
                    .graph
                    .nodes_ordered()
                    .flat_map(|n| {
                        n.inputs
                            .iter()
                            .enumerate()
                            .map(move |(i, _)| (n.input_port_pos(i), n.id, i))
                    })
                    .filter(|(p, _, _)| {
                        (p.x - world.x).powi(2) + (p.y - world.y).powi(2) <= snap_r * snap_r
                    })
                    .min_by(|(a, _, _), (b, _, _)| {
                        let da = (a.x - world.x).powi(2) + (a.y - world.y).powi(2);
                        let db = (b.x - world.x).powi(2) + (b.y - world.y).powi(2);
                        da.partial_cmp(&db).unwrap()
                    })
                    .map(|(p, _, _)| p);
                pending.current_pos = snapped.unwrap_or(world);
                self.pending = Some(pending.clone());
            }
            Interaction::Selecting { current, .. } => {
                *current = world;
            }
            Interaction::Panning { last } => {
                let delta = Vector::new(screen.x - last.x, screen.y - last.y);
                self.canvas_state.pan.x += delta.x;
                self.canvas_state.pan.y += delta.y;
                *last = screen;
                self.caches.invalidate_view();
            }
            Interaction::Idle => {}
        }
    }

    fn handle_scroll(&mut self, screen: Point, delta: mouse::ScrollDelta) {
        let scroll_y = match delta {
            mouse::ScrollDelta::Lines { y, .. } => y,
            mouse::ScrollDelta::Pixels { y, .. } => y / 50.0,
        };
        let factor = if scroll_y > 0.0 { 1.1 } else { 1.0 / 1.1 };
        let new_zoom = (self.canvas_state.zoom * factor).clamp(0.1, 4.0);
        let world_before = self.canvas_state.screen_to_world(screen);
        self.canvas_state.zoom = new_zoom;
        let world_after = self.canvas_state.screen_to_world(screen);
        self.canvas_state.pan.x += (world_after.x - world_before.x) * new_zoom;
        self.canvas_state.pan.y += (world_after.y - world_before.y) * new_zoom;
        self.caches.invalidate_view();
    }

    fn hit_test_node(&self, world: Point) -> Option<Uuid> {
        for node in self
            .graph
            .nodes_ordered()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            if node.bounds().contains(world) {
                return Some(node.id);
            }
        }
        None
    }

    fn hit_test_output_port(&self, world: Point) -> Option<(Uuid, usize, Point)> {
        let zoom = self.canvas_state.zoom;
        for node in self
            .graph
            .nodes_ordered()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            if let Some(i) = node.hit_test_output_port_zoomed(world, zoom) {
                return Some((node.id, i, node.output_port_pos(i)));
            }
        }
        None
    }

    fn hit_test_input_port(&self, world: Point) -> Option<(Uuid, usize)> {
        let zoom = self.canvas_state.zoom;
        for node in self
            .graph
            .nodes_ordered()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            if let Some(i) = node.hit_test_input_port_zoomed(world, zoom) {
                return Some((node.id, i));
            }
        }
        None
    }

    fn hit_test_any_port(&self, world: Point) -> Option<(Uuid, PortSide, usize)> {
        let zoom = self.canvas_state.zoom;
        for node in self
            .graph
            .nodes_ordered()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            if let Some(i) = node.hit_test_output_port_zoomed(world, zoom) {
                return Some((node.id, PortSide::Output, i));
            }
            if let Some(i) = node.hit_test_input_port_zoomed(world, zoom) {
                return Some((node.id, PortSide::Input, i));
            }
        }
        None
    }

    fn try_insert_node_on_wire(&mut self, node_id: Uuid, wire_id: Uuid) {
        let conn = match self.graph.get_connection(wire_id) {
            Some(c) => c.clone(),
            None => return,
        };
        let src_out_type = self
            .graph
            .nodes
            .get(&conn.src_node)
            .and_then(|n| n.outputs.get(conn.src_port))
            .map(|p| p.port_type.clone());
        let dst_in_type = self
            .graph
            .nodes
            .get(&conn.dst_node)
            .and_then(|n| n.inputs.get(conn.dst_port))
            .map(|p| p.port_type.clone());
        let (src_out_type, dst_in_type) = match (src_out_type, dst_in_type) {
            (Some(a), Some(b)) => (a, b),
            _ => return,
        };
        let node_in_port = self.graph.nodes.get(&node_id).and_then(|n| {
            n.inputs
                .iter()
                .enumerate()
                .find(|(_, p)| src_out_type.compatible_with(&p.port_type))
                .map(|(i, _)| i)
        });
        let node_out_port = self.graph.nodes.get(&node_id).and_then(|n| {
            n.outputs
                .iter()
                .enumerate()
                .find(|(_, p)| p.port_type.compatible_with(&dst_in_type))
                .map(|(i, _)| i)
        });
        match (node_in_port, node_out_port) {
            (Some(in_idx), Some(out_idx)) => {
                self.graph.remove_connection(wire_id);
                self.graph
                    .add_connection(conn.src_node, conn.src_port, node_id, in_idx);
                self.graph
                    .add_connection(node_id, out_idx, conn.dst_node, conn.dst_port);
                self.status = "Węzeł wstawiony w połączenie.".into();
            }
            _ => {
                self.status = "Węzeł niekompatybilny z tym połączeniem.".into();
            }
        }
    }

    fn fit_all_nodes(&mut self) {
        if self.graph.nodes.is_empty() {
            return;
        }
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for node in self.graph.nodes.values() {
            let b = node.bounds();
            min_x = min_x.min(b.x);
            min_y = min_y.min(b.y);
            max_x = max_x.max(b.x + b.width);
            max_y = max_y.max(b.y + b.height);
        }
        let pad = 80.0;
        let content_w = max_x - min_x + pad * 2.0;
        let content_h = max_y - min_y + pad * 2.0;
        let view_w = self.canvas_bounds.width;
        let view_h = self.canvas_bounds.height;
        let zoom = (view_w / content_w).min(view_h / content_h).min(1.5);
        self.canvas_state.zoom = zoom;
        self.canvas_state.pan = Vector::new(
            (view_w - content_w * zoom) * 0.5 - (min_x - pad) * zoom,
            (view_h - content_h * zoom) * 0.5 - (min_y - pad) * zoom,
        );
        self.status = format!("Widok dopasowany — zoom {:.0}%.", zoom * 100.0);
        self.caches.invalidate_view();
    }

    pub fn view(&self) -> Element<'_> {
        let toolbar = self.view_toolbar();
        let canvas = self.view_canvas();
        let sidebar = self.view_sidebar();
        let status = self.view_statusbar();

        let main_row = row![
            canvas,
            if self.show_node_picker {
                sidebar
            } else {
                Space::new().width(0).into()
            },
        ]
        .spacing(0)
        .height(Length::Fill);

        let canvas_layer: Element<'_> = if let Some(qc) = &self.quick_connect {
            stack![main_row, self.view_quick_connect_menu(qc)].into()
        } else {
            main_row.into()
        };

        column![toolbar, canvas_layer, status].spacing(0).into()
    }

    fn view_toolbar(&self) -> Element<'_> {
        let btn = |label: &'static str, msg: Message| {
            button(text(label).size(12)).padding([4, 10]).on_press(msg)
        };
        let zoom_label = text(format!("{:.0}%", self.canvas_state.zoom * 100.0))
            .size(12)
            .color(Color::from_rgb(0.7, 0.7, 0.8));

        let toolbar = container(
            row![
                btn("Nowy", Message::NewGraph),
                btn("Zapisz JSON", Message::SaveGraph),
                Space::new().width(Length::Fixed(12.0)),
                Space::new().width(Length::Fixed(12.0)),
                btn("Cofnij", Message::Undo),
                btn("Ponów", Message::Redo),
                Space::new().width(Length::Fixed(12.0)),
                Space::new().width(Length::Fixed(12.0)),
                btn("Usuń [Del]", Message::DeleteSelected),
                btn("Duplikuj [D]", Message::DuplicateSelected),
                btn("Zaznacz wszystko [A]", Message::SelectAll),
                Space::new().width(Length::Fill),
                btn("Dopasuj [F]", Message::FitAll),
                btn("Auto-layout", Message::AutoLayout),
                btn("100%", Message::ResetZoom),
                zoom_label,
                Space::new().width(Length::Fixed(12.0)),
                btn(
                    if self.show_node_picker {
                        "✕ Węzły"
                    } else {
                        "+ Węzły [Space]"
                    },
                    Message::ToggleNodePicker
                ),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        )
        .padding([6, 10])
        .width(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.13, 0.13, 0.16))),
            border: iced::Border {
                color: Color::from_rgb(0.22, 0.22, 0.28),
                width: 0.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        });
        toolbar.into()
    }

    fn view_canvas(&self) -> Element<'_> {
        let canvas_widget = Canvas::new(NodeCanvas {
            graph: &self.graph,
            state: &self.canvas_state,
            pending: self.pending.as_ref(),
            caches: &self.caches,
        })
        .width(Length::Fill)
        .height(Length::Fill);

        CanvasEventCapture {
            inner: canvas_widget,
            state: &self.canvas_state,
        }
        .into_element()
    }

    fn view_sidebar(&self) -> Element<'_> {
        let query = self.search_query.to_lowercase();
        let search = text_input("Szukaj węzłów...", &self.search_query)
            .on_input(Message::SearchChanged)
            .padding([6, 10])
            .size(13);

        let mut items: Vec<Element<'_>> = vec![];

        for category in NodeCategory::all() {
            let nodes: Vec<_> = category
                .nodes()
                .into_iter()
                .filter(|k| query.is_empty() || k.label().to_lowercase().contains(&query))
                .collect();
            if nodes.is_empty() {
                continue;
            }
            let cat_color = category.color();
            let cat_header = container(text(category.label()).size(11).color(Color::WHITE))
                .padding([3, 8])
                .width(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(cat_color)),
                    border: iced::Border {
                        radius: 3.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                });
            items.push(cat_header.into());
            for kind in nodes {
                let label = kind.label();
                let world_pos = self.canvas_state.screen_to_world(Point::new(
                    self.canvas_bounds.width * 0.5 - NODE_WIDTH * 0.5,
                    self.canvas_bounds.height * 0.5,
                ));
                let kind_clone = kind.clone();
                let btn = button(text(label).size(12))
                    .padding([5, 12])
                    .width(Length::Fill)
                    .on_press(Message::AddNode(kind_clone, world_pos))
                    .style(button::secondary);
                items.push(btn.into());
            }
            items.push(Space::new().height(4).into());
        }

        let content = scrollable(column(items).spacing(2).padding([4, 6])).height(Length::Fill);
        let panel = container(
            column![
                container(text("  Węzły").size(13).color(THEME.text))
                    .padding([8, 0])
                    .width(Length::Fill),
                search,
                Space::new().height(6),
                content,
            ]
            .spacing(0),
        )
        .width(Length::Fixed(210.0))
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.14, 0.14, 0.17))),
            border: iced::Border {
                color: Color::from_rgb(0.22, 0.22, 0.28),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        });
        panel.into()
    }

    fn view_statusbar(&self) -> Element<'_> {
        let sel_count = self.graph.selected_ids().len();
        let node_count = self.graph.nodes.len();
        let conn_count = self.graph.connections.len();
        let info = format!(
            "Węzły: {}  |  Połączenia: {}  |  Zaznaczone: {}  |  Zoom: {:.0}%",
            node_count,
            conn_count,
            sel_count,
            self.canvas_state.zoom * 100.0,
        );
        container(
            row![
                text(&self.status)
                    .size(11)
                    .color(Color::from_rgb(0.7, 0.8, 0.7)),
                Space::new().width(Length::Fill),
                text(info).size(11).color(Color::from_rgb(0.5, 0.5, 0.6)),
            ]
            .align_y(Alignment::Center),
        )
        .padding([4, 10])
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.10, 0.10, 0.12))),
            border: iced::Border {
                color: Color::from_rgb(0.20, 0.20, 0.25),
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
    }

    fn view_quick_connect_menu(&self, qc: &QuickConnectMenu) -> Element<'_> {
        let mx = qc.screen_pos.x + 8.0;
        let my = qc.screen_pos.y + 8.0;
        let port_color = qc.src_type.color();

        let header = container(
            row![
                Space::new().width(Length::Fixed(2.0)),
                text(format!("→  {:?}", qc.src_type))
                    .size(11)
                    .color(Color::from_rgb(port_color.r, port_color.g, port_color.b)),
            ]
            .align_y(Alignment::Center)
            .spacing(6),
        )
        .padding([6, 10])
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.16, 0.16, 0.20))),
            ..Default::default()
        });

        let mut items = column![header].spacing(1);
        for (kind, _port_idx) in &qc.candidates {
            let kind_clone = kind.clone();
            let cat_color = kind.category().color();
            let btn = button(
                row![
                    Space::new().width(Length::Fixed(4.0)),
                    container(Space::new().width(Length::Fixed(3.0)))
                        .height(Length::Fixed(14.0))
                        .style(move |_| container::Style {
                            background: Some(iced::Background::Color(cat_color)),
                            border: iced::Border {
                                radius: 2.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    Space::new().width(Length::Fixed(7.0)),
                    text(kind.label()).size(12),
                ]
                .align_y(Alignment::Center)
                .spacing(0),
            )
            .padding([5, 10])
            .width(Length::Fill)
            .on_press(Message::QuickConnectPick(kind_clone))
            .style(|_, status| button::Style {
                background: Some(iced::Background::Color(match status {
                    button::Status::Hovered | button::Status::Pressed => {
                        Color::from_rgb(0.25, 0.30, 0.40)
                    }
                    _ => Color::from_rgb(0.15, 0.15, 0.19),
                })),
                border: iced::Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
                text_color: Color::from_rgb(0.88, 0.88, 0.92),
                ..Default::default()
            });
            items = items.push(btn);
        }

        let cancel_btn = button(
            text("✕  Anuluj")
                .size(11)
                .color(Color::from_rgb(0.5, 0.5, 0.6)),
        )
        .padding([4, 10])
        .width(Length::Fill)
        .on_press(Message::QuickConnectCancel)
        .style(|_, _| button::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.13, 0.13, 0.16))),
            ..Default::default()
        });

        items = items.push(Space::new().height(Length::Fixed(2.0)));
        items = items.push(cancel_btn);

        let menu = container(items)
            .width(Length::Fixed(200.0))
            .style(|_| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.14, 0.14, 0.18))),
                border: iced::Border {
                    color: Color::from_rgb(0.30, 0.35, 0.50),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.6),
                    offset: iced::Vector::new(4.0, 6.0),
                    blur_radius: 16.0,
                },
                ..Default::default()
            });

        container(menu)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(iced::Padding {
                top: my.max(0.0),
                left: mx.max(0.0),
                right: 0.0,
                bottom: 0.0,
            })
            .into()
    }
}

fn build_quick_connect_candidates(src_type: &PortType) -> Vec<(NodeKind, usize)> {
    use crate::node_graph::node::NodeKind::*;
    let all: &[NodeKind] = &[
        Add,
        Multiply,
        Lerp,
        Clamp,
        Power,
        Fresnel,
        Mix,
        Gamma,
        TextureSample,
        NormalMap,
        HsvToRgb,
        RgbToHsv,
        PBROutput,
        UnlitOutput,
    ];
    let mut result = Vec::new();
    for kind in all {
        let (inputs, _) = kind.ports();
        for (idx, port) in inputs.iter().enumerate() {
            if src_type.compatible_with(&port.port_type) {
                result.push((kind.clone(), idx));
                break;
            }
        }
    }
    result
}

// ── CanvasEventCapture ───────────────────────────────────────────────────────

use iced::advanced::{self, layout, renderer, widget::Widget};

struct CanvasEventCapture<'a> {
    inner: Canvas<NodeCanvas<'a>, Message, Theme>,
    state: &'a CanvasState,
}

impl<'a> CanvasEventCapture<'a> {
    fn into_element(self) -> Element<'a> {
        Element::new(self)
    }
}

impl<'a> Widget<Message, Theme, iced::Renderer> for CanvasEventCapture<'a> {
    fn size(&self) -> Size<Length> {
        Widget::size(&self.inner)
    }

    fn layout(
        &mut self,
        tree: &mut advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        Widget::layout(&mut self.inner, tree, renderer, limits)
    }

    fn draw(
        &self,
        tree: &advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        Widget::draw(
            &self.inner,
            tree,
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        )
    }

    fn tag(&self) -> advanced::widget::tree::Tag {
        Widget::tag(&self.inner)
    }

    fn state(&self) -> advanced::widget::tree::State {
        Widget::state(&self.inner)
    }

    fn children(&self) -> Vec<advanced::widget::Tree> {
        Widget::children(&self.inner)
    }

    fn update(
        &mut self,
        tree: &mut advanced::widget::Tree,
        event: &Event,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer, // ZMIANA: z BackendRenderer na iced::Renderer
        shell: &mut advanced::Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let cursor_pos = match cursor.position_in(bounds) {
            Some(p) => p,
            None => {
                return Widget::update(
                    &mut self.inner,
                    tree,
                    event,
                    layout,
                    cursor,
                    renderer,
                    shell,
                    viewport,
                );
            }
        };

        let screen = Point::new(cursor_pos.x, cursor_pos.y);
        
        let msg = match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                Some(Message::CanvasEvent(CanvasMsg::LeftPressed(screen)))
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                Some(Message::CanvasEvent(CanvasMsg::LeftReleased(screen)))
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                Some(Message::CanvasEvent(CanvasMsg::RightPressed(screen)))
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) => {
                Some(Message::CanvasEvent(CanvasMsg::MiddlePressed(screen)))
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Middle)) => {
                Some(Message::CanvasEvent(CanvasMsg::MiddleReleased(screen)))
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                let rel = Point::new(position.x - bounds.x, position.y - bounds.y);
                Some(Message::CanvasEvent(CanvasMsg::MouseMoved(rel)))
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                Some(Message::CanvasEvent(CanvasMsg::Scrolled(screen, *delta)))
            }
            _ => None,
        };

        if let Some(m) = msg {
            shell.publish(m);
            // Uwaga: Jeśli chcesz, żeby Canvas też dostał to zdarzenie, 
            // nie rób tutaj return, tylko wywołaj Widget::update na końcu.
            return;
        }

        Widget::update(
            &mut self.inner,
            tree,
            event,
            layout,
            cursor,
            renderer,
            shell,
            viewport,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &advanced::widget::Tree,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer, // ZMIANA: z BackendRenderer na iced::Renderer
    ) -> mouse::Interaction {
        Widget::mouse_interaction(&self.inner, tree, layout, cursor, viewport, renderer)
    }
}

fn selection_rect(a: Point, b: Point) -> Rectangle {
    let x = a.x.min(b.x);
    let y = a.y.min(b.y);
    Rectangle::new(
        Point::new(x, y),
        Size::new((a.x - b.x).abs(), (a.y - b.y).abs()),
    )
}
