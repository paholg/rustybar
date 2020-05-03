use crate::{stdin, updater::Updater};
use async_trait::async_trait;

/// A statusbar for stdin.
#[derive(Clone, Debug)]
pub struct Stdin {
    width: u32,
}

impl Stdin {
    pub async fn new(padding: u32) -> Box<Stdin> {
        Box::new(Stdin { width: padding })
    }
}

#[async_trait]
impl crate::bar::Bar for Stdin {
    fn width(&self) -> u32 {
        self.width
    }

    async fn render(&self) -> String {
        stdin::Stdin.line().await
    }

    fn box_clone(&self) -> crate::bar::DynBar {
        Box::new(self.clone())
    }

    fn updater(&self) -> Box<dyn Updater> {
        Box::new(stdin::Stdin)
    }
}
