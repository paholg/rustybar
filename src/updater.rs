use crate::bar::RunningBar;
use async_trait::async_trait;
use std::io;

#[async_trait]
pub trait Updater: std::fmt::Debug + Sync + Send {
    /// Register a bar to be updated by this Updater.
    async fn register(&self, bar: RunningBar);

    /// Clear all bars from this Updater.
    async fn clear(&self);

    /// Wait until ready to update state, then do so.
    async fn update_state(&self);

    /// Set a flag to true to indicate that this Updater is running. That flag should not be mutated
    /// outside this method.
    async fn mark_running(&self);

    /// Return whether this Updater is running.
    async fn running(&self) -> bool;

    // TODO: This should not be part of the trait, but should be a function for all Updaters.
    async fn run(&self);
}

pub(crate) async fn render_bars(bars: impl Iterator<Item = &RunningBar>) -> Vec<String> {
    let mut res = Vec::new();
    for rb in bars {
        let mut string = rb.bar.render().await;
        string.push('\n');
        res.push(string);
    }

    res
}

pub(crate) async fn update_bars(
    bars: impl Iterator<Item = &mut RunningBar>,
    strings: impl Iterator<Item = &String>,
) -> io::Result<()> {
    for (rb, string) in bars.zip(strings) {
        rb.write(string.as_bytes()).await?;
    }

    Ok(())
}
