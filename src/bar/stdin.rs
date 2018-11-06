use failure;
use std::{io,
          io::{BufRead, Write},
          process};

use crate::bar::{Bar, WriteBar, Writer};

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
            char_width: char_width,
            buffer: String::new(),
        })
    }
}

impl Bar for Stdin {
    fn len(&self) -> u32 {
        self.length * self.char_width
    }

    fn block(&self) -> Result<(), failure::Error> {
        Ok(())
    }

    fn write(&mut self, w: &mut Writer) -> Result<(), failure::Error> {
        self.buffer.clear();
        // fixme: This locks for every read.
        io::stdin().read_line(&mut self.buffer)?;

        w.write(b"\n^tw()")?;
        w.write(self.buffer.as_bytes())?;

        Ok(())
    }
}
