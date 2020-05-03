use crate::ticker::Ticker;
use async_trait::async_trait;

/// A statusbar for cpu information. All data is gathered from /proc/stat and /proc/cpuinfo.
#[derive(Clone, Debug)]
pub struct Temp {
    colormap: crate::ColorMap,
    pub width: u32,
}

impl Temp {
    pub async fn new(colormap: crate::ColorMap, padding: u32) -> Box<Temp> {
        let char_width = crate::config::get().await.font.width;

        Box::new(Temp {
            colormap,
            width: char_width * 6 + padding,
        })
    }
}

#[async_trait]
impl crate::bar::Bar for Temp {
    fn width(&self) -> u32 {
        self.width
    }

    async fn render(&self) -> String {
        let temp = Ticker.temperature().await;

        format!("^fg({}){:3.0} °C", self.colormap.map(temp), temp)
    }

    fn box_clone(&self) -> crate::bar::DynBar {
        Box::new(self.clone())
    }
}
