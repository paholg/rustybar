#![feature(globs)]
#![feature(phase)]
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;
extern crate inotify;
extern crate time;
extern crate core;
// extern crate xlib;
// #![feature(phase)]
#[phase(plugin, link)]
extern crate log;
extern crate serialize;
// extern crate glob;
extern crate collections;

use std::io::Command;
// use std::io::timer;
// use std::time::Duration;

use colormap::{ColorMap, Color};
use statusbar::StatusBar;
use cpubar::CpuBar;
use batterybar::BatteryBar;
use brightnessbar::BrightnessBar;
use std::io::{File};
use std::io::fs::PathExtensions;
// use std::sync::Future;
// use std::comm::channel;
use std::io::process;
use std::io::pipe;

use serialize::json;
use std::collections::HashMap;
use serialize::{Decodable, Decoder};

// use std::ptr::RawPtr;
// use xlib::*;

mod colormap;
mod statusbar;
mod cpubar;
mod batterybar;
mod brightnessbar;

fn get_resolution() -> uint {
    let mut process = match Command::new("xrandr").spawn() {
        Ok(out) => out,
        Err(e) => panic!("Failed to run xrandr with error: {}", e),
    };
    // fixme: replace unwrap()s with error checking
    let out = process.stdout.as_mut().unwrap().read_to_string().unwrap();
    let re = regex!(r"current\s(\d+)\sx\s\d+");
    let cap = re.captures_iter(out.as_slice()).nth(0).unwrap();
    let res: uint = from_str(cap.at(1)).unwrap();
    res
}

fn get_dpi(default: uint) -> uint {
    let path = Path::new("/var/log/Xorg.0.log");
    if !path.exists() {
        // fixme: make this an actual warning if such exists -- couldn't find it
        println!("Warning!! couldn't find Xorg log to set dpi. Using the default of {}. If this is wrong, your bars won't be sized correctly. The log file searched for was: {}", default, path.display());
        return default;
    }
    let x_log = File::open(&path).read_to_string().unwrap();
    let re = regex!(r"DPI.*?\((\d+),\s\d+\)");
    let mut dpi: uint = 0;
    for cap in re.captures_iter(x_log.as_slice()) {
        dpi = from_str(cap.at(1)).unwrap();
    }
    if dpi == 0 {
        println!("Warning!! I couldn't find your dpi. Using the default of {}. If this is wrong, your bars won't be sized correctly. The log file searched was: {}", default, path.display());
        return default;
    }
    dpi
}
#[deriving(Decodable)]
struct Global {
    font: String,
    fontsize: uint,
    left_gap: uint,
    right_gap: uint,
    height: uint,
    background: String,
}

