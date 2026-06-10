use std::collections::{HashMap, HashSet};
use std::time::Duration;

use rustybar::{iced_bar, producer::niri};
use tokio::task::JoinHandle;

const BAR_RESTART_DELAY: Duration = Duration::from_secs(1);

#[derive(Default)]
struct BarManager {
    bars: HashMap<String, JoinHandle<()>>,
}

/// Run the iced bar for `output`, restarting it on exit.
async fn supervise_bar(output: String) {
    loop {
        let o = output.clone();
        match tokio::task::spawn_blocking(move || iced_bar::run(o)).await {
            Ok(Ok(())) => {
                eprintln!("bar: bar for {output:?} exited cleanly");
                break;
            }
            Ok(Err(e)) => eprintln!("bar: bar for {output:?} errored, restarting: {e:?}"),
            Err(e) => eprintln!("bar: bar for {output:?} panicked, restarting: {e}"),
        }
        tokio::time::sleep(BAR_RESTART_DELAY).await;
    }
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
                eprintln!("bar: aborting bar for removed output {name:?}");
                jh.abort();
                false
            }
        });

        for name in output_names {
            if !manager.bars.contains_key(name) {
                eprintln!("bar: spawning bar for new output {name:?}");
                let name_clone = name.to_owned();
                let bar = tokio::spawn(supervise_bar(name_clone));
                manager.bars.insert(name.clone(), bar);
            }
        }
    }
}
