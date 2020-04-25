pub mod bar;
mod bytes;
mod color;
mod colormap;
pub mod config;
pub mod screen;
pub mod state;

use bar::DynBar;
pub use bytes::format_bytes;
pub use color::Color;
pub use colormap::ColorMap;

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

    pub async fn start(&self, screen: &screen::Screen) {
        let mut state = state::write().await;
        let mut stdin = bar::stdin::state::STDIN.write().await;
        let config = config::read().await;

        let mut start_bar = |bar: &DynBar, x, pad| {
            let running_bar = bar::RunningBar::start(
                bar.box_clone(),
                config.font.name,
                x,
                screen.y,
                bar.width() + pad,
                config.height,
                config.background,
            );
            match running_bar.bar.update_on() {
                bar::UpdateOn::Tick => state.register_bar(running_bar),
                bar::UpdateOn::Stdin => stdin.register_bar(running_bar),
                bar::UpdateOn::Custom => (), // Do nothing here; it's up to the user to handle
            }
        };

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
            start_bar(bar, x, pad);
            x += bar.width() + pad;
        }

        for (i, bar) in self.center.iter().enumerate() {
            let pad = if i == self.center.len() - 1 {
                screen.x + screen.width - x - right_width - bar.width()
            } else {
                0
            };
            start_bar(bar, x, pad);
            x += bar.width() + pad;
        }

        for bar in &self.right {
            start_bar(bar, x, 0);
            x += bar.width();
        }
    }
}

pub async fn start(bars: &[RustyBar]) {
    let screens = state::read().await.screens.clone();
    for bar in bars {
        if let Some(screen) = screens.iter().find(|&&screen| screen.id == bar.screen_id) {
            bar.start(screen).await;
        }
    }
}
