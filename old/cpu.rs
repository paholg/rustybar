use std::sync::Arc;

use crate::{producer::SingleQueue, util::draw};

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

impl crate::bar::Bar for Cpu {
    type Data = crate::producer::SystemInfo;

    fn width(&self) -> u32 {
        self.min_max_width * 2 + self.avg_width + self.space * 2 + self.padding
    }

    fn render(&self, data: &Self::Data) -> String {
        let avg = data.global_cpu;
        let (min, max) = data
            .cpus
            .iter()
            .fold((f32::MAX, f32::MIN), |(min, max), &cpu| {
                (min.min(cpu), max.max(cpu))
            });

        let mut result = draw::bar(
            min,
            self.colormap.map(min),
            self.min_max_width,
            self.bar_height,
        );
        result.push_str(&draw::space(self.space));
        result.push_str(&draw::bar(
            avg,
            self.colormap.map(avg),
            self.avg_width,
            self.bar_height,
        ));
        result.push_str(&draw::space(self.space));
        result.push_str(&draw::bar(
            max,
            self.colormap.map(max),
            self.min_max_width,
            self.bar_height,
        ));

        result
    }

    fn data_queue(&self) -> SingleQueue<Arc<Self::Data>> {
        crate::producer::SYSTEM.clone()
    }
}
