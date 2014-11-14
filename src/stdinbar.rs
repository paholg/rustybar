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

extern crate time;

use statusbar::*;
use colormap::ColorMap;
use std::io::pipe;
use std::io;

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
