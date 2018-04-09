use std::{process, thread, time, io::Write};

use bar::{write_space, StatusBar};

use failure;
use chrono;

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
    lspace: u32,
    rspace: u32,
}

impl Clock {
    pub fn from_config(config: &ClockConfig, char_width: u32) -> Result<Clock, failure::Error> {
        let mut bar = Clock {
            color: config.color.clone(),
            format: config.format.clone(),
            char_width: char_width,
            length: 0,
            lspace: 0,
            rspace: 0,
        };

        let now = chrono::Local::now();
        let text = now.format(&config.format);
        let string = format!("{}", text);
        bar.length = char_width * string.trim().len() as u32;
        Ok(bar)
    }
}

impl StatusBar for Clock {
    fn run(&self, w: &mut process::ChildStdin) -> Result<(), failure::Error> {
        loop {
            let now = chrono::Local::now();
            let text = now.format(&self.format);
            write_space(w, self.lspace)?;
            write!(w, "^fg({}){}\n", self.color, text)?;
            write_space(w, self.rspace)?;
            thread::sleep(time::Duration::from_secs(1));
        }
    }

    fn len(&self) -> u32 {
        self.lspace + self.length + self.rspace
    }

    fn get_lspace(&self) -> u32 {
        self.lspace
    }

    fn set_lspace(&mut self, lspace: u32) {
        self.lspace = lspace
    }

    fn set_rspace(&mut self, rspace: u32) {
        self.rspace = rspace
    }
}
