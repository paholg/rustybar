use std::sync::Arc;

use crate::producer::SingleQueue;

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

impl crate::bar::Bar for Temp {
    type Data = crate::producer::SystemInfo;

    fn width(&self) -> u32 {
        self.width
    }

    fn render(&self, data: &Self::Data) -> String {
        let temp = data.temperature;

        format!("^fg({}){:3.0} °C", self.colormap.map(temp), temp)
    }

    fn data_queue(&self) -> SingleQueue<Arc<Self::Data>> {
        crate::producer::SYSTEM.clone()
    }
}
