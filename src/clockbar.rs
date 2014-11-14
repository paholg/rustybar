use statusbar::*;
use colormap::ColorMap;
use std::io::timer;
use std::time::Duration;
use std::io::pipe;
use time;

/// A statusbar for testing colormaps.
pub struct ClockBar {
    pub format: String,
    pub color: String,
    pub lspace: uint,
    char_width: uint,
    length: uint,
}

impl ClockBar {
    pub fn new() -> ClockBar {
        ClockBar {
            color: "#cccccc".to_string(),
            lspace: 0,
            format: "%I:%M:%S".to_string(),
            char_width: 0,
            length: 0,
        }
    }
}

impl StatusBar for ClockBar {
    fn initialize(&mut self, char_width: uint) {
        self.char_width = char_width;

        let now = time::now();
        let text = now.strftime(self.format.as_slice()).unwrap();
        let string = format!("{}", text);
        self.length = char_width*string.trim().len();
    }
    fn run(&self, mut stream: Box<pipe::PipeStream>) {
        loop {
            write_space(&mut *stream, self.lspace);

            let now = time::now();
            let text = now.strftime(self.format.as_slice()).unwrap();
            let full_text = format!("^fg({}){}\n", self.color, text);
            match stream.write_str(full_text.as_slice()) {
                Err(msg) => println!("Trouble writing to test bar: {}", msg),
                Ok(_) => (),
            };
            timer::sleep(Duration::seconds(1));
        }
    }
    fn set_colormap(&mut self, cmap: Box<ColorMap>) {
        cmap.map(1);
    }
    fn len(&self) -> uint {
        self.lspace + self.length
    }
    fn get_lspace(&self) -> uint { self.lspace }
    fn set_lspace(&mut self, lspace: uint) { self.lspace = lspace }
    fn set_width(&mut self, width: uint) { width +1; }
    fn set_height(&mut self, height: uint) { height+1; }
}