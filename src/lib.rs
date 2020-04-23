pub mod bar;
pub mod bytes;
pub mod color;
pub mod colormap;
pub mod state;

use bar::{Bar, DynBar};
pub use bytes::format_bytes;

#[derive(Debug, Eq, PartialEq)]
struct Screen {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}
