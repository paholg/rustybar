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

/// Renders a workspace's bar label exactly like the workspace consumer does:
/// its name, or its index when unnamed. Kept in sync with `consumer::workspace`.
#[cfg(test)]
fn label(ws: &Workspace) -> String {
    ws.name
        .as_deref()
        .map(|s| s.to_string())
        .unwrap_or_else(|| ws.idx.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn ws(id: u64, idx: u8, name: Option<&str>, output: &str, active: bool) -> Workspace {
        Workspace {
            id,
            idx,
            name: name.map(str::to_string),
            output: Some(output.to_string()),
            is_urgent: false,
            is_active: active,
            is_focused: active,
            active_window_id: None,
        }
    }

    fn state(workspaces: Vec<Workspace>) -> EventStreamState {
        let mut s = EventStreamState::default();
        s.workspaces.workspaces = workspaces.into_iter().map(|w| (w.id, w)).collect();
        s
    }

    /// The bar labels for `output`, in display order, after `produce`.
    fn labels(state: &EventStreamState, output: &str) -> Vec<String> {
        produce(state).outputs[output]
            .workspaces
            .iter()
            .map(label)
            .collect()
    }

    /// The reported scenario: one `main`, unique indices, single output.
    /// `produce` must yield each workspace exactly once, in index order.
    #[test]
    fn single_output_unique_indices_no_duplicates() {
        let st = state(vec![
            ws(10, 1, Some("devc"), "DP-1", false),
            ws(11, 2, Some("main"), "DP-1", true),
            ws(12, 3, Some("pread"), "DP-1", false),
        ]);
        assert_eq!(labels(&st, "DP-1"), ["devc", "main", "pread"]);
    }

    /// `produce` reads from a `HashMap`, whose iteration order Rust randomizes.
    /// The output order must be stable across many independent builds.
    #[test]
    fn output_order_is_deterministic() {
        let workspaces = vec![
            ws(101, 1, Some("devc"), "DP-1", false),
            ws(102, 2, Some("main"), "DP-1", true),
            ws(103, 3, Some("pread"), "DP-1", false),
            ws(104, 4, Some("scratch"), "DP-1", false),
            ws(105, 5, None, "DP-1", false),
        ];
        let expected = labels(&state(workspaces.clone()), "DP-1");
        for _ in 0..200 {
            // rebuild the state so the HashMap gets a fresh iteration order
            assert_eq!(labels(&state(workspaces.clone()), "DP-1"), expected);
        }
    }

    /// A `main` that also exists on a *different* monitor (niri allows the same
    /// idx and name across monitors) must not leak into this output's bar.
    #[test]
    fn other_monitor_workspaces_do_not_leak() {
        let st = state(vec![
            ws(1, 1, Some("devc"), "DP-1", false),
            ws(2, 2, Some("main"), "DP-1", true),
            ws(3, 3, Some("pread"), "DP-1", false),
            // a second "main" on another output, same idx
            ws(4, 2, Some("main"), "HDMI-1", true),
        ]);
        assert_eq!(labels(&st, "DP-1"), ["devc", "main", "pread"]);
        let dp1_mains = labels(&st, "DP-1").iter().filter(|l| *l == "main").count();
        assert_eq!(dp1_mains, 1, "duplicate main leaked into DP-1");
    }

    /// If niri ever reports two workspaces with the same idx on one output,
    /// the stable sort falls back to HashMap order — pin down that this is the
    /// only way `produce` can reorder, by checking it stays sorted by idx.
    #[test]
    fn output_is_sorted_by_idx() {
        let st = state(vec![
            ws(1, 3, Some("pread"), "DP-1", false),
            ws(2, 1, Some("devc"), "DP-1", false),
            ws(3, 2, Some("main"), "DP-1", true),
        ]);
        let idxs: Vec<u8> = produce(&st).outputs["DP-1"]
            .workspaces
            .iter()
            .map(|w| w.idx)
            .collect();
        assert_eq!(idxs, [1, 2, 3]);
    }
}
