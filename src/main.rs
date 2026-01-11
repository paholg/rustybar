use std::collections::{HashMap, HashSet};

use rustybar::{iced_bar, producer::niri};
use tokio::task::JoinHandle;

#[derive(Default)]
struct BarManager {
    bars: HashMap<String, JoinHandle<eyre::Result<()>>>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let mut manager = BarManager::default();

    let mut receiver = niri::listen();

    loop {
        receiver.changed().await.unwrap();
        let msg = receiver.borrow();
        let output_names = msg.outputs.keys().collect::<HashSet<_>>();

        manager.bars.retain(|name, jh| {
            if output_names.contains(name) {
                true
            } else {
                jh.abort();
                false
            }
        });

        for name in output_names {
            if !manager.bars.contains_key(name) {
                let name_clone = name.to_owned();
                let bar = tokio::task::spawn_blocking(move || iced_bar::run(name_clone));
                manager.bars.insert(name.clone(), bar);
            }
        }
    }
}
