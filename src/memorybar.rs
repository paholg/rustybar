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
use std::old_io::{File};
use std::old_io::fs::PathExtensions;
use std::old_io::timer;
use std::time::Duration;
use std::old_io::pipe;

/// A statusbar for cpu information. All data is gathered from /proc/stat and /proc/cpuinfo.
pub struct MemoryBar {
    cmap: ColorMap,
    pub width: uint,
    pub height: uint,
    lspace: uint,
}

impl MemoryBar {
    pub fn new() -> MemoryBar {
        MemoryBar {
            width: 20,
            height: 10,
            lspace: 0,
            cmap: ColorMap::new(),
        }
    }
}

impl StatusBar for MemoryBar {
    fn initialize(&mut self, char_width: uint) {
        // just so it doesn't warn us about char_width being unused
        char_width + 1;
    }

    fn run(&self, mut stream: Box<pipe::PipeStream>) {
        let path = Path::new("/proc/meminfo");
        if !path.is_file() {
            panic!("The file {} does not exist. You cannot use the cpu bar without it. Are you sure you're running GNU/Linux?", path.display());
        }
        let re_tot = regex!(r"MemTotal.*?(\d+)");
        let re_free = regex!(r"MemFree.*?(\d+)");
        let re_buffers = regex!(r"Buffers.*?(\d+)");
        let re_cached = regex!(r"Cached.*?(\d+)");
        let info = File::open(&path).read_to_string().unwrap();
        // fixme: need to redo with new syntax
        let total: f32 = 1.0;//re_tot.captures_iter(info.as_slice().from_str().nth(0).unwrap().at(1)).unwrap();
        // -------------------
        loop {
            let info = File::open(&path).read_to_string().unwrap();
            let free: f32 = 0.0; //re_free.captures_iter(info.as_slice().from_str().nth(0).unwrap().at(1)).unwrap();
            let buffers: f32 = 0.0; //re_buffers.captures_iter(info.as_slice().from_str().nth(0).unwrap().at(1)).unwrap();
            let cached: f32 = 0.0; //re_cached.captures_iter(info.as_slice().from_str().nth(0).unwrap().at(1)).unwrap();
            let val = (total - free - buffers - cached)/total;

            write_space(&mut *stream, self.lspace);
            write_one_bar(&mut *stream, val, self.cmap.map((val*100.) as u8), self.width, self.height);
            match stream.write_str("\n") {
                Err(msg) => println!("Trouble writing to memory bar: {}", msg),
                Ok(_) => (),
            }

            timer::sleep(Duration::seconds(1));
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
