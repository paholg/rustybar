use std::fmt;
use std::iter::{range_step};

/// An RGB triplet
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// creates a Color object, guaranteeing that the colors are acceptable values.
    pub fn new(red: f64, green: f64, blue: f64) -> Color {
        assert!((red >= 0.0) && (red <= 1.0), "Tried to set a red color value to {} (needs to be in [0,1])", red);
        assert!((green >= 0.0) && (green <= 1.0), "Tried to set a green color value to {} (needs to be in [0,1])", green);
        assert!((blue >= 0.0) && (blue <= 1.0), "Tried to set a blue color value to {} (needs to be in [0,1])", blue);
        Color{r: (red*255.) as u8, g: (green*255.) as u8, b: (blue*255.) as u8}
    }
    // pub fn newu(red: u8, green: u8, blue: u8) -> Color {
    //     Color{r: red, g: green, b: blue}
    // }
}

impl fmt::Show for Color {
    /// Color triplets will be printed in the form #ffffff
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b) }
}

/// A map that stores (value, color) pairs, which can be used to interpolate between
/// colors for arbitrary values. Values are in the range [0,100].
///
/// # Example
/// fixme: add example

pub struct ColorMap {
    colors: Vec<Color>,
    values: Vec<u8>,
}

impl ColorMap {
    /// Creates a new colormap. To start, it maps 0 to black and 100 to white, but these
    /// values are changeable.
    pub fn new() -> ColorMap {
        ColorMap{colors: vec![Color::new(0., 0., 0.), Color::new(1., 1., 1.)], values: vec![0, 100]}
    }

    /// Adds a (value, color) pair to the colormap. Value must be in the range
    /// [0,100]. If the value already exists, then it will override the corresponding
    /// color.
    pub fn add_pair(&mut self, val: u8, color: Color) {
        assert!(val <= 100, "Value {} outside of range. Must be in [0,100].", val);
        let mut i = 0u;
        loop {
            if self.values[i] == val {
                self.colors[i] = color;
                break;
            }
            else if self.values[i] > val {
                self.values.insert(i, val);
                self.colors.insert(i, color);
                break;
            }
            i += 1;
        }
    }

    /// This does the interpolation, and gives you the color corresponding to the value
    /// called with, as dicated by the color map. Should throw an error if index is not
    /// in the range [0, 100].
    pub fn map(&self, val: u8) -> Color {
        assert!(val <= 100, "Tried to get a color using index {} (needs to be in [0,100]).", val);
        let mut i = 1u;
        while self.values[i] < val {
            i += 1;
        }
        let lower: f32 = ((self.values[i] - val) as f32)/((self.values[i] - self.values[i-1]) as f32);
        let upper: f32 = ((val - self.values[i-1]) as f32)/((self.values[i] - self.values[i-1]) as f32);

        let red: u8 = (lower*(self.colors[i-1].r as f32) + upper*(self.colors[i].r as f32)) as u8;
        let green: u8 = (lower*(self.colors[i-1].g as f32) + upper*(self.colors[i].g as f32)) as u8;
        let blue: u8 = (lower*(self.colors[i-1].b as f32) + upper*(self.colors[i].b as f32)) as u8;
        Color{r: red, g: green, b: blue}
    }

    /// Print even numbers from 0 to 100, each with the appropriate color code.
    pub fn test(&self) {
        for i in range_step(0u8, 101, 2) {
            print!("^fg({}){} ", self.map(i), i);
        }
        println!("");
    }
    // fixme: add show?
    // fixme: we should have a way to to remove elements
}
