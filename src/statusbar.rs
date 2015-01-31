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

use colormap::{Color, ColorMap};
use std::old_io::pipe;

// fixme: this should be settable
static TEXTCOLOR: &'static str = "#888888";

pub trait StatusBar {
    /// Find any initial information needed for the bar, then begin a thread where it
    /// updates either based on time or some other qualification
    fn initialize(&mut self, char_width: uint);
    fn run(&self, mut stream: Box<pipe::PipeStream>);
    fn set_colormap(&mut self, cmap: Box<ColorMap>);
    /// Give the length in pixels of the output string.
    fn len(&self) -> uint;
    fn get_lspace(&self) -> uint;
    fn set_lspace(&mut self, lspace: uint);
    fn set_width(&mut self, width: uint);
    fn set_height(&mut self, height: uint);
}

pub fn write_one_bar(stream: &mut pipe::PipeStream, val: f32, color: Color, width: uint, height: uint) {
    // fixme: .round()?
    let wfill = (val*(width as f32) + 0.5) as uint;
    let wempty = width - wfill;
    match write!(stream, "^fg({0:?})^r({2}x{1})^ro({3}x{1})", color, height, wfill, wempty) {
        Err(msg) => panic!("Failed to write pipe: {}", msg.desc),
        Ok(_) => (),
    };
}

pub fn write_space(stream: &mut pipe::PipeStream, width: uint) {
    match write!(stream, "^r({}x0)", width) {
        Err(msg) => panic!("Failed to write pipe: {}", msg.desc),
        Ok(_) => (),
    };
}

pub fn write_sep(stream: &mut pipe::PipeStream, height: uint) {
    match write!(stream, "^fg({})^r(2x{})", TEXTCOLOR, height) {
        Err(msg) => panic!("Failed to write pipe: {}", msg.desc),
        Ok(_) => (),
    };
}
