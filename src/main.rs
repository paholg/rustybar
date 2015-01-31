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

#![feature(int_uint)]
#![feature(plugin)]

#[macro_use(regex)]
#[plugin] extern crate regex_macros;
extern crate regex;
extern crate inotify;
extern crate time;
extern crate core;
//extern crate xlib;
extern crate serialize;

// #![feature(phase)]
// #[phase(plugin, link)]
extern crate log;
extern crate collections;
extern crate toml;
extern crate getopts;

use std::old_io::{Command, File, process, pipe, fs, FilePermission};
use std::old_io::fs::PathExtensions;
use getopts::{optopt,optflag,getopts,OptGroup};
use std::os;
use std::str::FromStr;
use std::thread::Thread;
use core::fmt::Debug;

use toml::*;

//use std::ptr::RawPtr;
//use xlib::*;

use colormap::{ColorMap, Color};
use statusbar::StatusBar;

use batterybar::BatteryBar;
use brightnessbar::BrightnessBar;
use clockbar::ClockBar;
use cpubar::CpuBar;
use cputempbar::CpuTempBar;
use memorybar::MemoryBar;
use stdinbar::StdinBar;
use testbar::TestBar;
use volumebar::VolumeBar;

mod colormap;
mod statusbar;

mod batterybar;
mod brightnessbar;
mod clockbar;
mod cpubar;
mod cputempbar;
mod memorybar;
mod stdinbar;
mod testbar;
mod volumebar;


