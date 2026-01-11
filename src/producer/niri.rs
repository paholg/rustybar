use std::{collections::HashMap, sync::LazyLock};

use niri_ipc::{
    Request, Workspace,
    socket::Socket,
    state::{EventStreamState, EventStreamStatePart},
};
use tokio::sync::watch;

#[derive(Debug)]
pub struct Message {
    pub outputs: HashMap<String, Output>,
}

#[derive(Debug, Default)]
pub struct Output {
    pub workspaces: Vec<Workspace>,
    pub window: String,
}

fn produce(state: &EventStreamState) -> Message {
    let mut outputs = HashMap::new();

    for ws in state.workspaces.workspaces.values() {
        let Some(output_name) = ws.output.clone() else {
            continue;
        };

        let output = outputs.entry(output_name).or_insert_with(Output::default);

        if let Some(id) = ws.active_window_id
            && ws.is_active
        {
            if let Some(window) = state.windows.windows.get(&id) {
                output.window = window.title.clone().unwrap_or_default();
            }
        }

        output.workspaces.push(ws.clone());
    }

    for (_, output) in outputs.iter_mut() {
        output.workspaces.sort_by_key(|ws| ws.idx);
    }

    Message { outputs }
}

pub fn listen() -> watch::Receiver<Message> {
    static SENDER: LazyLock<watch::Sender<Message>> = LazyLock::new(|| {
        let mut socket = Socket::connect().unwrap();
        let mut state = EventStreamState::default();
        let init = produce(&state);
        let (sender, _) = watch::channel(init);

        let s = sender.clone();

        tokio::task::spawn_blocking(move || {
            socket.send(Request::EventStream).unwrap().unwrap();
            let mut read_event = socket.read_events();

            loop {
                let event = read_event().unwrap();
                state.apply(event);
                sender.send(produce(&state)).unwrap();
            }
        });
        s
    });

    SENDER.subscribe()
}