fn main() {
    // ------------ color maps --------------------------------
    let mut cpu_cmap = box ColorMap::new();
    cpu_cmap.add_pair(0,   Color::new(0., 0., 0.));
    cpu_cmap.add_pair(5,  Color::new(0., 0., 0.));
    cpu_cmap.add_pair(15,  Color::new(0.2, 0.2, 0.2));
    cpu_cmap.add_pair(30,  Color::new(0.7, 0.8, 0.25));
    cpu_cmap.add_pair(60,  Color::new(0.8, 0.6, 0.0));
    cpu_cmap.add_pair(100, Color::new(1.0, 0.0, 0.0));
    cpu_cmap.test();

    let mut battery_cmap = box ColorMap::new();
    battery_cmap.add_pair(0,   Color::new(1., 0., 0.));
    battery_cmap.add_pair(100, Color::new(0., 1., 0.));

    let mut brightness_cmap = box ColorMap::new();
    brightness_cmap.add_pair(0,   Color::new(1., 1., 1.));
    brightness_cmap.add_pair(100, Color::new(0.5, 0.5, 0.5));
    // -------------------------------------------------------

    let dpi = get_dpi(96);
    let res = get_resolution();
    println!("{}, {}", res, dpi);


    // ---------- doing config nonsense -----------------------------------
    let config_path = Path::new("config.json");
    assert!(config_path.is_file(), "config path bad!!");
    let json_string = File::open(&config_path).read_to_string().unwrap();
    let json_str = json_string.as_slice();
    let json = match json::from_str(json_str) {
        Ok(json) => json,
        Err(err) => panic!("json::from_str(): {}", err),
    };
    // let map: HashMap<String, String> = json::decode(json_str).unwrap();
    // let mut decoder = Decoder::new(json);
    // let hm: HashMap<uint, bool> = Decodable::decode(&mut decoder);

    // println!("map: {}", map);

    // ----- global parameters ------------------------------------
    let font_size: uint = match json.find("font_size") {
        Some(json) => match json.as_u64() {
            Some(size) => size as uint,
            None => 9u,
        },
        None => 9u,
    };
    let font: String = match json.find("font") {
        Some(json) => match json.as_string() {
            Some(name) => format!("{}-{}", name, font_size),
            None => format!("Monospace-{}", font_size),
        },
        None => format!("Monospace-{}", font_size),
    };
    let height: uint = match json.find("height") {
        Some(json) => match json.as_u64() {
            Some(size) => size as uint,
            None => 18u,
        },
        None => 18u,
    };
    let lgap: uint = match json.find("left_gap") {
        Some(json) => match json.as_u64() {
            Some(size) => size as uint,
            None => 0u,
        },
        None => 0u,
    };
    let rgap: uint = match json.find("right_gap") {
        Some(json) => match json.as_u64() {
            Some(size) => size as uint,
            None => 0u,
        },
        None => 0u,
    };
    let bg: &str = match json.find("background") {
        Some(json) => match json.as_string() {
            Some(name) => name,
            None => "#000000",
        },
        None => "#000000",
    };

    // fixme: make sure this holds when it's not an integer.
    // I can also just use Xlib to find it more better
    let char_width = font_size*dpi/72;

    // ----- figure out bars --------------------------------
    // center first as that one's special
    let empty: Vec<json::Json> = Vec::new();
    let center: &Vec<json::Json> = match json.find("center") {
        Some(json) => match json.as_list() {
            Some(list) => list,
            None => panic!("The json key \"center\" needs to be a list."),
        },
        None => &empty,
    };
    let left: &Vec<json::Json> = match json.find("left") {
        Some(json) => match json.as_list() {
            Some(list) => list,
            None => panic!("The json key \"left\" needs to be a list."),
        },
        None => &empty,
    };
    let right: &Vec<json::Json> = match json.find("right") {
        Some(json) => match json.as_list() {
            Some(list) => list,
            None => panic!("The json key \"right\" needs to be a list."),
        },
        None => &empty,
    };

    fn add_from_json(bars: &mut Vec<Box<StatusBar+Send>>, json: &json::Json) {
        let bar_names = ["battery", "brightness", "cpu", "volume"];
        let mut data: &json::Json;
        let mut found = false;
        for name in bar_names.iter() {
            match json.find(*name) {
                Some(json) => found = true,
                None => (),
            }
            if found {
                data = json.find(*name).unwrap();
                match *name {
                    "battery" => bars.push(box BatteryBar::new()),
                    "brightness" => bars.push(box BrightnessBar::new()),
                    "cpu" => bars.push(box CpuBar::new()),
                    //"volume" => box VolumeBar::new(),
                    _ => panic!("This can't happen."),
                };
                return;
            }
        }
        panic!("The following json does not correspond to a valid bar: {}", json.to_pretty_str());
    }
    let mut bars: Vec<Box<StatusBar + Send>> = Vec::new();
    let mut lefts: Vec<uint> = Vec::new();
    let mut lengths: Vec<uint> = Vec::new();

    let mut coord = 0u;
    for item in center.iter() {
        println!("item: {}", item);
        // match item.find("space")
        // if item.find("space") {
        // }
        if item.is_object() {
            add_from_json(&mut bars, item);
            let i = bars.len() - 1;
            bars[i].initialize(char_width);
            println!("sadsa");

            lefts.push(coord);
            lengths.push(bars[i].len());
            coord += lengths[i];
        }
        else {
            //panic!("Invalid entry in config:\n{}", item.to_pretty_str());
        }
    }
    //return;
    // println!("cen: {}", center);
    // println!("left: {}", left);
    // println!("right: {}", right);
    // println!("null: {}", empty);



    // fixme: this stuff will be moved ------------------------

    // let mut cpu = box CpuBar::new();
    // cpu.set_colormap(cpu_cmap);
    // bars.push(cpu);

    // let mut battery = box BatteryBar::new();
    // // fixme: add this to config
    // battery.set_bat_num(0);
    // battery.set_colormap(battery_cmap);
    // bars.push(battery);

    // let mut brightness = box BrightnessBar::new();
    // brightness.set_colormap(brightness_cmap);
    // // fixme: add option to set dir in config
    // bars.push(brightness);


    // for i in range(0u, bars.len()) {
    //     bars[i].initialize(char_width);
    // }
    // --------------------------------------------------------








    // // fixme: code goes here that determines which of the bars we use and where they go,
    // // now that we have their sizes from initialize(). This will give us two parameters
    // // per bar: the left coordinate and the length.
    // for bar in bars.iter() {
    //     lefts.push(coord);
    //     lengths.push(bar.len()*2);
    //     coord += bar.len()*2 + 50;
    //     println!("len: {}", bar.len());
    // }

    let mut bar_processes: Vec<process::Process> = Vec::new();
    let mut streams: Vec<Box<pipe::PipeStream>> = Vec::new();
    for i in range(0u, bars.len()) {
        let mut process = match Command::new("dzen2").args([
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
                ]).spawn() {
            Err(msg) => panic!("Couldn't make dzen bar: {}", msg.desc),
            Ok(process) => process,
        };
        let stdin = box process.stdin.take().unwrap();
        streams.push(stdin);
        bar_processes.push(process);
    }

    for (bar, stream) in bars.into_iter().zip(streams.into_iter()) {
        spawn( proc() {bar.run(stream);} );
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
