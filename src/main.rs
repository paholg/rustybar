#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

use directories;
use regex;
use std::{
    fs,
    io::{Read, Write},
    process,
    sync::{atomic, Arc},
    thread,
};
use structopt::StructOpt;
use toml;

// use structopt::StructOpt;

mod bar;
mod color;
mod colormap;
mod config;

use crate::bar::Bar;

#[derive(Debug, StructOpt)]
#[structopt(name = "rustybar", about = "A simple statusbar program.")]
struct Opt {}

fn run() -> Result<(), failure::Error> {
    let project_dirs = directories::ProjectDirs::from("", "", "rustybar");
    let config_dir = project_dirs.config_dir();
    let config_path = config_dir.join("config.toml");

    if !config_path.is_file() {
        println!(
            "You do not have an existing config file. Creating and populating: {}",
            config_path.display()
        );
        let mut file = fs::File::create(&config_path)?;
        file.write_all(default_config())?;
    }

    // -- load config file -------------------------------------------
    assert!(config_path.is_file(), "config path bad!!");
    let toml_string = {
        let mut toml_string = String::new();
        fs::File::open(&config_path)?.read_to_string(&mut toml_string)?;
        toml_string
    };

    // let config: toml::Value = toml::from_str(&toml_string)?;
    // println!("Config: {:#?}", config);

    let config: config::Config = toml::from_str(&toml_string)?;
    println!("Config: {:#?}", config);

    // -- get dpi (fixme) ------------
    // let dpi = get_dpi(96);

    let font = &config.font;
    let height = config.height;
    let _lgap = config.left_gap;
    let _rgap = config.right_gap;
    let bg = &config.background;

    // TODO: THIS IS TMEPOERERXS
    // let bgs = vec![
    //     "#ff0000", "#00ff00", "#0000ff", "#ffff00", "#ff00ff", "#00ffff", "#ff0000", "#00ff00",
    //     "#0000ff", "#ffff00", "#ff00ff", "#00ffff",
    // ];
    // let mut bg_iter = bgs.iter();

    let char_width = char_width(font);

    let mut screens = Vec::new();
    let mut threads: Vec<thread::JoinHandle<_>> = Vec::new();
    let reset = Arc::new(atomic::AtomicBool::new(true));

    loop {
        let new_screens = get_screens()?;
        if new_screens == screens {
            thread::sleep(std::time::Duration::from_secs(1));
            continue;
        }
        std::mem::replace(&mut screens, new_screens);

        reset.store(true, atomic::Ordering::Relaxed);

        for thread in threads {
            thread.join().unwrap();
        }
        threads = Vec::new();

        reset.store(false, atomic::Ordering::Relaxed);

        let bars = config::generate_bars(&config, char_width, screens[0].width)?;

        let mut left = 0;
        for mut bar in bars {
            let len = bar.len();
            let reset = reset.clone();
            {
                let font = font.clone();
                let left = left;
                let bg = bg.clone();
                let handler = thread::spawn(move || -> Result<(), failure::Error> {
                    let process = process::Command::new("dzen2")
                        .args(&[
                            "-dock",
                            "-fn",
                            &font,
                            "-x",
                            &left.to_string(),
                            "-w",
                            &(bar.len()).to_string(),
                            "-h",
                            &height.to_string(),
                            "-bg",
                            &bg,
                            "-ta",
                            "l",
                            "-e",
                            "onstart=lower",
                            "-xs",
                            "0",
                        ])
                        .stdin(process::Stdio::piped())
                        .spawn()
                        .unwrap();
                    let mut child_stdin = &mut process.stdin.unwrap();
                    bar.initialize()?;
                    loop {
                        bar.write(&mut child_stdin)?;
                        bar.block()?;
                        if reset.load(atomic::Ordering::Relaxed) {
                            break;
                        }
                    }
                    Ok(())
                });

                threads.push(handler);
            }
            left += len;
        }
    }

    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error: {}\n{}", e, e.backtrace());
            std::process::exit(1);
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Screen {
    pub x: u32,
    pub y: u32,
    pub width: u32,
}

fn get_screens() -> Result<Vec<Screen>, failure::Error> {
    lazy_static! {
        static ref RE: regex::Regex = regex::Regex::new(r"\d:.* (\d+).*+(\d+)+(\d+)").unwrap();
    }

    let output = std::process::Command::new("xrandr")
        .arg("--listactivemonitors")
        .output()?
        .stdout;
    let out = String::from_utf8(output)?;
    RE.captures_iter(&out)
        .map(|cap| {
            Ok(Screen {
                width: cap[1].parse::<u32>()? - 1,
                x: cap[2].parse()?,
                y: cap[3].parse()?,
            })
        })
        .collect()
}

fn char_width(_font: &str) -> u32 {
    10
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
