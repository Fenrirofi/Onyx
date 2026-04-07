use iced::{Background, Border, Color, Element, Font, Length, Padding, font::Family, widget::*};

fn main() -> iced::Result {
    iced::application(OnyxApp::default, OnyxApp::update, OnyxApp::view)
        .font(include_bytes!("../assets/fonts/Switzer-Regular.otf").as_slice())
        .run()
}

const BG_PRIM: Color = Color::from_rgb8(38, 38, 38);
const BG_SECO: Color = Color::from_rgb8(34, 34, 34);
const SEPA_CO: Color = Color::from_rgb8(27, 27, 27);
const CT_PRIM: Color = Color::from_rgb8(179, 179, 179);
// const CT_SECO: Color = Color::from_rgb8(r, g, b);

#[derive(Debug, Clone)]
enum Message {
    PaneResized(pane_grid::ResizeEvent),
}

struct OnyxApp {
    panes: pane_grid::State<Pane>,
}

impl Default for OnyxApp {
    fn default() -> Self {
        OnyxApp::new()
    }
}

impl OnyxApp {
    fn new() -> Self {
        let (mut panes, node_pane) = pane_grid::State::new(Pane::new(PanelType::NodeGraph));

        let (preview_3d_pane, _) = panes
            .split(
                pane_grid::Axis::Horizontal,
                node_pane,
                Pane::new(PanelType::Preview3D),
            )
            .expect("Pierwszy podział nie udał się");

        let _ = panes.split(
            pane_grid::Axis::Vertical,
            preview_3d_pane,
            Pane::new(PanelType::Preview2D),
        );

        OnyxApp { panes }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::PaneResized(event) => {
                self.panes.resize(event.split, event.ratio);
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let grid = pane_grid(&self.panes, |_id, pane, _is_maximized| {
            pane_grid::Content::new(self.draw_panel(pane))
        })
        
        .on_resize(10.0, |event| Message::PaneResized(event));

        container(grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn draw_panel(&self, pane: &Pane) -> Element<'_, Message> {
        let content: Element<'_, Message> = match pane.panel_type {
            PanelType::NodeGraph => container(text("h").size(14)).into(),
            PanelType::Preview3D => {
                container(button("C")).into()
            }
            PanelType::Preview2D => container(text("Text")).into(),
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
                    weight: iced::font::Weight::Medium,
                    ..Default::default()
                })
                .color(CT_PRIM),
        )
        .width(Length::Fill)
        .padding(Padding::default().left(22).bottom(4).top(4))
        .style(|_theme| container::Style {
            background: Some(BG_PRIM.into()),
            ..Default::default()
        });

        iced::widget::column![
            header,
            container(
                container(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(|_| container::Style {
                        background: Some(Background::Color(BG_SECO)),

                        border: Border {
                            width: 1.5,
                            color: SEPA_CO,
                            ..Default::default()
                        },
                        ..Default::default()
                    })
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(2)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(BG_PRIM)),
                ..Default::default()
            })
        ]
        .into()
    }
}

#[derive(Debug, Clone, Copy)]
enum PanelType {
    NodeGraph,
    Preview3D,
    Preview2D,
}

#[derive(Debug)] 
struct Pane {
    pub panel_type: PanelType,
}

impl Pane {
    fn new(panel_type: PanelType) -> Self {
        Self { panel_type }
    }
}