fn main() {
    // -- option parsing ---------------------------------------------
    let args: Vec<String> = os::args();
    let program = args[0].clone();
    let opts = [
        optopt("d", "dir",
               "Config directory. Default is $HOME/.config/rustybar/",
               "NAME"),
        optflag("h", "help", "Print this help menu"),
        optflag("v", "version", "Print version info"),
        ];
    let matches = match getopts(args.tail(), &opts) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(program.as_slice(), &opts);
        return;
    }
    if matches.opt_present("v") {
        print!("{}", version_info());
        return;
    }
    let dir = match matches.opt_str("d") {
        Some(dir) => {
            let path = Path::new(dir);
            if path.is_dir() { path }
            else { panic!("Please give an existing directory. You supplied: {}", path.display()) }
        },
        None => {
            let home = match os::homedir() {
                Some(p) => p,
                None => panic!("Can't find your home directory. You must specify config location."),
            };
            let bardir = home.join(".config/rustybar/");
            if bardir.exists() == false {
                match fs::mkdir_recursive(&bardir, FilePermission::from_bits(448).unwrap()) {
                    Ok(_) => (),
                    Err(msg) => panic!(msg),
                };
            }
            bardir
        },
    };
    let config_path = dir.join("config.toml");
    let log_path = dir.join("log");
    if config_path.is_file() == false {
        println!("You do not have an existing config file. Creating and populating:\n{}", config_path.display());
        let mut file = File::create(&config_path);
        match file.write(default_config()) {
            Ok(_) => (),
            Err(msg) => panic!("Couldn't write to config file: {}", msg),
        };
    }
    // -- load config file -------------------------------------------
    assert!(config_path.is_file(), "config path bad!!");
    let toml_string = File::open(&config_path).read_to_string().unwrap();
    let toml_str = toml_string.as_slice();
    let mut parser = toml::Parser::new(toml_str);
    let toml = match parser.parse() {
        Some(toml) => toml,
        // fixme: error handle better
        None => panic!("Errors in config file:\n{:?}", parser.errors),
    };

    // -- get resolution and dpi (fixme) ------------
    // let dpi = get_dpi(96);
    let res = get_resolution();

    // ---------- settings from config file -----------------------------------------
    // ----- global parameters -----------------------------------------
    let font: String = match toml.get(&"font".to_string()) {
        Some(name) => match name.as_str() {
            Some(string) => string.to_string(),
            None => panic!("Invalid key for font: {}", name),
        },
        None => "monospace-9".to_string(),
    };
    let height: uint = match toml.get(&"height".to_string()) {
        Some(val) => match val.as_integer() {
            Some(integer) => integer as uint,
            None => panic!("Invalid key for height: {}", val),
        },
        None => 18u,
    };
    let lgap: uint = match toml.get(&"left_gap".to_string()) {
        Some(val) => match val.as_integer() {
            Some(integer) => integer as uint,
            None => panic!("Invalid key for left_gap: {}", val),
        },
        None => 18u,
    };
    let rgap: uint = match toml.get(&"right_gap".to_string()) {
        Some(val) => match val.as_integer() {
            Some(integer) => integer as uint,
            None => panic!("Invalid key for right_gap: {}", val),
        },
        None => 18u,
    };
    let bg: &str = match toml.get(&"background".to_string()) {
        Some(name) => match name.as_str() {
            Some(string) => string,
            None => panic!("Invalid key for font: {}", name),
        },
        None => "#000000",
    };
    // fixme: this is broken
    let char_width = textw(font.as_slice());

    // ----- figure out bars --------------------------------
    let empty: &[toml::Value] = &[];
    let center_bars: &[toml::Value] = match toml.get(&"center".to_string()) {
        Some(val) => match val.as_slice() {
            Some(list) => list,
            None => panic!("The config entry [[center]] needs to be a list."),
        },
        None => empty,
    };
    let left_bars: &[toml::Value] = match toml.get(&"left".to_string()) {
        Some(val) => match val.as_slice() {
            Some(list) => list,
            None => panic!("The config entry [[left]] needs to be a list."),
        },
        None => empty,
    };
    let right_bars: &[toml::Value] = match toml.get(&"right".to_string()) {
        Some(val) => match val.as_slice() {
            Some(list) => list,
            None => panic!("The config entry [[right]] needs to be a list."),
        },
        None => empty,
    };
    let mut bars: Vec<Box<StatusBar + Send>> = Vec::new();
    let mut lefts: Vec<uint> = Vec::new();
    let mut lengths: Vec<uint> = Vec::new();

    // map of (index, filler_coefficient)
    let mut center_filler_space: Vec<(uint, uint)> = Vec::new();
    // center first as it will tell us how much space we have for left and right
    // center bars -------------------------------------------------------------
    extract_bars_from_toml(&mut bars, &mut lefts, &mut lengths, &mut center_filler_space, center_bars, char_width);
    if center_filler_space.len() > 0 {
        panic!("Can't have filler space (designated by negative integers) in the center region.");
    }
    let mut center_size = 0u;
    for len in lengths.iter() {
        center_size += *len;
    }
    let center_bars_left = (res - center_size)/2;
    let center_bars_right = center_bars_left + center_size;
    for i in range(0u, lefts.len()) {
        lefts[i] += center_bars_left;
    }

    // map of (index, filler_coefficient)
    let mut left_filler_space: Vec<(uint, uint)> = Vec::new();
    // left bars ---------------------------------------------------------------
    let first_left = bars.len();
    extract_bars_from_toml(&mut bars, &mut lefts, &mut lengths, &mut left_filler_space, left_bars, char_width);
    for i in range(first_left, bars.len()) {
        lefts[i] += lgap;
    }

    let mut lused = 0u;
    for len in lengths.iter().skip(first_left) {
        lused += *len;
    }
    let lfree: int = center_bars_left as int - lused as int - lgap as int;
    if lfree < 0 {
        panic!("You have not left enough for the left bars. You need at least {} more pixels.", -lfree);
    }
    let mut denom = 0u;
    for &(_, num) in left_filler_space.iter() {
        denom += num;
    }
    for &(ind, num) in left_filler_space.iter() {
        let space: uint = (lfree as uint)*num/denom;
        if ind < first_left {
            let new_lspace = bars[ind+1].get_lspace() + space;
            bars[ind + 1].set_lspace(new_lspace);
            lengths[ind + 1] += space;
            for i in range(ind + 2, lefts.len()) {
                lefts[i] += space;
            }
        }
        else {
            lengths[ind] += space;
            for i in range(ind + 1, lefts.len()) {
                lefts[i] += space;
            }
        }
    }
    // Now we'll re-find free space in case round-off errors or no filler space was selected
    // and add it all to the end
    lused = 0u;
    for len in lengths.iter().skip(first_left) {
        lused += *len;
    }
    let lfree = center_bars_left - lused - lgap;
    // if first_left > bars.len() {
    //     let i = lengths.len() - 1;
    //     lengths[i] += lfree;
    // }
    lengths[first_left] += lfree;

    // map of (index, filler_coefficient)
    let mut right_filler_space: Vec<(uint, uint)> = Vec::new();
    // right bars ---------------------------------------------------------------
    let first_right = bars.len();
    extract_bars_from_toml(&mut bars, &mut lefts, &mut lengths, &mut right_filler_space, right_bars, char_width);
    for i in range(first_right, bars.len()) {
        lefts[i] += center_bars_right;
    }

    let mut rused = 0u;
    for len in lengths.iter().skip(first_right) {
        rused += *len;
    }
    let rfree: int = res as int - center_bars_right as int - rused as int - rgap as int;
    if rfree < 0 {
        panic!("You have not left enough for the right bars. You need at least {} more pixels.", -rfree);
    }
    let mut denom = 0u;
    for &(_, num) in right_filler_space.iter() {
        denom += num;
    }
    for &(ind, num) in right_filler_space.iter() {
        let space: uint = (rfree as uint)*num/denom;
        if ind < first_right {
            let new_lspace = bars[ind + 1].get_lspace() + space;
            bars[ind + 1].set_lspace(new_lspace);
            lengths[ind + 1] += space;
            for i in range(ind + 2, lefts.len()) {
                lefts[i] += space;
            }
        }
        else {
            lengths[ind] += space;
            for i in range(ind + 1, lefts.len()) {
                lefts[i] += space;
            }
        }
    }
    // Now we'll re-find free space in case round-off errors or no filler space was selected
    // and add it all to the beginning
    rused = 0u;
    for len in lengths.iter().skip(first_right) {
        rused += *len;
    }
    let rfree = res - center_bars_right - rused - rgap;
    let cur_lspace = bars[first_right].get_lspace();
    bars[first_right].set_lspace(rfree + cur_lspace);
    lengths[first_right] += rfree;
    for i in range(first_right + 1, bars.len()) {
        lefts[i] += rfree;
    }

    //println!("fnt: {}", font);

    // start all the bars ----------------------------------------------------------------
    let mut bar_processes: Vec<process::Process> = Vec::new();
    let mut streams: Vec<Box<pipe::PipeStream>> = Vec::new();
    for i in range(0u, bars.len()) {
        let mut process = match Command::new("dzen2").args(&[
            "-fn",
            font.as_slice(),
            "-x",
            lefts[i].to_string().as_slice(),
            "-w",
            lengths[i].to_string().as_slice(),
            "-h",
            height.to_string().as_slice(),
            "-bg",
            bg,
            "-ta",
            "l",
            "-e",
            // "''",
           "onstart=lower",
                ]).spawn() {
            Err(msg) => panic!("Couldn't make dzen bar: {}", msg.desc),
            Ok(process) => process,
        };
        let stdin = Box::new(process.stdin.take().unwrap());
        streams.push(stdin);
        bar_processes.push(process);
    }

    for (bar, stream) in bars.into_iter().zip(streams.into_iter()) {
        Thread::spawn( move || {bar.run(stream);} );
    }



    // // Testing xlib
    // //let mut disp: i8 = 0;
    // //let display: *mut i8 = &mut 0i8;
    // let dpy: *mut Display;
    // unsafe {
    //     dpy = XOpenDisplay(&mut 0i8);
    // }
    // assert!(dpy.is_not_null(), "Failed to open display");
    // let black_color: u64;
    // let white_color: u64;
    // unsafe {
    //     black_color = XBlackPixel(dpy, XDefaultScreen(dpy));
    //     white_color = XWhitePixel(dpy, XDefaultScreen(dpy));
    // }
    // let mut w: Window;
    // unsafe {
    //     // Probably don't want a simple window eventually
    //     w = XCreateSimpleWindow(dpy, XDefaultRootWindow(dpy), 0, 0, res, 18, 0, black_color, black_color);
    //     // fixme: StructureNotifyMask? Not sure if I even need this though
    //     // XSelectInput(dpy, w, StructureNotifyMask);
    //     XMapWindow(dpy, w);
    //     let gc = XCreateGC(dpy, w, 0, RawPtr::null());
    // }
    // println!("{} {}", white_color, black_color);
}

