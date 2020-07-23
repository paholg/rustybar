use crate::ticker::Ticker;
use crate::Color;
use async_trait::async_trait;

/// A statusbar for battery information.
#[derive(Debug, Clone)]
pub struct Battery {
    colormap: crate::ColorMap,
    bar_width: u32,
    bar_height: u32,
    char_width: u32,
    space: u32,
    padding: u32,
    colors: BatteryColors,
}

#[derive(Debug, Clone)]
pub struct BatteryColors {
    pub charge: Color,
    pub discharge: Color,
    pub unknown: Color,
}

impl Battery {
    pub async fn new(
        colormap: crate::ColorMap,
        state_colors: BatteryColors,
        bar_width: u32,
        bar_height: u32,
        char_width: u32,
        space: u32,
        padding: u32,
    ) -> Box<Battery> {
        Box::new(Battery {
            colormap,
            colors: state_colors,
            bar_width,
            bar_height,
            char_width,
            space,
            padding,
        })
    }
}

#[async_trait]
impl crate::bar::Bar for Battery {
    fn width(&self) -> u32 {
        self.bar_width + self.space + self.char_width + self.padding
    }

    async fn render(&self) -> String {
        let (val, state) = Ticker.battery().await;
        let bar = crate::draw::bar(val, self.colormap.map(val), self.bar_width, self.bar_height);
        let space = crate::draw::space(self.space);

        let (ch, color) = match state {
            battery::State::Unknown => ('*', self.colors.unknown),
            battery::State::Charging => ('+', self.colors.charge),
            battery::State::Discharging => ('-', self.colors.discharge),
            battery::State::Empty => ('!', self.colors.unknown),
            battery::State::Full => (' ', self.colors.charge),
            battery::State::__Nonexhaustive => ('*', self.colors.unknown),
        };

        format!("{}{}^fg({}){}", bar, space, color, ch)
    }

    fn box_clone(&self) -> crate::bar::DynBar {
        Box::new(self.clone())
    }
}
