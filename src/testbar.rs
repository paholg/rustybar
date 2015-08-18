// rustybar - a lightweight but featureful status bar
// Copyright (C) 2014  Paho Lurie-Gregg

// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc.,
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

use statusbar::*;
use colormap::ColorMap;
use std::old_io::{timer, pipe};
use std::time::Duration;

/// A statusbar for testing colormaps.
pub struct TestBar {
    cmap: ColorMap,
    pub width: uint,
    pub height: uint,
    pub lspace: uint,
}

impl TestBar {
    pub fn new() -> TestBar {
        TestBar {
            cmap: ColorMap::new(),
            width: 30,
            height: 10,
            lspace: 0,
        }
    }
}

impl StatusBar for TestBar {
    fn initialize(&mut self, char_width: uint) {
        // just so it doesn't warn us about char_width being unused
        char_width + 1;
    }
    fn run(&self, mut stream: Box<pipe::PipeStream>) {
        let each_wid = if self.width > 100 { self.width/100 } else { 1 };
        let num_bars = if self.width > 100 { 100 } else { self.width };
        loop {
            // if the bar is more than 100 pixels, we'll just use an increment of 100
            // (this is to prevent buffer overflow by just drawing hundreds of single pixel rectangles)
            write_space(&mut *stream, self.lspace);
            if self.width > 100 {
                let space = (self.width % 100)/2;
                write_space(&mut *stream, space);
            }
            for i in 0..num_bars {
                let val = ((i as f32)/(num_bars as f32)*100.) as u8;
                let string = format!("^fg({:?})^r({}x{})", self.cmap.map(val), each_wid, self.height);
                match stream.write_str(string.as_slice()) {
                    Err(msg) => println!("Trouble writing to test bar: {}", msg),
                    Ok(_) => (),
                };
            }
            if self.width > 100 {
                let space = ((self.width % 100) + 1)/2; //the +1 accounts for rounding
                write_space(&mut *stream, space);
            }
            match stream.write_str("\n") {
                Err(msg) => println!("Trouble writing to test bar: {}", msg),
                Ok(_) => (),
            };
            timer::sleep(Duration::seconds(100));
        }
    }
    fn set_colormap(&mut self, cmap: Box<ColorMap>) {
        self.cmap = *cmap;
    }
    fn len(&self) -> uint {
        self.lspace + self.width
    }
    fn get_lspace(&self) -> uint { self.lspace }
    fn set_lspace(&mut self, lspace: uint) { self.lspace = lspace }
    fn set_width(&mut self, width: uint) { self.width = width }
    fn set_height(&mut self, height: uint) { self.height = height }
}
