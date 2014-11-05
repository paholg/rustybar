use std::fmt;
use colormap::ColorMap;
use std::num;

pub trait StatusBar : fmt::Show {
    /// Find any initial information needed for the bar, then begin a thread where it
    /// updates either based on time or some other qualification
    fn initialize(&mut self, width: uint, space: uint, height: uint);
    fn update(&mut self);
    fn set_colormap(&mut self, cmap: Box<ColorMap>);
    /// Give the length in pixels of the output string.
    fn len(&self) -> uint;
}

// Every struct implementing statusbar should contain a string object, and show should
// do no more than write it. Everything else should be in update. This is because not
// all bars will update at the same frequency, a given bar may be called to show() far
// more often than to update().
impl<'a> fmt::Show for Box<StatusBar + 'a> {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result { (**self).fmt(f) }
}

pub trait FormatBar {
    /// adds a rectangle of width width and height height to self, filled in
    /// fractionally by val assumes val in [0,1]
    fn add_bar(&mut self, val: f64, width: uint, height: uint);
    /// add a space to self
    fn add_space(&mut self, width: uint);
}

impl FormatBar for String {
    // fixme: make this more efficient -- it should just add to self. Maybe I should be
    // using a Writer instead of a String?
    fn add_bar(&mut self, val: f64, width: uint, height: uint) {
        let wfill = (val*(width as f64).round()) as uint;
        let wempty = width - wfill;
        self.push_str(format!("^r({1}x{0})^ro({2}x{0})", height, wfill, wempty).as_slice());
    }
    fn add_space(&mut self, width: uint) {
        self.push_str(format!("^r({}x0)", width).as_slice());
    }
}

