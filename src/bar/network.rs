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

use colormap::ColorMap;
use statusbar::*;
use std::old_io::fs::PathExtensions;
use std::old_io::pipe;
use std::old_io::timer;
use std::old_io::File;
use std::time::Duration;

/// A statusbar for network connection information. All data is gathered from '/sys/class/net/'
pub struct NetworkBar {
    path_wifi: Path,
    path_eth: Path,
    connected_color: Color,
    disconnected_color: Color,
    char_width: uint,
    pub length: uint,
    pub lspace: uint,
}

impl NetworkBar {
    pub fn new() -> NetworkBar {
        NetworkBar {
            path_wifi: Path::new("/"),
            path_eth: Path::new("/"),
            char_width: 0,
            connected_color: Color::new(200, 200, 200),
            disconnected_color: Color::new(200, 0, 0),
            length: 40,
            lspace: 0,
        }
    }
}

// impl StatusBar for NetworkBar {
//     fn initialize(&mut self, char_width: uint) {
//         self.char_width = char_width;
//         // set paths
//     }
//     fn run(&self, mut stream: Box<pipe::PipeStream>) {
//         loop {
//             write_space(&mut *stream, self.lspace);
//             timer::sleep(Duration::seconds(1));
//         }
//     }
//     fn set_colormap(&mut self, cmap: Box<ColorMap>) {
//         self.cmap = *cmap;
//     }
//     fn len(&self) -> uint {
//         self.lspace + self.width + self.space + self.char_width
//     }
//     fn get_lspace(&self) -> uint { self.lspace }
//     fn set_lspace(&mut self, lspace: uint) { self.lspace = lspace }
//     fn set_width(&mut self, width: uint) { self.width = width }
//     fn set_height(&mut self, height: uint) { self.height = height }
// }
