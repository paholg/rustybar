use crate::ticker::Ticker;
use async_trait::async_trait;

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

#[async_trait]
impl crate::bar::Bar for Network {
    fn width(&self) -> u32 {
        self.width
    }

    async fn render(&self) -> String {
        let duration = Ticker.tick_duration().await;

        let recieved = Ticker.bytes_recieved().await;
        let transmitted = Ticker.bytes_transmitted().await;

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

    fn box_clone(&self) -> crate::bar::DynBar {
        Box::new(self.clone())
    }
}
