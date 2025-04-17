use std::sync::Arc;

use crate::producer::SingleQueue;

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

impl crate::bar::Bar for Memory {
    type Data = crate::producer::SystemInfo;

    fn width(&self) -> u32 {
        self.width
    }

    fn render(&self, data: &Self::Data) -> String {
        let memory = data.available_memory;

        format!(
            "^fg({}){}",
            self.colormap.map(memory as f32),
            crate::format_bytes(memory)
        )
    }

    fn data_queue(&self) -> SingleQueue<Arc<Self::Data>> {
        crate::producer::SYSTEM.clone()
    }
}
