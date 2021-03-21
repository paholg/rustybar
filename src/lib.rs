use bar::{BarData, BarParams};
pub use util::bytes::format_bytes;
pub use util::color::{Color, ColorMap};
use util::screen::Screen;

pub mod bar;
pub mod config;
pub mod producer;
pub mod util;

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
    pub name: String,
    pub width: u32,
}

impl Font {
    pub const fn new(name: String, width: u32) -> Font {
        Font { name, width }
    }
}

pub struct RustyBar {
    pub left: Vec<BarData>,
    pub center: Vec<BarData>,
    pub right: Vec<BarData>,
}

impl RustyBar {
    pub fn new(left: Vec<BarData>, center: Vec<BarData>, right: Vec<BarData>) -> RustyBar {
        RustyBar {
            left,
            center,
            right,
        }
    }

    pub async fn start(&mut self, screen: &Screen) {
        let config = config::get().await;
        let mut x = screen.x;

        let center_width: u32 = self.center.iter().map(|b| b.width()).sum();
        let right_width: u32 = self.right.iter().map(|b| b.width()).sum();
        let center_x = (screen.width - center_width) / 2 + screen.x;

        let last_idx = self.left.len() - 1;
        for (i, bar) in self.left.iter_mut().enumerate() {
            let pad = if i == last_idx {
                center_x - x - bar.width()
            } else {
                0
            };
            let params = BarParams {
                x,
                y: screen.y,
                w: bar.width() + pad,
                font: config.font.name.clone(),
                bg: config.background.clone(),
                h: config.height,
            };
            bar.init(params).await;
            x += bar.width() + pad;
        }

        let last_idx = self.center.len() - 1;
        for (i, bar) in self.center.iter_mut().enumerate() {
            let pad = if i == last_idx {
                screen.x + screen.width - x - right_width - bar.width()
            } else {
                0
            };
            let params = BarParams {
                x,
                y: screen.y,
                w: bar.width() + pad,
                font: config.font.name.clone(),
                bg: config.background.clone(),
                h: config.height,
            };
            bar.init(params).await;
            x += bar.width() + pad;
        }

        for bar in &mut self.right {
            let params = BarParams {
                x,
                y: screen.y,
                w: bar.width(),
                font: config.font.name.clone(),
                bg: config.background.clone(),
                h: config.height,
            };
            bar.init(params).await;
            x += bar.width();
        }
    }
}
