use crate::bar::*;

use failure;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub font: String,
    pub char_width: u32,
    pub left_gap: u32,
    pub right_gap: u32,
    pub height: u32,
    pub background: String,

    #[serde(default)]
    pub left: Vec<EntryConfig>,
    #[serde(default)]
    pub center: Vec<EntryConfig>,
    #[serde(default)]
    pub right: Vec<EntryConfig>,
}

#[derive(Debug, Deserialize)]
pub struct Space {
    space: i32,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum EntryConfig {
    Bar(BarConfig),
    Space(Space),
}

enum Entry {
    Bar(Box<dyn Bar>),
    Space(i32),
}

pub fn generate_bars(config: &Config, total_width: u32) -> Result<Vec<BarWithSep>, failure::Error> {
    let center = configs_to_bars(&config.center, config.char_width, None)?;
    let center_len: u32 = center.iter().map(|b| b.len()).sum();
    let left_space = (total_width - center_len) / 2 - config.left_gap;
    let right_space = (total_width - center_len + 1) / 2 - config.right_gap;
    debug!(
        "total: {}, center: {}, left: {}, right: {}",
        total_width, center_len, left_space, right_space
    );
    let mut left = configs_to_bars(&config.left, config.char_width, Some(left_space))?;
    let right = configs_to_bars(&config.right, config.char_width, Some(right_space))?;
    left.extend(center);
    left.extend(right);
    Ok(left)
}

fn configs_to_bars(
    entry_configs: &[EntryConfig],
    char_width: u32,
    space_available: Option<u32>,
) -> Result<Vec<BarWithSep>, failure::Error> {
    let entries = configs_to_entries(entry_configs, char_width)?;
    entries_to_bars(entries, space_available)
}

fn configs_to_entries(
    entry_configs: &[EntryConfig],
    char_width: u32,
) -> Result<Vec<Entry>, failure::Error> {
    entry_configs
        .iter()
        .map(|e| match e {
            EntryConfig::Space(s) => Ok(Entry::Space(s.space)),
            EntryConfig::Bar(b) => Ok(Entry::Bar(b.to_bar(char_width)?)),
        })
        .collect()
}

// Don't use space_available for the center bar
fn entries_to_bars(
    entries: Vec<Entry>,
    space_available: Option<u32>,
) -> Result<Vec<BarWithSep>, failure::Error> {
    let space_used: u32 = entries
        .iter()
        .map(|e| match *e {
            Entry::Space(s) => {
                if s > 0 {
                    s as u32
                } else {
                    0
                }
            }
            Entry::Bar(ref b) => b.len(),
        })
        .sum();

    if let Some(space_available) = space_available {
        if space_used > space_available {
            bail!(
                "Out of space. {} used, but {} available.",
                space_used,
                space_available
            );
        }
    }

    let space_remaining = space_available.map(|a| a - space_used).unwrap_or(0);

    let dynamic_space_denom: u32 = entries
        .iter()
        .filter_map(|e| {
            if let Entry::Space(s) = *e {
                if s < 0 {
                    Some(-s as u32)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .sum();

    let dynamic_space_remainder = if dynamic_space_denom == 0 {
        0
    } else {
        space_remaining - (space_remaining / dynamic_space_denom * dynamic_space_denom)
    };
    debug!("Dynamic remainder: {}", dynamic_space_remainder);

    if space_available.is_none() && dynamic_space_denom > 0 {
        bail!("Cannot use dynamic space on the center bar.");
    }

    let mut res: Vec<BarWithSep> = Vec::new();

    let mut space = 0;
    for entry in entries.into_iter() {
        match entry {
            Entry::Space(s) => {
                let sp = if s < 0 {
                    ((-s) as u32 * space_remaining) / dynamic_space_denom
                } else {
                    s as u32
                };
                space += sp;
            }
            Entry::Bar(bar) => {
                let mut new = BarWithSep::new(bar);
                new.left = space;
                space = 0;
                res.push(new);
            }
        }
    }

    if let Some(bar) = res.last_mut() {
        bar.right = space + dynamic_space_remainder;
    }

    Ok(res)
}
