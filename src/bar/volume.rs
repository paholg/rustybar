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
use statusbar::*;
use std::old_io::{pipe, timer, Command};
use std::str::FromStr;
use std::string::String;
use std::time::Duration;

/// A statusbar for volume information.
pub struct VolumeBar {
    pub card: uint,
    pub channel: String,
    pub mute_color: Color,
    cmap: ColorMap,
    pub width: uint,
    pub height: uint,
    pub lspace: uint,
}

impl VolumeBar {
    pub fn new() -> VolumeBar {
        VolumeBar {
            card: 0,
            channel: "Master".to_string(),
            mute_color: Color::new(130, 0, 200),
            cmap: ColorMap::new(),
            width: 30,
            height: 10,
            lspace: 0,
        }
    }
}

// impl StatusBar for VolumeBar {
//     fn initialize(&mut self, char_width: uint) {
//         // just so it doesn't warn us about char_width being unused
//         char_width + 1;
//     }
//     fn run(&self, mut stream: Box<pipe::PipeStream>) {
//         let re_vol = regex::Regex::new(r"Playback.*\[(\d+)%\]").unwrap();
//         let re_mute = regex::Regex::new(r"Playback.*\[(on|off)\]\s*$").unwrap();
//         loop {
//             let info = match Command::new("amixer")
//                 .args(&[
//                     "-c",
//                     self.card.to_string().as_slice(),
//                     "sget",
//                     self.channel.as_slice(),
//                 ])
//                 .output()
//             {
//                 Ok(out) => out,
//                 Err(msg) => panic!("Failed to run amixer with message: {}", msg),
//             };
//             if info.status.success() == false {
//                 println!("amixer returned exit signal {}.", info.status);
//                 println!("error: {}", String::from_utf8(info.error).unwrap());
//             }
//             let output = String::from_utf8(info.output).unwrap();
//             let mut cap = re_vol.captures_iter(output.as_slice());
//             let v = match cap.nth(0) {
//                 Some(val) => val,
//                 None => panic!("Volume bar error. Couldn't find value."),
//             };
//             let val: u8 = FromStr::from_str(v.at(1).unwrap()).unwrap();
//             let mut cap = re_mute.captures_iter(output.as_slice());
//             let state = match cap.nth(0) {
//                 Some(val) => val,
//                 None => panic!("Volume bar error. Couldn't find mute state."),
//             };
//             let color = match state.at(1).unwrap() {
//                 "on" => self.cmap.map(val),
//                 "off" => self.mute_color.clone(),
//                 _ => panic!("This can't happen"),
//             };
//             write_space(&mut *stream, self.lspace);
//             match stream.write_str("^ca(1,xdotool key XF86AudioMute)^ca(4,xdotool key XF86AudioRaiseVolume)^ca(5,xdotool key XF86AudioLowerVolume)") {
//                 Err(msg) => println!("Trouble writing to volume bar: {}", msg),
//                 Ok(_) => (),
//             };
//             write_one_bar(
//                 &mut *stream,
//                 (val as f32) / 100.,
//                 color,
//                 self.width,
//                 self.height,
//             );
//             match stream.write_str("^ca()^ca()^ca()\n") {
//                 Err(msg) => println!("Trouble writing to volume bar: {}", msg),
//                 Ok(_) => (),
//             };
//             timer::sleep(Duration::seconds(1));
//         }
//     }
//     fn set_colormap(&mut self, cmap: Box<ColorMap>) {
//         self.cmap = *cmap;
//     }
//     fn len(&self) -> uint {
//         self.lspace + self.width
//     }
//     fn get_lspace(&self) -> uint {
//         self.lspace
//     }
//     fn set_lspace(&mut self, lspace: uint) {
//         self.lspace = lspace
//     }
//     fn set_width(&mut self, width: uint) {
//         self.width = width
//     }
//     fn set_height(&mut self, height: uint) {
//         self.height = height
//     }
// }