fn print_usage(program: &str, _opts: &[OptGroup]) {
    println!("Usage: {} [OPTIONS]", program);
    let width = 10u;
    for opt in _opts.iter() {
        let len = opt.long_name.len() + opt.hint.len();
        let num = if len > width { 0 }
        else { width - len };
        // fixme!
        // println!("  -{}, --{}={}  {}{}",
        //          opt.short_name, opt.long_name, opt.hint, String::from_char(num, ' '), opt.desc);
    }
}


fn get_resolution() -> uint {
    let mut process = match Command::new("xrandr").spawn() {
        Ok(out) => out,
        Err(e) => panic!("Failed to run xrandr with error: {}", e),
    };
    // fixme: replace unwrap()s with error checking
    let out = process.stdout.as_mut().unwrap().read_to_string().unwrap();
    let re = regex!(r"current\s(\d+)\sx\s\d+");
    let cap = re.captures_iter(out.as_slice()).nth(0).unwrap();
    let res: uint = FromStr::from_str(cap.at(1).unwrap()).unwrap();
    res
}

// fn setfont(fnt: &str) {
//     unsafe {
// 	      char *def, **missing;
// 	      int i, n;

// 	      missing = NULL;
// 	      if(font.set)
// 		        XFreeFontSet(dpy, font.set);
// 	      font.set = XCreateFontSet(dpy, fontstr, &missing, &n, &def);
// 	      if(missing)
// 		        XFreeStringList(missing);
// 	      if(font.set) {
// 		        XFontSetExtents *font_extents;
// 		        XFontStruct **xfonts;
// 		        char **font_names;
// 		        font.ascent = font.descent = 0;
// 		        font_extents = XExtentsOfFontSet(font.set);
// 		        n = XFontsOfFontSet(font.set, &xfonts, &font_names);
// 		        for(i = 0, font.ascent = 0, font.descent = 0; i < n; i++) {
// 			          if(font.ascent < (*xfonts)->ascent)
// 				            font.ascent = (*xfonts)->ascent;
// 			          if(font.descent < (*xfonts)->descent)
// 				            font.descent = (*xfonts)->descent;
// 			          xfonts++;
// 		        }
// 	      }
// 	      else {
// 		        if(font.xfont)
// 			          XFreeFont(dpy, font.xfont);
// 		        font.xfont = NULL;
// 		        if(!(font.xfont = XLoadQueryFont(dpy, fontstr))) {
// 			          fprintf(stderr, "error, cannot load font: '%s'\n", fontstr);
// 			          exit(EXIT_FAILURE);
// 		        }
// 		        font.ascent = font.xfont->ascent;
// 		        font.descent = font.xfont->descent;
// 	      }
// 	      font.height = font.ascent + font.descent;
//     }
// }

