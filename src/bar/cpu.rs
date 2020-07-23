use crate::ticker::Ticker;
use async_trait::async_trait;

/// A statusbar for cpu use.
#[derive(Clone, Debug)]
pub struct Cpu {
    colormap: crate::ColorMap,
    min_max_width: u32,
    avg_width: u32,
    bar_height: u32,
    space: u32,
    padding: u32,
}

impl Cpu {
    pub async fn new(
        colormap: crate::ColorMap,
        min_max_width: u32,
        avg_width: u32,
        bar_height: u32,
        space: u32,
        padding: u32,
    ) -> Box<Cpu> {
        Box::new(Cpu {
            colormap,
            min_max_width,
            avg_width,
            bar_height,
            space,
            padding,
        })
    }
}

#[async_trait]
impl crate::bar::Bar for Cpu {
    fn width(&self) -> u32 {
        self.min_max_width * 2 + self.avg_width + self.space * 2 + self.padding
    }

    async fn render(&self) -> String {
        let avg = Ticker.global_cpu().await;
        let cpus = Ticker.cpus().await;
        let (min, max) = cpus
            .into_iter()
            .fold((f32::MAX, f32::MIN), |(min, max), cpu| {
                (min.min(cpu), max.max(cpu))
            });

        let mut result = crate::draw::bar(
            min,
            self.colormap.map(min),
            self.min_max_width,
            self.bar_height,
        );
        result.push_str(&crate::draw::space(self.space));
        result.push_str(&crate::draw::bar(
            avg,
            self.colormap.map(avg),
            self.avg_width,
            self.bar_height,
        ));
        result.push_str(&crate::draw::space(self.space));
        result.push_str(&crate::draw::bar(
            max,
            self.colormap.map(max),
            self.min_max_width,
            self.bar_height,
        ));

        result
    }

    fn box_clone(&self) -> crate::bar::DynBar {
        Box::new(self.clone())
    }
}
