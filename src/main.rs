use rustybar::{bar, Color};
use std::time::Duration;
use tokio;

fn main() {
    let mut runtime = tokio::runtime::Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async { tokio_main().await });
}

async fn tokio_main() {
    tokio::spawn(rustybar::state::tick());
    tokio::spawn(rustybar::bar::run());

    let font = rustybar::Font::new("Monospace-12", 12);
    let height = 22;
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
        let mut config = rustybar::config::write().await;
        config.font = font;
        config.height = height;
        config.background = bg4;
    }

    let rb = rustybar::RustyBar::new(
        0,
        vec![bar::Stdin::new(100).await],
        vec![
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
            .await,
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
                16,
                ch,
                ch * 2,
            )
            .await,
            bar::Memory::new(
                [(1e9, red.into()), (3e9, blue.into()), (8e9, aqua.into())]
                    .iter()
                    .collect(),
                0,
            )
            .await,
        ],
        vec![
            bar::Clock::new(aqua, "%a %Y-%m-%d", 14, 2 * ch).await,
            bar::Clock::new(blue, "%H:%M:%S", 8, ch).await,
        ],
    );
    let mut rb2 = rb.clone();
    rb2.screen_id = 1;
    let mut rb3 = rb.clone();
    rb3.screen_id = 2;

    rustybar::start(&[rb, rb2, rb3]).await;

    tokio::time::delay_for(Duration::from_secs(100000)).await;
}