fn textw(font: &str) -> uint {
    // let cmd1 = Command::new("xterm").args([
    //     "-fa", font,
    //     "-geometry", "40x10",
    //     "-e", "'xwininfo -id $WINDOWID'"]).output();
    // let cmd2 = Command::new("xterm").args([
    //     "-fa", font,
    //     "-geometry", "80x20",
    //     "-e", "'xwininfo -id $WINDOWID"]).output();
    // println!("{}", cmd1.unwrap().output);
    // println!("{}", cmd1.unwrap().status);

    // unsafe {
    //     let dpy = XOpenDisplay (RawPtr::null());
    //     let test = "-*-*-*-R-Normal--*-180-100-100-*-*".to_c_str().as_mut_ptr();
    //     let mut i = 0i8;
    //     let missing: *mut *mut *mut i8 = &mut (&mut (&mut 0 as *mut i8) as *mut *mut i8) as *mut *mut *mut i8;
    //     let n: *mut i32 = &mut 0 as *mut i32;
    //     let def: *mut *mut i8 = &mut (&mut 0 as *mut i8) as *mut *mut i8;
    //     let x = XCreateFontSet(dpy, test, missing, n, def);
    //     println!("{}, {}, {}, {}, {}, {}", dpy, *test, ***missing, *n, **def, x);
    // }
    7
}

