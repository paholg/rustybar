use crate::producer::SingleQueue;
use crate::Color;
use std::sync::Arc;

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

impl crate::bar::Bar for Battery {
    type Data = (f32, starship_battery::State);

    fn width(&self) -> u32 {
        self.bar_width + self.space + self.char_width + self.padding
    }

    fn render(&self, data: &Self::Data) -> String {
        let (val, state) = *data;
        let bar =
            crate::util::draw::bar(val, self.colormap.map(val), self.bar_width, self.bar_height);
        let space = crate::util::draw::space(self.space);

        let (ch, color) = match state {
            starship_battery::State::Unknown => ('*', self.colors.unknown),
            starship_battery::State::Charging => ('+', self.colors.charge),
            starship_battery::State::Discharging => ('-', self.colors.discharge),
            starship_battery::State::Empty => ('!', self.colors.unknown),
            starship_battery::State::Full => (' ', self.colors.charge),
        };

        format!("{}{}^fg({}){}", bar, space, color, ch)
    }

    fn data_queue(&self) -> SingleQueue<Arc<Self::Data>> {
        crate::producer::BATTERY.clone()
    }
}
