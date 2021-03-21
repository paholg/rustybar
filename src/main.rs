use rustybar::{
    bar::{self, Bar},
    util::screen::get_screens,
    RustyBar,
};
use std::time::Duration;
use tokio;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut screens = Vec::new();
    let mut bars;

    loop {
        let new_screens = get_screens();
        if new_screens != screens {
            screens = new_screens;
            bars = Vec::new();
            for screen in &screens {
                let mut bar = init_bar().await;
                bar.start(screen).await;
                bars.push(bar);
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn init_bar() -> RustyBar {
    let font = rustybar::Font::new("Monospace-12".into(), 12);
    let height = 24;
    let ch = font.width;

    // Spacemacs dark colors
    let bg1 = "#292b2e";
    let bg2 = "#1f2022";
    let bg4 = "#0a0814";
    let aqua = "#2d9574";
    let blue = "#4f97d7";
    let magenta = "#a31db1";
    let red = "#f2241f";

    {
        // TODO: Move all this to methods
        let mut config = rustybar::config::write().await;
        config.font = font;
        config.height = height;
        config.background = bg4.into();
    }

    rustybar::RustyBar::new(
        vec![bar::Stdin::new(100).await.start()],
        vec![
            bar::Clock::new(aqua, "%H:%M:%S", 8, ch).await.start(),
            bar::Temp::new(
                [
                    (40.0, aqua.into()),
                    (60.0, blue.into()),
                    (80.0, magenta.into()),
                    (100.0, red.into()),
                ]
                .iter()
                .collect(),
                ch * 2,
            )
            .await
            .start(),
            bar::Cpu::new(
                [
                    (0.0, bg2.into()),
                    (0.2, bg1.into()),
                    (0.4, aqua.into()),
                    (0.8, magenta.into()),
                    (1.0, red.into()),
                ]
                .iter()
                .collect(),
                40,
                80,
                16,
                ch,
                ch * 2,
            )
            .await
            .start(),
            bar::Memory::new(
                [(1e9, red.into()), (3e9, blue.into()), (8e9, aqua.into())]
                    .iter()
                    .collect(),
                0,
            )
            .await
            .start(),
        ],
        vec![
            bar::Network::new(
                [
                    (0.0, bg2.into()),
                    (1e3, bg1.into()),
                    (10e3, aqua.into()),
                    (100e3, blue.into()),
                    (1e6, magenta.into()),
                    (50e6, red.into()),
                ]
                .iter()
                .collect(),
                4 * ch,
            )
            .await
            .start(),
            bar::Battery::new(
                [
                    (0.0, red.into()),
                    (0.3, magenta.into()),
                    (0.7, blue.into()),
                    (1.0, aqua.into()),
                ]
                .iter()
                .collect(),
                bar::BatteryColors {
                    charge: aqua.into(),
                    discharge: red.into(),
                    unknown: magenta.into(),
                },
                40,
                16,
                ch,
                ch,
                ch * 2,
            )
            .await
            .start(),
            bar::Clock::new(blue, "%a %Y-%m-%d", 14, 2 * ch)
                .await
                .start(),
            bar::Clock::new(aqua, "%H:%M:%S", 8, ch).await.start(),
        ],
    )
}
