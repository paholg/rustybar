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
use std::old_io::{File, timer, pipe};
use std::old_io::fs::PathExtensions;
use std::time::Duration;
use std::str::FromStr;

use self::time::{Timespec, get_time};

/// A statusbar for cpu information. All data is gathered from /proc/stat and /proc/cpuinfo.
pub struct CpuBar {
    // Gives a mapping from the order the OS lists the processors to the order we want
    // to display them. We want to group hyperthreads on a single core, but they tend
    // not to be listed in the appropriate order.
    proc_map: Vec<uint>,
    cmap: ColorMap,
    procs_per_core: uint,
    num_cores: uint,
    pub width: uint,
    pub space: uint,
    pub height: uint,
    lspace: uint,
}

impl CpuBar {
    pub fn new() -> CpuBar {
        CpuBar {
            proc_map: Vec::new(),
            cmap: ColorMap::new(),
            num_cores: 0,
            procs_per_core: 0,
            width: 20,
            space: 8,
            height: 10,
            lspace: 0,
        }
    }
}

impl StatusBar for CpuBar {
    fn initialize(&mut self, char_width: uint) {
        // just so it doesn't warn us about char_width being unused
        char_width + 1;
        if self.proc_map.len() > 0 {
            panic!("Tried to initialize CpuBar, but it was already initialized.");
        }
        let info_path = Path::new("/proc/cpuinfo");
        if !info_path.is_file() {
            panic!("The file {} does not exist. You cannot use the cpu bar without it. Are you sure you're running GNU/Linux?", info_path.display());
        }
        let cpu_info = File::open(&info_path).read_to_string().unwrap();

        // want to first get number of cores and processors
        let re_core = regex!(r"(?s)cpu\scores\s:\s(\d+).*?siblings\s:\s(\d+)");
        let cap = re_core.captures_iter(cpu_info.as_slice()).nth(0).unwrap();
        self.num_cores = FromStr::from_str(cap.at(1).unwrap()).unwrap();
        let procs: uint = FromStr::from_str(cap.at(2).unwrap()).unwrap();
        if procs == 0 { panic!("Something went wrong with finding number of processors."); }
        self.procs_per_core = procs/self.num_cores;
        // fixme: there should be a better way to set length
        self.proc_map.reserve_exact(procs);
        for _ in [..procs].iter() {
            self.proc_map.push(0);
        }
        // now we want to get the map from processor number to display order
        // note: I have the .*? after core id because \s:\s doesn't match for some reason.
        let re_map = regex!(r"(?s)processor\s:\s(\d+).*?core id.*?(\d+)");
        let mut proc_counter: Vec<uint> = Vec::new();
        //fixme: there should be a better way to initialize this
        proc_counter.reserve_exact(self.num_cores);
        for _ in [..self.num_cores].iter() {
            proc_counter.push(0);
        }
        for cap in re_map.captures_iter(cpu_info.as_slice()) {
            let proc_id: uint = FromStr::from_str(cap.at(1).unwrap()).unwrap();
            let core_id: uint = FromStr::from_str(cap.at(2).unwrap()).unwrap();
            self.proc_map[proc_id] = core_id*self.procs_per_core + proc_counter[core_id];
            proc_counter[core_id] += 1;
        }
    }

    fn run(&self, mut stream: Box<pipe::PipeStream>) {
        let path = Path::new("/proc/stat");
        if !path.is_file() {
            panic!("The file {} does not exist. You cannot use the cpu bar without it. Are you sure you're running GNU/Linux?", path.display());
        }
        let re = regex!(r"cpu(\d+)\s(\d+\s){3}(\d+)");
        // fixme: there should be a better way to initialize this to an array of zeros
        // used to be Vec::from_elem()
        let mut old_idles: Vec<uint> = Vec::with_capacity(self.proc_map.len());
        for _ in [..self.proc_map.len()].iter() {
            old_idles.push(0);
        }
        // setting old_idles (this is redundant code and should be removed for a better solution)
        let info = File::open(&path).read_to_string().unwrap();
        for cap in re.captures_iter(info.as_slice()) {
            let proc_id: uint = FromStr::from_str(cap.at(1).unwrap()).unwrap();
            let idle: uint = FromStr::from_str(cap.at(3).unwrap()).unwrap();
            old_idles[self.proc_map[proc_id]] = idle;
        }
        // -------------------
        let mut last: Timespec = get_time();
        loop {
            timer::sleep(Duration::seconds(1));
            // fixme: there should be a better way to initialize this to an array of zeros
            // used to be Vec::from_elem()
            let mut idles: Vec<uint> = Vec::with_capacity(self.proc_map.len());
            for _ in [..self.proc_map.len()].iter() {
                idles.push(0);
            }
            let info = File::open(&path).read_to_string().unwrap();
            for cap in re.captures_iter(info.as_slice()) {
                let proc_id: uint = FromStr::from_str(cap.at(1).unwrap()).unwrap();
                let idle: uint = FromStr::from_str(cap.at(3).unwrap()).unwrap();
                idles[self.proc_map[proc_id]] = idle;
            }
            let now = get_time();
            let dt = now - last;
            last = now;
            write_space(&mut *stream, self.lspace);
            write_sep(&mut *stream, 2*self.height);
            write_space(&mut *stream, self.space);
            for i in 0..idles.len() {
                let usage = 1.0 - ((idles[i] - old_idles[i]) as f32)/(dt.num_nanoseconds().unwrap() as f32)*1e7;
                let val = if usage > 1.0 { 1.0 }
                          else if usage < 0.0 { 0.0 }
                          else { usage };
                let color =  self.cmap.map((val*100.0) as u8);
                write_one_bar(&mut *stream, val, color, self.width, self.height);
                write_space(&mut *stream, self.space);
                if (i-1) % self.procs_per_core == 0 {
                    write_sep(&mut *stream, 2*self.height);
                    if i < idles.len()-1 {
                        write_space(&mut *stream, self.space);
                    }
                }
            }
            match stream.write_str("\n") {
                Err(msg) => println!("Trouble writing to cpu bar: {}", msg),
                Ok(_) => (),
            }
            old_idles = idles;
        }
    }

    fn set_colormap(&mut self, cmap: Box<ColorMap>) {
        self.cmap = *cmap;
    }

    fn len(&self) -> uint {
        self.lspace + (self.width + self.space)*self.procs_per_core*self.num_cores +
            (self.space + 2)*(self.num_cores + 1) - self.space
    }
    fn get_lspace(&self) -> uint { self.lspace }
    fn set_lspace(&mut self, lspace: uint) { self.lspace = lspace }
    fn set_width(&mut self, width: uint) { self.width = width }
    fn set_height(&mut self, height: uint) { self.height = height }
}
