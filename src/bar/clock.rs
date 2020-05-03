use crate::ticker::Ticker;
use async_trait::async_trait;

/// A statusbar for testing colormaps.
#[derive(Clone, Debug)]
pub struct Clock {
    color: String,
    format: String,
    char_width: u32,
    width: u32,
}

impl Clock {
    pub async fn new(
        color: impl Into<String>,
        format: impl Into<String>,
        num_chars: u32,
        padding: u32,
    ) -> Box<Clock> {
        let char_width = crate::config::get().await.font.width;

        Box::new(Clock {
            color: color.into(),
            format: format.into(),
            char_width,
            width: char_width * num_chars + padding,
        })
    }
}

#[async_trait]
impl crate::bar::Bar for Clock {
    fn width(&self) -> u32 {
        self.width
    }

    async fn render(&self) -> String {
        let text = Ticker.time().await.format(&self.format);
        format!("^fg({}){}", self.color, text)
    }

    fn box_clone(&self) -> crate::bar::DynBar {
        Box::new(self.clone())
    }
}
