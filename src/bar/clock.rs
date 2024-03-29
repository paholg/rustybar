use chrono::{DateTime, Local};
use std::sync::Arc;

use crate::{producer::SingleQueue, Color};

/// A statusbar for testing colormaps.
#[derive(Clone, Debug)]
pub struct Clock {
    color: Color,
    format: String,
    width: u32,
}

impl Clock {
    pub async fn new(
        color: impl Into<Color>,
        format: impl Into<String>,
        num_chars: u32,
        padding: u32,
    ) -> Box<Clock> {
        let char_width = crate::config::get().await.font.width;

        Box::new(Clock {
            color: color.into(),
            format: format.into(),
            width: char_width * num_chars + padding,
        })
    }
}

impl crate::bar::Bar for Clock {
    type Data = DateTime<Local>;

    fn width(&self) -> u32 {
        self.width
    }

    fn render(&self, data: &Self::Data) -> String {
        let text = data.format(&self.format);
        format!("^fg({}){}", self.color, text)
    }

    fn data_queue(&self) -> SingleQueue<Arc<Self::Data>> {
        crate::producer::CLOCK.clone()
    }
}
