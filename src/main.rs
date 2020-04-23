use rustybar::*;
use std::time::Duration;
use structopt::StructOpt;
use tokio::{self, io::AsyncWriteExt};

/// A convenience macro for creating a static Regex, for repeated use at only one call-site.  Copied
/// from https://github.com/Canop/lazy-regex
#[macro_export]
macro_rules! regex {
    ($s: literal) => {{
        lazy_static! {
            static ref RE: regex::Regex = regex::Regex::new($s).unwrap();
        }
        &*RE
    }};
}

struct Font<'a> {
    pub name: &'a str,
    pub width: u32,
}

impl<'a> Font<'a> {
    fn new(name: &str, width: u32) -> Font {
        Font { name, width }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "rustybar", about = "A simple statusbar program.")]
struct Opt {}

#[tokio::main]
async fn main() {
    tokio::spawn(state::tick());
    let font = Font::new("Monospace-9", 10);

    // ----------------------------------------------------------------------------------------------

    let bar = rustybar::bar::Clock::new("#aaaaaa", "%H:%M:%S", 10, 8);
    let rb = rustybar::bar::RustyBar::new(0, vec![Box::new(bar)], vec![], vec![]);
    rb.run().await;

    tokio::time::delay_for(Duration::from_secs(100000)).await;

    // ----------------------------------------------------------------------------------------------

    // let mut process = tokio::process::Command::new("dzen2")
    //     .args(&[
    //         "-dock",
    //         "-fn",
    //         font.name,
    //         "-x",
    //         "1200",
    //         "-w",
    //         "200",
    //         "-h",
    //         "18",
    //         "-bg",
    //         "#222222",
    //         "-ta",
    //         "l",
    //         "-e",
    //         "onstart=raise",
    //         "-xs",
    //         "0",
    //     ])
    //     .kill_on_drop(true)
    //     .stdin(std::process::Stdio::piped())
    //     .spawn()
    //     .unwrap();
    // // let mut child_stdin = process.stdin.unwrap();

    // let clock = rustybar::bar::Clock::new("#223344", "%H:%M:%S", 10, 80);

    // let cmap: colormap::ColorMap = [[0, 0, 255, 0], [50, 255, 255, 0], [100, 255, 0, 0]]
    //     .iter()
    //     .collect();
    // loop {
    //     let state = state::read().await;

    //     let map_val = (state.bytes_recieved() as f32 / 10e6 * 100.0) as u8;
    //     //dbg!(state.bytes_recieved() as f32, 100e9);
    //     let mut mem = format!(
    //         "^fg({}){}",
    //         cmap.map(map_val),
    //         bytes::format_bytes(state.bytes_recieved())
    //     );

    //     mem.push_str("\n");
    //     process
    //         .stdin
    //         .as_mut()
    //         .unwrap()
    //         .write_all(mem.as_bytes())
    //         .await
    //         .unwrap();

    //     print!("| ");
    //     state.cpus().for_each(|cpu| print!("{:.2} ", cpu / 100.0));

    //     let temp = state.temperature();
    //     print!("| {}° ", temp);

    //     print!("| {} ", bytes::format_bytes(state.free_memory()));
    //     print!("| {} ", bytes::format_bytes(state.bytes_recieved()));

    //     println!("|");

    //     std::mem::drop(state);
    //     tokio::time::delay_for(Duration::from_secs(1)).await;
    // }
}

// fn run(config_path: &std::path::Path) -> Result<(), failure::Error> {
//     // -- Populate default config ----------------------------------------------

//     if !config_path.is_file() {
//         info!(
//             "You do not have an existing config file. Creating and populating '{}'",
//             config_path.display()
//         );
//         let mut file = fs::File::create(&config_path)?;
//         let example_config = include_bytes!("../example_config.toml");
//         file.write_all(example_config)?;
//     }

//     // -- load config file -----------------------------------------------------
//     ensure!(config_path.is_file(), "config path bad!!");
//     let toml_string = {
//         let mut toml_string = String::new();
//         fs::File::open(&config_path)?.read_to_string(&mut toml_string)?;
//         toml_string
//     };

//     let config: config::Config = toml::from_str(&toml_string)?;
//     // debug!("Config: {:#?}", config);

//     let font = &config.font;
//     let height = config.height;
//     let bg = &config.background;

//     let mut screens = Vec::new();
//     let mut threads: Vec<thread::JoinHandle<_>> = Vec::new();
//     let reset = Arc::new(atomic::AtomicBool::new(true));

//     loop {
//         let new_screens = get_screens()?;
//         if new_screens == screens {
//             thread::sleep(Duration::from_secs(1));
//             continue;
//         }
//         std::mem::replace(&mut screens, new_screens);

//         reset.store(true, atomic::Ordering::Relaxed);

//         for thread in threads {
//             thread.join().unwrap()?;
//         }
//         threads = Vec::new();

//         reset.store(false, atomic::Ordering::Relaxed);

//         let bars = config::generate_bars(&config, screens[0].width)?;

//         let mut left = config.left_gap;
//         for mut bar in bars {
//             let len = bar.len();
//             let reset = reset.clone();
//             {
//                 let font = font.clone();
//                 let left = left;
//                 let bg = bg.clone();
//                 let handler = thread::spawn(move || -> Result<(), failure::Error> {
//                     let process = process::Command::new("dzen2")
//                         .args(&[
//                             "-dock",
//                             "-fn",
//                             &font,
//                             "-x",
//                             &left.to_string(),
//                             "-w",
//                             &(bar.len()).to_string(),
//                             "-h",
//                             &height.to_string(),
//                             "-bg",
//                             &bg,
//                             "-ta",
//                             "l",
//                             "-e",
//                             "onstart=lower",
//                             "-xs",
//                             "0",
//                         ])
//                         .stdin(process::Stdio::piped())
//                         .spawn()
//                         .unwrap();
//                     let mut child_stdin = &mut process.stdin.unwrap();
//                     bar.initialize()?;
//                     loop {
//                         if let Err(e) = bar.write(&mut child_stdin).and_then(|_| bar.block()) {
//                             error!("{}", e);
//                         }
//                         if reset.load(atomic::Ordering::Relaxed) {
//                             break;
//                         }
//                     }
//                     Ok(())
//                 });

//                 threads.push(handler);
//             }
//             left += len;
//         }
//     }
// }

// #[derive(Debug, Eq, PartialEq)]
// struct Screen {
//     pub x: u32,
//     pub y: u32,
//     pub width: u32,
// }

// fn get_screens() -> Result<Vec<Screen>, failure::Error> {
//     let output = std::process::Command::new("xrandr")
//         .arg("--listactivemonitors")
//         .output()?
//         .stdout;
//     let out = String::from_utf8(output)?;

//     regex!(r"\d:.* (\d+).*+(\d+)+(\d+)")
//         .captures_iter(&out)
//         .map(|cap| {
//             Ok(Screen {
//                 width: cap[1].parse::<u32>()?,
//                 x: cap[2].parse()?,
//                 y: cap[3].parse()?,
//             })
//         })
//         .collect()
// }
// ;
