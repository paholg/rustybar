use failure;
use std::{io, io::Write};

use crate::bar::{Bar, Writer};

#[derive(Debug, Deserialize)]
pub struct StdinConfig {
    length: u32,
}

/// A statusbar for cpu information. All data is gathered from /proc/stat and /proc/cpuinfo.
#[derive(Debug)]
pub struct Stdin {
    length: u32,
    char_width: u32,
    buffer: String,
}

impl Stdin {
    pub fn from_config(config: &StdinConfig, char_width: u32) -> Result<Stdin, failure::Error> {
        Ok(Stdin {
            length: config.length,
            char_width,
            buffer: String::new(),
        })
    }
}

impl Bar for Stdin {
    fn len(&self) -> u32 {
        self.length * self.char_width
    }

    // This bar is a bit unusual, and the blocking happens in the `write` function.
    fn block(&mut self) -> Result<(), failure::Error> {
        Ok(())
    }

    fn write(&mut self, w: &mut Writer) -> Result<(), failure::Error> {
        self.buffer.clear();
        io::stdin().read_line(&mut self.buffer)?;

        w.write_all(b"^tw()")?;
        // Skip the final new line as that is provided by `BarWithSep`
        w.write_all(self.buffer[0..self.buffer.len() - 1].as_bytes())?;

        Ok(())
    }
}
