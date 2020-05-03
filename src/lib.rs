use bar::DynBar;
pub use bytes::format_bytes;
pub use color::{Color, ColorMap};
use screen::Screen;
use ticker::Ticker;

pub mod bar;
mod bytes;
mod color;
pub mod config;
pub mod draw;
pub mod screen;
mod stdin;
mod ticker;
mod updater;

/// A convenience macro for creating a static Regex, for repeated use at only one call-site.  Copied
/// from https://github.com/Canop/lazy-regex
#[macro_export]
macro_rules! regex {
    ($s: literal) => {{
        lazy_static::lazy_static! {
            static ref RE: regex::Regex = regex::Regex::new($s).unwrap();
        }
        &*RE
    }};
}

#[derive(Clone, Debug)]
pub struct Font {
    pub name: &'static str,
    pub width: u32,
}

impl Font {
    pub const fn new(name: &'static str, width: u32) -> Font {
        Font { name, width }
    }
}

#[derive(Debug)]
pub struct RustyBar {
    pub screen_id: u32,
    pub left: Vec<DynBar>,
    pub center: Vec<DynBar>,
    pub right: Vec<DynBar>,
}

impl Clone for RustyBar {
    fn clone(&self) -> Self {
        RustyBar {
            screen_id: self.screen_id,
            left: self.left.iter().map(|b| b.box_clone()).collect(),
            center: self.center.iter().map(|b| b.box_clone()).collect(),
            right: self.right.iter().map(|b| b.box_clone()).collect(),
        }
    }
}

impl RustyBar {
    pub fn new(
        screen_id: u32,
        left: Vec<DynBar>,
        center: Vec<DynBar>,
        right: Vec<DynBar>,
    ) -> RustyBar {
        RustyBar {
            screen_id,
            left,
            center,
            right,
        }
    }

    pub async fn stop(&self) {
        for bar in self
            .left
            .iter()
            .chain(self.center.iter())
            .chain(self.right.iter())
        {
            bar.updater().clear().await;
        }
    }

    pub async fn start(&self, screen: &screen::Screen) {
        async fn start_bar(bar: DynBar, x: u32, y: u32, pad: u32, config: &config::Config) {
            let width = bar.width();
            let running_bar = bar::RunningBar::start(
                bar,
                config.font.name,
                x,
                y,
                width + pad,
                config.height,
                config.background,
            );
            running_bar.register().await;
        }

        let config = config::get().await;
        let mut x = screen.x;

        let center_width: u32 = self.center.iter().map(|b| b.width()).sum();
        let right_width: u32 = self.right.iter().map(|b| b.width()).sum();
        let center_x = (screen.width - center_width) / 2 + screen.x;

        for (i, bar) in self.left.iter().enumerate() {
            let pad = if i == self.left.len() - 1 {
                center_x - x - bar.width()
            } else {
                0
            };
            start_bar(bar.box_clone(), x, screen.y, pad, &config).await;
            x += bar.width() + pad;
        }

        for (i, bar) in self.center.iter().enumerate() {
            let pad = if i == self.center.len() - 1 {
                screen.x + screen.width - x - right_width - bar.width()
            } else {
                0
            };
            start_bar(bar.box_clone(), x, screen.y, pad, &config).await;
            x += bar.width() + pad;
        }

        for bar in &self.right {
            start_bar(bar.box_clone(), x, screen.y, 0, &config).await;
            x += bar.width();
        }
    }
}

pub async fn start(bars: Vec<RustyBar>) {
    Ticker.set_bar_config(bars).await;
    Ticker.restart().await;
}
