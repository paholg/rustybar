use crate::ticker::Ticker;
use async_trait::async_trait;

#[derive(Clone, Debug)]
pub struct Memory {
    colormap: crate::ColorMap,
    width: u32,
}

impl Memory {
    pub async fn new(colormap: crate::ColorMap, padding: u32) -> Box<Memory> {
        let char_width = crate::config::get().await.font.width;

        Box::new(Memory {
            colormap,
            width: char_width * 6 + padding,
        })
    }
}

#[async_trait]
impl crate::bar::Bar for Memory {
    fn width(&self) -> u32 {
        self.width
    }

    async fn render(&self) -> String {
        let memory = Ticker.free_memory().await;

        println!("Rendering memory bar");

        format!(
            "^fg({}){}",
            self.colormap.map(memory as f32),
            crate::format_bytes(memory)
        )
    }

    fn box_clone(&self) -> crate::bar::DynBar {
        Box::new(self.clone())
    }
}
