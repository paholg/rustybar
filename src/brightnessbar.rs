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

extern crate libc;

use statusbar::*;
use colormap::ColorMap;
use std::old_io::fs::PathExtensions;
use std::old_io::{File, timer, pipe, fs};
use std::time::Duration;
use std::str::FromStr;
use inotify::ffi;
use std::ffi::AsOsStr;

// use std::ffi::CString;

/// A statusbar for brightness information. Uses information from /sys/class/backlight/
pub struct BrightnessBar {
    path_cur: Path,
    max: f32,
    cmap: ColorMap,
    pub width: uint,
    pub height: uint,
    pub lspace: uint,
}

impl BrightnessBar {
    pub fn new() -> BrightnessBar {
        BrightnessBar {
            path_cur: Path::new("/"),
            max: 0.,
            cmap: ColorMap::new(),
            width: 30,
            height: 10,
            lspace: 0,
        }
    }
}

impl StatusBar for BrightnessBar {
    fn initialize(&mut self, char_width: uint) {
        // just so it doesn't warn us about char_width being unused
        char_width + 1;
        let path = Path::new("/sys/class/backlight");
        if path.is_dir() {
            let dirs = fs::readdir(&path).unwrap();
            if dirs.len() > 1 {
                println!("You have multiple backlight directories. They are:");
                for dir in dirs.iter() { println!("\t{}", dir.display()); }
                // fixme: add this to config
                println!("I am going to use the first one. To use another, edit the configuration file (Not yet enabled, please report this and I will enable it).");
            }
            self.path_cur = dirs[0].join("brightness");
            assert!(self.path_cur.exists(), "The file {} doesn't exists. I can't make a brightness bar. Please report this.", self.path_cur.display());
            let path_max = dirs[0].join("max_brightness");
            assert!(path_max.exists(), "The file {} doesn't exists. I can't make a brightness bar. Please report this.", path_max.display());
            let max_string = File::open(&path_max).read_to_string().unwrap();
            self.max = FromStr::from_str(max_string.trim().as_slice()).unwrap();
            //println!("max: {}", self.max);
        }
        else {
            panic!("The directory: {} doesn't exist. The brightness bar won't work.");
        }
    }
    fn run(&self, mut stream: Box<pipe::PipeStream>) {
        let mut fd: i32;
        //let mut wd: i32;

        unsafe {
            fd = ffi::inotify_init();
            assert!(fd >= 0, "Invalid file descriptor in brightness bar.");
            // wd = ffi::inotify_add_watch(fd, self.path_cur.to_c_str().as_ptr(),
            //                                      ffi::IN_MODIFY);
            ffi::inotify_add_watch(fd, self.path_cur.as_str().unwrap().as_ptr() as *const i8,
                                   ffi::IN_MODIFY);
        }
        let mut buffer = [0u8; 1024];
        loop {
            let cur_string = File::open(&self.path_cur).read_to_string().unwrap();
            let cur: f32 = FromStr::from_str(cur_string.trim().as_slice()).unwrap();
            let val = cur/self.max;
            write_space(&mut *stream, self.lspace);
            match stream.write_str(
                "^ca(4,xdotool key XF86MonBrightnessUp)^ca(5,xdotool key XF86MonBrightnessDown)") {
                Err(msg) => println!("Trouble writing to brightness bar: {}", msg),
                Ok(_) => (),
            }
            write_one_bar(&mut *stream, val, self.cmap.map((val*100.) as u8), self.width, self.height);
            match stream.write_str("^ca()^ca()\n") {
                Err(msg) => println!("Trouble writing to brightness bar: {}", msg),
                Ok(_) => (),
            }
            timer::sleep(Duration::milliseconds(30));
            unsafe {
                ffi::read(fd, buffer.as_mut_ptr() as *mut libc::c_void,
                          buffer.len() as u64);
            }
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
