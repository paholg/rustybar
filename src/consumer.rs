use async_trait::async_trait;
use iced::{Element, Point, window};
use iced_layershell::to_layer_message;

pub mod battery;
pub mod clock;
pub mod cpu;
pub mod memory;
pub mod network;
pub mod temp;
pub mod tray;
pub mod window_diagram;
pub mod window_title;
pub mod workspace;

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
pub enum IcedMessage {
    A,
    Exit,
    /// The pointer moved over one of our surfaces.
    Cursor {
        window: window::Id,
        position: Point,
    },
    WindowClosed(window::Id),
    TrayActivate {
        address: String,
        secondary: bool,
    },
    TrayMenuOpen {
        address: String,
    },
    TrayMenuClick {
        popup: window::Id,
        address: String,
        menu_path: String,
        item: i32,
    },
}

#[async_trait]
pub trait Consumer: Send + Sync + 'static {
    // NOTE: Iced re-renders everything together anyway, so we don't use this,
    // at least for now.
    async fn consume(&mut self);

    fn render(&self, output: &str) -> Element<'_, IcedMessage>;
}

#[typetag::serde(tag = "type")]
pub trait Config {
    fn into_consumer(self: Box<Self>) -> Box<dyn Consumer>;
}
