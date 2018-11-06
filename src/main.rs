#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate slog_scope;
#[macro_use(slog_o, slog_debug, slog_info, slog_warn, slog_error)]
extern crate slog;

use directories;
use regex;
use slog::Drain;
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

fn run(config_path: &std::path::Path) -> Result<(), failure::Error> {
    // -- Populate default config ----------------------------------------------

    if !config_path.is_file() {
        info!(
            "You do not have an existing config file. Creating and populating '{}'",
            config_path.display()
        );
        let mut file = fs::File::create(&config_path)?;
        let example_config = include_bytes!("../example_config.toml");
        file.write_all(example_config)?;
    }

    // -- load config file -----------------------------------------------------
    ensure!(config_path.is_file(), "config path bad!!");
    let toml_string = {
        let mut toml_string = String::new();
        fs::File::open(&config_path)?.read_to_string(&mut toml_string)?;
        toml_string
    };

    let config: config::Config = toml::from_str(&toml_string)?;
    // debug!("Config: {:#?}", config);

    let font = &config.font;
    let height = config.height;
    let bg = &config.background;

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
            thread.join().unwrap()?;
        }
        threads = Vec::new();

        reset.store(false, atomic::Ordering::Relaxed);

        let bars = config::generate_bars(&config, screens[0].width)?;

        let mut left = config.left_gap;
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
                        if let Err(e) = bar.write(&mut child_stdin).and_then(|_| bar.block()) {
                            error!("{}", e);
                        }
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
}

fn main() {
    let project_dirs = directories::ProjectDirs::from("", "", "rustybar");
    let config_dir = project_dirs.config_dir();
    let config_path = config_dir.join("config.toml");
    let log_path = config_dir.join("log");

    // -- setup logging --------------------------------------------------------

    {
        // _guard needs to be in a smaller scope so that it is dropped before `exit`
        let _guard = {
            let log_file = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(log_path)
                .unwrap();
            let drain_file = {
                let decorator = slog_term::PlainDecorator::new(log_file);
                let drain = slog_term::FullFormat::new(decorator).build().fuse();
                slog_async::Async::new(drain).build().fuse()
            };

            let drain_stdout = {
                let decorator = slog_term::TermDecorator::new().build();
                let drain = slog_term::FullFormat::new(decorator).build().fuse();
                slog_async::Async::new(drain).build().fuse()
            };

            let drain = slog::Duplicate::new(drain_file, drain_stdout).fuse();
            let log = slog::Logger::root(drain, slog_o!());

            slog_scope::set_global_logger(log)
        };

        // Run!
        if let Err(e) = run(&config_path) {
            error!("Error: {}\nBacktrace: {}", e, e.backtrace());
        }
    }
    std::process::exit(1);
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
                width: cap[1].parse::<u32>()?,
                x: cap[2].parse()?,
                y: cap[3].parse()?,
            })
        })
        .collect()
}
