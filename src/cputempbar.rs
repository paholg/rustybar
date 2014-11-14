extern crate time;

//use std::fmt;
//use regex::Regex;
use statusbar::*;
use colormap::ColorMap;
use std::io::{timer, pipe, Command};
use std::string::String;
use std::time::Duration;

/// A statusbar for cpu information. All data is gathered from /proc/stat and /proc/cpuinfo.
pub struct CpuTempBar {
    cmap: ColorMap,
    pub min: f32,
    pub max: f32,
    pub width: uint,
    pub height: uint,
    lspace: uint,
}

impl CpuTempBar {
    pub fn new() -> CpuTempBar {
        CpuTempBar {
            min: 0.,
            max: 100.,
            width: 20,
            height: 10,
            lspace: 0,
            cmap: ColorMap::new(),
        }
    }
}

impl StatusBar for CpuTempBar {
    fn initialize(&mut self, char_width: uint) {
        // just so it doesn't warn us about char_width being unused
        char_width + 1;
    }

    fn run(&self, mut stream: Box<pipe::PipeStream>) {
        let re = regex!(r"(\d+\.\d+)\s*degrees.*");
        loop {
            let info = match Command::new("acpi").arg("-t").output() {
                Ok(out) => out,
                Err(msg) => panic!("Failed to run \"acpi -t\" with message: {}", msg),
            };
            if info.status.success() == false {
                println!("\"acpi -t\" returned exit signal {}.", info.status);
                println!("error: {}", String::from_utf8(info.error).unwrap());
            }
            let output = String::from_utf8(info.output).unwrap();
            let mut cap = re.captures_iter(output.as_slice());
            let t = match cap.nth(0) {
                Some(val) => val,
                None => panic!("Cpu temp bar error. Couldn't find value."),
            };
            let temp: f32 = from_str(t.at(1)).unwrap();
            let val: f32 =
                if temp > self.max {1.0}
                else if temp < self.min {0.0}
                else {(temp - self.min)/(self.max - self.min)};
            write_space(&mut *stream, self.lspace);
            write_one_bar(&mut *stream, val, self.cmap.map((val*100.) as u8), self.width, self.height);
            match stream.write_str("\n") {
                Err(msg) => println!("Trouble writing to volume bar: {}", msg),
                Ok(_) => (),
            };
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
