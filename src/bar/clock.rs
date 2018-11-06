use std::io::Write;

use crate::bar::{Bar, Writer};

use chrono;
use failure;

#[derive(Debug, Deserialize)]
pub struct ClockConfig {
    pub color: String,
    pub format: String,
}

/// A statusbar for testing colormaps.
#[derive(Debug, Clone)]
pub struct Clock {
    color: String,
    format: String,
    char_width: u32,
    length: u32,
}

impl Clock {
    pub fn from_config(config: &ClockConfig, char_width: u32) -> Result<Clock, failure::Error> {
        let mut clock = Clock {
            color: config.color.clone(),
            format: config.format.clone(),
            char_width,
            length: 0,
        };

        let now = chrono::Local::now();
        let text = now.format(&config.format);
        let string = format!("{}", text);
        clock.length = char_width * string.trim().len() as u32;
        Ok(clock)
    }
}

impl Bar for Clock {
    fn len(&self) -> u32 {
        self.length
    }

    fn write(&mut self, w: &mut Writer) -> Result<(), failure::Error> {
        let now = chrono::Local::now();
        let text = now.format(&self.format);
        write!(w, "^fg({}){}", self.color, text)?;
        Ok(())
    }
}
