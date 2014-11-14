extern crate time;

//use std::fmt;
//use regex::Regex;
use statusbar::*;
use colormap::ColorMap;
// use std::io::{File};
// use std::io::fs::PathExtensions;
// use std::io::timer;
// use std::time::Duration;
use std::io::pipe;
use std::io;
// use self::time::{Timespec, get_time};

/// A statusbar for cpu information. All data is gathered from /proc/stat and /proc/cpuinfo.
pub struct StdinBar {
    pub length: uint,
    lspace: uint,
}

impl StdinBar {
    pub fn new() -> StdinBar {
        StdinBar {
            length: 30,
            lspace: 0,
        }
    }
}

impl StatusBar for StdinBar {
    fn initialize(&mut self, char_width: uint) {
        // just so it doesn't warn us about char_width being unused
        char_width + 1;
    }

    fn run(&self, mut stream: Box<pipe::PipeStream>) {
        // -------------------
        let mut stdin = io::stdio::stdin();
        loop {
            write_space(&mut *stream, self.lspace);
            let line = stdin.read_line();
            match stream.write_str(line.unwrap().as_slice()) {
                Err(msg) => println!("Trouble writing to stdin bar: {}", msg),
                Ok(_) => (),
            };
        }
    }

    fn set_colormap(&mut self, cmap: Box<ColorMap>) {
        cmap.map(1);
    }

    fn len(&self) -> uint {
        self.length
    }
    fn get_lspace(&self) -> uint { self.lspace }
    fn set_lspace(&mut self, lspace: uint) { self.lspace = lspace }
    fn set_width(&mut self, width: uint) { width + 1; }
    fn set_height(&mut self, height: uint) { height + 1; }
}
