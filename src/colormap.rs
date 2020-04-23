use crate::color::Color;

/// A map that stores (value, color) pairs, which can be used to interpolate between
/// colors for arbitrary values. Values are in the range [0,100].
///
/// # Example
/// fixme: add example
#[derive(Clone, Debug)]
pub struct ColorMap {
    colors: Vec<Color>,
    values: Vec<u8>,
}

impl<'a> std::iter::FromIterator<&'a [u8; 4]> for ColorMap {
    /// Convert an iterator of 4 element arrays to a ColorMap. The first element in each array is
    /// treated as the value, and the last 3 as an RGB triplet.
    fn from_iter<I: IntoIterator<Item = &'a [u8; 4]>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut values = vec![];
        let mut colors = vec![];
        for &[v, r, g, b] in iter {
            values.push(v);
            colors.push(Color::new(r, g, b));
        }
        ColorMap { values, colors }
    }
}

impl ColorMap {
    /// This does the interpolation, and gives you the color corresponding to the value
    /// called with, as dicated by the color map.
    /// Clamps val to a maximum of 100.
    pub fn map(&self, val: u8) -> Color {
        let val = std::cmp::min(val, 100);

        let mut i = 1;
        while self.values[i] < val {
            i += 1;
        }
        let lower: f32 =
            f32::from(self.values[i] - val) / f32::from(self.values[i] - self.values[i - 1]);
        let upper: f32 =
            f32::from(val - self.values[i - 1]) / f32::from(self.values[i] - self.values[i - 1]);

        let interpolate = |c1, c2| (lower * (f32::from(c1)) + upper * f32::from(c2)) as u8;
        let red: u8 = interpolate(self.colors[i - 1].r, self.colors[i].r);
        let green: u8 = interpolate(self.colors[i - 1].g, self.colors[i].g);
        let blue: u8 = interpolate(self.colors[i - 1].b, self.colors[i].b);

        Color {
            r: red,
            g: green,
            b: blue,
        }
    }
}
