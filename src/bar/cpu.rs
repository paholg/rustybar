use crate::ticker::Ticker;
use async_trait::async_trait;
use itertools::Itertools;

/// A statusbar for testing colormaps.
#[derive(Clone, Debug)]
pub struct Cpu {
    colormap: crate::ColorMap,
    bar_width: u32,
    bar_height: u32,
    space: u32,
    width: u32,
}

impl Cpu {
    pub async fn new(
        colormap: crate::ColorMap,
        bar_width: u32,
        bar_height: u32,
        space: u32,
        padding: u32,
    ) -> Box<Cpu> {
        let num_cores = Ticker.cpus().await.len() as u32;

        Box::new(Cpu {
            colormap,
            bar_width,
            bar_height,
            space,
            width: num_cores * bar_width + (num_cores - 1) * space + padding,
        })
    }
}

#[async_trait]
impl crate::bar::Bar for Cpu {
    fn width(&self) -> u32 {
        self.width
    }

    async fn render(&self) -> String {
        Ticker
            .cpus()
            .await
            .into_iter()
            .map(|cpu| {
                crate::draw::bar(cpu, self.colormap.map(cpu), self.bar_width, self.bar_height)
            })
            .join(&crate::draw::space(self.space))
    }

    fn box_clone(&self) -> crate::bar::DynBar {
        Box::new(self.clone())
    }
}