fn extract_bars_from_toml(bars: &mut Vec<Box<StatusBar+Send>>, lefts: &mut Vec<uint>,
                          lengths: &mut Vec<uint>, filler_space: &mut Vec<(uint, uint)>,
                          toml: &[toml::Value], char_width: uint) {
    let mut coord = 0u;
    let mut lspace = 0u;
    let mut space: uint;
    let mut first_bar = true;
    for item in toml.iter() {
        let table = match item.as_table() {
            Some(tbl) => tbl,
            None => panic!("Invalid entry in center bar: {}", item),
        };
        let something = table.get(&"bar".to_string());
        if something == Option::None {
            let sp: int = match table.get(&"space".to_string()) {
                Some(val) => match val.as_integer() {
                    Some(integer) => integer as int,
                    None => panic!("Invalid value for space: {}", val),
                },
                None => panic!("The entries must either contain a bar name or be a space. This one is bad: {:?}",
                               table),
            };
            if sp < 0 {
                // tracking filler space
                let i = lengths.len() - 1;
                filler_space.push((i, (-sp) as uint));
            }
            else {
                match first_bar {
                    true => lspace += sp as uint,
                    false => {
                        space = sp as uint;
                        coord += space;
                        let i = lengths.len() - 1;
                        lengths[i] += space;
                    },
                };
            }
        }
        else {
            add_bar_from_toml(bars, table);
            let i = bars.len() - 1;
            if first_bar {
                bars[i].set_lspace(lspace);
                lspace = 0u;
                first_bar = false;
            }
            bars[i].initialize(char_width);
            lefts.push(coord);
            coord += bars[i].len();
            lengths.push(bars[i].len());
        }
    }
}

