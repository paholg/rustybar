use async_trait::async_trait;

/// A statusbar for testing colormaps.
#[derive(Clone, Debug)]
pub struct Clock {
    color: String,
    format: String,
    char_width: u32,
    length: u32,
}

impl Clock {
    pub fn new(
        color: impl Into<String>,
        format: impl Into<String>,
        char_width: u32,
        length: u32,
    ) -> Clock {
        Clock {
            color: color.into(),
            format: format.into(),
            char_width: char_width,
            length: length,
        }
    }
}

#[async_trait]
impl crate::bar::Bar for Clock {
    fn width(&self) -> u32 {
        self.length
    }

    async fn render(&self) -> String {
        let text = crate::state::read().await.time().format(&self.format);
        format!("^fg({}){}", self.color, text)
    }

    fn box_clone(&self) -> crate::bar::DynBar {
        Box::new(self.clone())
    }
}
