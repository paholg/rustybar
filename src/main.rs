use iced::futures::Stream;
use iced::widget::{row, text};
use iced::{stream, Element, Font, Pixels, Subscription, Task, Theme};
use iced_layershell::reexport::Anchor;
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::to_layer_message;
use iced_layershell::Application;
use jiff::Zoned;
use serde::{Deserialize, Deserializer};
use tokio::time::sleep;

#[derive(Deserialize)]
#[serde(default)]
struct RustybarSettings {
    height: u32,
    #[serde(deserialize_with = "de_pixels")]
    font_size: Pixels,
}

fn de_pixels<'de, D>(deserializer: D) -> Result<Pixels, D::Error>
where
    D: Deserializer<'de>,
{
    let value = f32::deserialize(deserializer)?;
    Ok(Pixels(value))
}

impl Default for RustybarSettings {
    fn default() -> Self {
        Self {
            height: 32,
            font_size: Pixels(15.0),
        }
    }
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let s = RustybarSettings::default();

    Rustybar::run(Settings {
        id: Some("rustybar".into()),
        antialiasing: true,
        default_font: Font::MONOSPACE,
        default_text_size: s.font_size,
        layer_settings: LayerShellSettings {
            size: Some((0, s.height)),
            exclusive_zone: s.height.try_into()?,
            anchor: Anchor::Top | Anchor::Left | Anchor::Right,
            ..Default::default()
        },
        ..Default::default()
    })?;

    Ok(())
}

struct Rustybar {
    time: Zoned,
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    Time(Zoned),
}

impl Application for Rustybar {
    type Message = Message;
    type Flags = ();
    type Theme = Theme;
    type Executor = iced::executor::Default;

    fn new(_flags: ()) -> (Self, Task<Message>) {
        (Self { time: Zoned::now() }, Task::none())
    }

    fn namespace(&self) -> String {
        String::from("rustybar")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Time(time) => {
                self.time = time;
                Task::none()
            }
            // These are the variants created by to_layer_message
            Message::AnchorChange(..)
            | Message::SetInputRegion(..)
            | Message::AnchorSizeChange(..)
            | Message::LayerChange(..)
            | Message::MarginChange(..)
            | Message::SizeChange(..)
            | Message::VirtualKeyboardPressed { .. } => unreachable!(),
        }
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }

    fn view(&self) -> Element<Message> {
        let date = self.time.strftime("%a %Y-%m-%d");
        let time = self.time.strftime("%H:%M:%S");
        row![
            text("hello"),
            text(date.to_string()),
            text(time.to_string())
        ]
        .spacing(20)
        .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::run(worker)
    }
}

fn worker() -> impl Stream<Item = Message> {
    stream::channel(100, |mut output| async move {
        loop {
            let now = Zoned::now();
            output.try_send(Message::Time(now)).unwrap();
            sleep(std::time::Duration::from_secs(1)).await;
        }
    })
}