fn add_bar_from_toml(bars: &mut Vec<Box<StatusBar+Send>>, toml: &Table) {
    let bar_name = match toml.get(&"bar".to_string()) {
        Some(name) => match name.as_str() {
            Some(string) => string,
            None => panic!("Invalid bar: {}", name),
        },
        None => panic!("Invalid config entry. Must either be space or include a bar name: {:?}", toml),
    };
    match bar_name {
        "battery" => {
            let mut bar = Box::new(BatteryBar::new());
            match toml.get(&"space".to_string()) {
                Some(val) => match val.as_integer() {
                    Some(integer) => bar.space = integer as uint,
                    None => panic!("Invalid value for battery space: {}", val),
                },
                None => (),
            };
            match toml.get(&"battery_number".to_string()) {
                Some(val) => match val.as_integer() {
                    Some(integer) => bar.set_bat_num(integer as uint),
                    None => panic!("Invalid value for battery number: {}", val),
                },
                None => (),
            };
            bars.push(bar);
        },
        "brightness" => {
            bars.push(Box::new(BrightnessBar::new()));
        },
        "clock" => {
            let mut bar = Box::new(ClockBar::new());
            match toml.get(&"format".to_string()) {
                Some(val) => match val.as_str() {
                    Some(string) => bar.format = string.to_string(),
                    None => panic!("Invalid value for clock format: {}", val),
                },
                None => (),
            };
            match toml.get(&"color".to_string()) {
                Some(val) => match val.as_str() {
                    Some(string) => bar.color = string.to_string(),
                    None => panic!("Invalid value for clock color: {}. It should be a string of format #ffffff.", val),
                },
                None => (),
            };
            bars.push(bar);
        },
        "cpu" => {
            let mut bar = Box::new(CpuBar::new());
            match toml.get(&"space".to_string()) {
                Some(val) => match val.as_integer() {
                    Some(integer) => bar.space = integer as uint,
                    None => panic!("Invalid value for cpu space: {}", val),
                },
                None => (),
            };
            bars.push(bar);
        },
        "cpu_temp" => {
            let mut bar = Box::new(CpuTempBar::new());
            match toml.get(&"min".to_string()) {
                Some(val) => match val.as_float() {
                    Some(num) => bar.min = num as f32,
                    None => panic!("Invalid value for cpu_temp min: {}", val),
                },
                None => (),
            };
            match toml.get(&"max".to_string()) {
                Some(val) => match val.as_float() {
                    Some(num) => bar.max = num as f32,
                    None => panic!("Invalid value for cpu_temp max: {}", val),
                },
                None => (),
            };
            bars.push(bar);
        },
        "memory" => {
            bars.push(Box::new(MemoryBar::new()));
        },
        "test" => {
            bars.push(Box::new(TestBar::new()));
        },
        "stdin" => {
            let mut bar = Box::new(StdinBar::new());
            match toml.get(&"length".to_string()) {
                Some(val) => match val.as_integer() {
                    Some(integer) => bar.length = integer as uint,
                    None => panic!("Invalid value for stdin length: {}", val),
                },
                None => (),
            };
            bars.push(bar);
        },
        "volume" => {
            let mut bar = Box::new(VolumeBar::new());
            match toml.get(&"card".to_string()) {
                Some(val) => match val.as_integer() {
                    Some(integer) => bar.card = integer as uint,
                    None => panic!("Invalid value for sound card for volume bar: {}", val),
                },
                None => (),
            };
            match toml.get(&"channel".to_string()) {
                Some(val) => match val.as_str() {
                    Some(string) => bar.channel = string.to_string(),
                    None => panic!("Invalid value for channel for volume bar: {}", val),
                },
                None => (),
            };
            match toml.get(&"mute_color".to_string()) {
                Some(val) => match val.as_str() {
                    Some(string) => bar.mute_color = Color::from_str(string),
                    None => panic!("Invalid value for mute color for volume bar: {}", val),
                },
                None => (),
            };
            bars.push(bar);
        },
        _ => panic!("Bad bar name: {}", bar_name),
    };
    let i = bars.len() - 1;
    // settings universal to all bar are just width, height, and colormap
    match toml.get(&"width".to_string()) {
        Some(val) => match val.as_integer() {
            Some(integer) => bars[i].set_width(integer as uint),
            None => panic!("Invalid value for width: {}", val),
        },
        None => {},
    };
    match toml.get(&"height".to_string()) {
        Some(val) => match val.as_integer() {
            Some(integer) => bars[i].set_height(integer as uint),
            None => panic!("Invalid value for height: {}", val),
        },
        None => {},
    };
    match toml.get(&"colormap".to_string()) {
        Some(map) => {
            let colormap = match map.as_slice() {
                Some(slice) => slice,
                None => panic!("Colormap must be array. Found {:?}.", toml),
            };
            let mut cmap = ColorMap::new();
            for pair in colormap.iter() {
                let array = match pair.as_slice() {
                    Some(slice) if slice.len() == 4 => slice,
                    _ => panic!("Each colormap entry must be in the form [key, red, green, blue]."),
                };
                let mut map_array = [0u8, 0, 0, 0];
                for i in range(0u, map_array.len()) {
                    map_array[i] = match array[i].as_integer() {
                        Some(num) if num >= 0 && num <= 255 => num as u8,
                        _ => panic!("The entries in a colormap must be numbers in the range [0,255]."),
                    };
                }
                cmap.add_pair(map_array[0], Color::new(map_array[1], map_array[2], map_array[3]));
            }
            bars[i].set_colormap(Box::new(cmap));
        },
        None => (),
    };
}

fn version_info<'a>() -> &'a str {
    r###"
Rustybar version 0.1.1, Copyright (C) 2014 Paho Lurie-Gregg
Rustybar comes with ABSOLUTELY NO WARRANTY.
This is free software, and you are welcome to redistribute it.

Written by Paho Lurie-Gregg.
"###
}

