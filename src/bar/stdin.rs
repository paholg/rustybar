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
        String::new()
    }

    fn box_clone(&self) -> crate::bar::DynBar {
        Box::new(self.clone())
    }

    fn update_on(&self) -> crate::bar::UpdateOn {
        crate::bar::UpdateOn::Stdin
    }
}

pub(crate) mod state {
    use tokio::io::AsyncBufReadExt;
    use tokio::stream::StreamExt;
    use tokio::{io, sync};

    pub(crate) struct State {
        bars_to_update: Vec<crate::bar::RunningBar>,
    }

    impl State {
        fn new() -> State {
            State {
                bars_to_update: Vec::new(),
            }
        }

        // These should be part of a trait for state tracking (as well as update_bars):
        pub(crate) fn register_bar(&mut self, bar: crate::bar::RunningBar) {
            self.bars_to_update.push(bar);
        }

        pub(crate) fn clear_bars(&mut self) {
            self.bars_to_update.clear();
        }
    }

    lazy_static::lazy_static! {
        pub(crate) static ref STDIN: sync::RwLock<State> = sync::RwLock::new(State::new());
    }

    pub async fn run() -> io::Result<()> {
        let mut lines = io::BufReader::new(io::stdin()).lines();
        while let Some(line) = lines.next().await {
            let mut line = line.unwrap();
            line.push('\n');
            let mut state = STDIN.write().await;
            let bars = state.bars_to_update.iter_mut();
            crate::bar::update_bars(bars, std::iter::repeat(&line)).await?;
        }

        unreachable!()
    }
}
pub use state::run;
