//! Entry point for the Onyx Node Editor application.

mod app;
mod constants;
mod node_editor;
mod pane;

use app::OnyxApp;

fn main() -> iced::Result {
    iced::application(OnyxApp::default, OnyxApp::update, OnyxApp::view)
        .title("Onyx Node Editor")
        .font(include_bytes!("../assets/fonts/Switzer-Regular.otf").as_slice())
        .run()
}