fn default_config<'a>() -> &'a [u8] {
    br###"
# --- global parameters ------
# Note: I don't currently obtain font size correctly for determining the pixel width of
# characters, so bars that include text will not be sized correctly.
font = "Monospace-9"
left_gap = 20
right_gap = 108
height = 18
background = "#000000"

# to add:
# wifi

# --- left section ----------------
[[left]]
  bar = "stdin"
  # stdin is the only bar with indeterminate length, so it must be set.
  # However, it will include all space to its right.
  # In this case, everything in the left section
  # will be writable by the stdin bar
  length = 10

# --- center section --------------
[[center]]
  space = 20
[[center]]
  # the temperature of your cpu, as reported by "acpi -t"
  bar = "cpu_temp"
  width = 35
  height = 12
  # min and max designate the respective temperatures to use at the ends of the bar
  # these are accepted as floating point numbers
  min = 0.0
  max = 100.0
  # the colormap for a bar dictates what color it will be at each key point
  # anywhere in between two set points will be interpolated to give a gradual change
  # each element in the colormap is a list of the form [key, red, green, blue]
  # where keys go from 0 (for empty) to 100 (for full)
  colormap = [[  0,   0, 255, 255],
              [ 40,   0, 255, 255],
              [ 80, 255, 255,   0],
              [100, 255,   0,   0]]
[[center]]
  space = 10
[[center]]
  # the usage of your processors, separated by physical core.
  bar = "cpu"
  # for something with multiple bars, like this, width designates the width of each
  # individual bar
  width = 20
  # height designates the height of the bars
  height = 10
  # space is the space between the individual bars. It is only for things that have multiple bars.
  space = 8
  colormap = [[  0,  20,  20,  20],
              [ 15,  50,  50,  50],
              [ 30, 180, 200,  60],
              [ 60, 200, 150,   0],
              [100, 255,   0,   0]]
[[center]]
  space = 10
[[center]]
  # used memory
  bar = "memory"
  width = 35
  height = 12
  colormap = [[  0,   0, 255, 255],
              [ 40,   0, 255, 255],
              [ 80, 255, 255,   0],
              [100, 255,   0,   0]]
[[center]]
  space = 20

# --- right section -------------
# positive values for space give you a space of that many pixels negative values for
# space spread any leftover space amongst them.  For example, if you have space = -1 and
# space = -2 at two places, then the first will get 1/3 of your leftover space and the
# second will get 2/3 of it
[[right]]
  space = -3
[[right]]
  # the volume bar uses amixer to get information, so you must have alsa installed to
  # use it. It is not ideal, as the volume is just polled every second, and when a
  # library exists for Rust, a better interface for volume will be implemented
  bar = "volume"
  width = 30
  height = 10
  colormap = [[  0, 150, 100, 255],
              [100,   0, 255, 255]]
  # the volume bar will change to this color when muted
  mute_color = "#b000b0"
  # the number of the sound card to use
  card = 0
  channel = "Master"
[[right]]
  space = 20
[[right]]
  # screen brightness. Probably only for laptops
  bar = "brightness"
  width = 30
  height = 10
  colormap = [[  0, 255, 255, 255],
              [100, 128, 128, 128]]
[[right]]
  space = 20
[[right]]
  # battery bar, only for laptops
  bar = "battery"
  width = 30
  height = 10
  space = 8
  # some laptops have multiple batteries, so you could include more
  battery_number = 0
  colormap = [[  0, 255,   0,   0],
              [ 35, 255, 255,   0],
              [100,   0, 255,   0]]
[[right]]
  space = -3

# "test" is useful for viewing colormaps. This one will give you a rainbow.
[[right]]
  bar = "test"
  width = 100
  colormap = [[  0, 255,   0,   0],
              [ 20, 255, 255,   0],
              [ 40,   0, 255,   0],
              [ 60,   0, 255, 255],
              [ 80,   0,   0, 255],
              [100, 255,   0, 255]]
[[right]]
  space = -2
[[right]]
  # I like my date and clock to be different colors, so I have two clock bars.
  bar = "clock"
  # This format string gives the date. See "man date" for more options.
  format = "%a %Y-%m-%d"
  color = "#3cb371"
[[right]]
  space = 20
[[right]]
  bar = "clock"
  format = "%H:%M:%S"
  color = "#50e0ff"

"###
}
