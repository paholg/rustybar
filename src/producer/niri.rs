use std::{collections::HashMap, sync::LazyLock, time::Duration};

use niri_ipc::{
    Request, Window, Workspace,
    socket::Socket,
    state::{EventStreamState, EventStreamStatePart},
};
use tokio::sync::watch;

const RECONNECT_DELAY: Duration = Duration::from_secs(1);

#[derive(Debug)]
pub struct Message {
    pub outputs: HashMap<String, Output>,
}

#[derive(Debug, Default)]
pub struct Output {
    pub workspaces: Vec<Workspace>,
    pub window: String,
    pub workspace_windows: Vec<Window>,
}

fn produce(state: &EventStreamState) -> Message {
    let mut outputs = HashMap::new();

    for ws in state.workspaces.workspaces.values() {
        let Some(output_name) = ws.output.clone() else {
            continue;
        };

        let output = outputs
            .entry(output_name.clone())
            .or_insert_with(Output::default);

        if let Some(id) = ws.active_window_id
            && ws.is_active
            && let Some(window) = state.windows.windows.get(&id)
        {
            output.window = window.title.clone().unwrap_or_default();
        }

        output.workspaces.push(ws.clone());
    }

    for (_, output) in outputs.iter_mut() {
        output.workspaces.sort_by_key(|ws| ws.idx);

        let active_workspace_id = output.workspaces.iter().find(|ws| ws.is_active).unwrap().id;
        output.workspace_windows = state
            .windows
            .windows
            .values()
            .filter(|w| w.workspace_id == Some(active_workspace_id))
            .cloned()
            .collect();
        output
            .workspace_windows
            .sort_by_key(|w| w.layout.pos_in_scrolling_layout);
    }

    Message { outputs }
}

pub fn listen() -> watch::Receiver<Message> {
    static SENDER: LazyLock<watch::Sender<Message>> = LazyLock::new(|| {
        let init = produce(&EventStreamState::default());
        let (sender, _) = watch::channel(init);

        let s = sender.clone();

        tokio::task::spawn_blocking(move || {
            loop {
                if let Err(e) = run_stream(&sender) {
                    eprintln!("niri: event stream ended, reconnecting: {e}");
                }
                std::thread::sleep(RECONNECT_DELAY);
            }
        });
        s
    });

    SENDER.subscribe()
}

/// Connect to niri and pump events into `sender` until the stream errors out.
fn run_stream(sender: &watch::Sender<Message>) -> eyre::Result<()> {
    let mut socket = Socket::connect()?;
    let mut state = EventStreamState::default();

    socket
        .send(Request::EventStream)?
        .map_err(|e| eyre::eyre!("niri rejected EventStream request: {e}"))?;
    let mut read_event = socket.read_events();

    loop {
        let event = match read_event() {
            Ok(event) => event,
            // This is likely an unkown event from a newer version of niri.
            Err(e) if e.kind() == std::io::ErrorKind::InvalidData => continue,
            Err(e) => return Err(e.into()),
        };
        state.apply(event);
        let msg = produce(&state);
        // All receivers gone means the app is shutting down; stop the stream.
        if sender.send(msg).is_err() {
            return Ok(());
        }
    }
}
