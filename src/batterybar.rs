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

//extern crate libc;

use statusbar::*;
use colormap::ColorMap;
use std::old_io::{File, timer, pipe};
use std::old_io::fs::PathExtensions;
use std::time::Duration;
use std::str::FromStr;


/// A statusbar for battery information. All data is gathered from '/sys/class/power_supply/BAT#/'
pub struct BatteryBar {
    bat_num: uint,
    path_capacity: Path,
    path_status: Path,
    cmap: ColorMap,
    // the size of bars and spaces between them
    char_width: uint,
    pub width: uint,
    pub space: uint,
    pub height: uint,
    pub lspace: uint,
}

impl BatteryBar {
    pub fn new() -> BatteryBar {
        BatteryBar {
            bat_num: 0,
            path_capacity: Path::new("/"),
            path_status: Path::new("/"),
            cmap: ColorMap::new(),
            char_width: 0,
            width: 30,
            space: 8,
            height: 10,
            lspace: 0,
        }
    }
    /// sets the battery number to use
    pub fn set_bat_num(&mut self, num: uint) {
        self.bat_num = num;
        let path = Path::new(format!("/sys/class/power_supply/BAT{}/", num).as_slice());
        if !path.exists() {
            panic!("The selected battery directory:\n\t{}\ndoes not exist and is needed for the battery bar. Perhaps you meant a different battery number or perhaps it isn't there. Go ahead and report this with the output of \"ls /sys/class/power_supply/", path.display());
        }
        self.path_capacity = Path::new(format!("/sys/class/power_supply/BAT{}/capacity", num).as_slice());
        if !self.path_capacity.exists() {
            panic!("The file:\n\t{}\ndoes not exist. It almost certainly should and is needed for the battery bar.", self.path_capacity.display());
        }
        self.path_status = Path::new(format!("/sys/class/power_supply/BAT{}/status", num).as_slice());
        if !self.path_status.exists() {
            panic!("The file:\n\t{}\ndoes not exist. It almost certainly should and is needed for the battery bar.", self.path_status.display());
        }
    }
}

impl StatusBar for BatteryBar {
    fn initialize(&mut self, char_width: uint) {
        self.char_width = char_width;
        // In case set_bat_num() hasn't been run. It doesn't hurt to run it twice.
        let num = self.bat_num;
        self.set_bat_num(num);
    }
    fn run(&self, mut stream: Box<pipe::PipeStream>) {
        loop {
            let cap_string = File::open(&self.path_capacity).read_to_string().unwrap();
            let capacity: u8 = FromStr::from_str(cap_string.trim().as_slice()).unwrap();

            write_space(&mut *stream, self.lspace);
            write_one_bar(&mut *stream, (capacity as f32)/100., self.cmap.map(capacity), self.width, self.height);
            write_space(&mut *stream, self.space);
            let status_string = File::open(&self.path_status).read_to_string().unwrap();
            let status = match status_string.trim().as_slice() {
                "Charging" => "^fg(#00ff00)+\n",
                "Discharging" => "^fg(#ff0000)-\n",
                "Full" => " \n",
                _ => "^bg(#ff0000)^fg(#000000)*^bg()\n",
            };
            match stream.write_str(status) {
                Err(msg) => println!("Trouble writing to battery bar: {}", msg),
                Ok(_) => (),
            }
            timer::sleep(Duration::seconds(1));
        }
    }
    fn set_colormap(&mut self, cmap: Box<ColorMap>) {
        self.cmap = *cmap;
    }
    fn len(&self) -> uint {
        self.lspace + self.width + self.space + self.char_width
    }
    fn get_lspace(&self) -> uint { self.lspace }
    fn set_lspace(&mut self, lspace: uint) { self.lspace = lspace }
    fn set_width(&mut self, width: uint) { self.width = width }
    fn set_height(&mut self, height: uint) { self.height = height }
}
