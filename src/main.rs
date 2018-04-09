extern crate chrono;
extern crate directories;
#[macro_use]
extern crate failure;
extern crate inotify;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate structopt;
extern crate toml;

use std::{fs, process, thread, io::{Read, Write}};
use std::str::FromStr;

// use structopt::StructOpt;

mod bar;
mod colormap;
mod config;

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

    // -- get resolution and dpi (fixme) ------------
    // let dpi = get_dpi(96);
    let res = get_resolution()?;

    let font = config.font;
    let height = config.height;
    let lgap = config.left_gap;
    let rgap = config.right_gap;
    let bg = config.background;

    // TODO: THIS IS TMEPOERERXS
    let bgs = vec![
        "#ff0000", "#00ff00", "#0000ff", "#ffff00", "#ff00ff", "#00ffff", "#ff0000", "#00ff00",
        "#0000ff", "#ffff00", "#ff00ff", "#00ffff",
    ];
    let mut bg_iter = bgs.iter();

    let char_width = char_width(&font);

    let bars = {
        let center = config::entries_to_bars(&config.center, char_width, None)?;
        let center_len: u32 = center.iter().map(|b| b.len()).sum();
        let left_space = (res - center_len) / 2;
        let right_space = (res - center_len + 1) / 2;
        println!("c: {}, l: {}, r: {}", center_len, left_space, right_space);
        let mut left = config::entries_to_bars(&config.left, char_width, Some(left_space))?;
        let right = config::entries_to_bars(&config.right, char_width, Some(right_space))?;
        left.extend(center);
        left.extend(right);
        left
    };

    let mut threads = Vec::new();
    let mut left = 0;
    for bar in bars {
        println!("Left: {}, Bar len: {}", left, bar.len());
        let len = bar.len();
        {
            let font = font.clone();
            let left = left.clone();
            let bg = bg.clone();
            // let bg = bg_iter.next().unwrap().clone();
            let handler = thread::spawn(move || {
                let process = process::Command::new("dzen2")
                    .args(&[
                        "-fn",
                        &font,
                        "-x",
                        &left.to_string(),
                        "-w",
                        &bar.len().to_string(),
                        "-h",
                        &height.to_string(),
                        "-bg",
                        &bg,
                        "-ta",
                        "l",
                        "-e",
                        "onstart=lower",
                    ])
                    .stdin(process::Stdio::piped())
                    .spawn()
                    .unwrap();
                bar.run(&mut process.stdin.unwrap()).unwrap();
            });
            threads.push(handler);
        }
        left += len;
    }

    ::std::thread::sleep(std::time::Duration::from_secs(100000000000));
    // for thread in threads {
    //     thread.join()?;
    // }

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

fn get_resolution() -> Result<u32, failure::Error> {
    let output = std::process::Command::new("xrandr").output()?.stdout;
    let out = String::from_utf8(output)?;

    let re = regex::Regex::new(r"current\s(\d+)\sx\s\d+")?;
    let cap = re.captures_iter(&out).nth(0).unwrap();
    let res: u32 = FromStr::from_str(&cap[1])?;
    Ok(res)
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
