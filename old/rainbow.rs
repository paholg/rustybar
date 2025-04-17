use crate::bar::{Bar, Writer};
use crate::colormap::{ColorMap, ColorMapConfig};
use std::io::Write;

#[derive(Debug, Deserialize)]
pub struct RainbowConfig {
    #[serde(default)]
    colormap: ColorMapConfig,
    width: u32,
    height: u32,
}

#[derive(Debug)]
pub struct Rainbow {
    cmap: ColorMap,
    width: u32,
    height: u32,
}

impl Rainbow {
    pub fn from_config(
        config: &RainbowConfig,
        _char_width: u32,
    ) -> Result<Rainbow, failure::Error> {
        Ok(Rainbow {
            cmap: ColorMap::from_config(&config.colormap)?,
            width: config.width,
            height: config.height,
        })
    }
}

impl Bar for Rainbow {
    fn len(&self) -> u32 {
        self.width
    }

    fn write(&mut self, w: &mut Writer) -> Result<(), failure::Error> {
        for i in 0..self.width {
            let color = self
                .cmap
                .map(((f64::from(i) / f64::from(self.width)) * 100.) as u8);
            write!(w, "^fg({})^r(1x{})", color, self.height)?;
        }
        Ok(())
    }
}
