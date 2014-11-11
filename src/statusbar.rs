
use colormap::{Color, ColorMap};
use std::io::pipe;

// fixme: this should be settable
static TEXTCOLOR: &'static str = "#888888";

pub trait StatusBar {
    /// Find any initial information needed for the bar, then begin a thread where it
    /// updates either based on time or some other qualification
    fn initialize(&mut self, char_width: uint);
    fn run(&self, mut stream: Box<pipe::PipeStream>);
    fn set_colormap(&mut self, cmap: Box<ColorMap>);
    /// Give the length in pixels of the output string.
    fn len(&self) -> uint;
}

pub fn write_one_bar(stream: &mut pipe::PipeStream, val: f32, color: Color, width: uint, height: uint) {
    let wfill = (val*(width as f32).round()) as uint;
    let wempty = width - wfill;
    match write!(stream, "^fg({0})^r({2}x{1})^ro({3}x{1})", color, height, wfill, wempty) {
        Err(msg) => panic!("Failed to write pipe: {}", msg.desc),
        Ok(_) => (),
    };
}

pub fn write_space(stream: &mut pipe::PipeStream, width: uint) {
    match write!(stream, "^r({}x0)", width) {
        Err(msg) => panic!("Failed to write pipe: {}", msg.desc),
        Ok(_) => (),
    };
}

pub fn write_sep(stream: &mut pipe::PipeStream, height: uint) {
    match write!(stream, "^fg({})^r(2x{})", TEXTCOLOR, height) {
        Err(msg) => panic!("Failed to write pipe: {}", msg.desc),
        Ok(_) => (),
    };
}
