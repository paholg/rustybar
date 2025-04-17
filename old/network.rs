use std::sync::Arc;

use crate::producer::SingleQueue;

#[derive(Clone, Debug)]
pub struct Network {
    colormap: crate::ColorMap,
    width: u32,
}

impl Network {
    pub async fn new(colormap: crate::ColorMap, padding: u32) -> Box<Network> {
        let char_width = crate::config::get().await.font.width;

        Box::new(Network {
            colormap,
            width: char_width * 13 + padding,
        })
    }
}

impl crate::bar::Bar for Network {
    type Data = crate::producer::SystemInfo;

    fn width(&self) -> u32 {
        self.width
    }

    fn render(&self, data: &Self::Data) -> String {
        let duration = data.tick_duration;

        let recieved = data.bytes_received;
        let transmitted = data.bytes_transmitted;

        let rps = recieved as f32 / duration.as_secs_f32();
        let tps = transmitted as f32 / duration.as_secs_f32();

        format!(
            "^fg({}){} ^fg({}){}",
            self.colormap.map(rps),
            crate::format_bytes(rps as u64),
            self.colormap.map(tps),
            crate::format_bytes(tps as u64),
        )
    }

    fn data_queue(&self) -> SingleQueue<Arc<Self::Data>> {
        crate::producer::SYSTEM.clone()
    }
}
