use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    sync::{LazyLock, OnceLock},
    time::Duration,
};

use iced::widget::{image, svg};
use system_tray::{
    client::{ActivateRequest, Client},
    item::{IconPixmap, Status, StatusNotifierItem},
    menu::{MenuItem, MenuType, ToggleState, ToggleType},
};
use tokio::sync::watch;

use crate::util::icon::find_icon;

const RECONNECT_DELAY: Duration = Duration::from_secs(1);

#[derive(Debug, Clone)]
pub enum Icon {
    Raster(image::Handle),
    Svg(svg::Handle),
    Missing,
}

#[derive(Debug, Clone)]
pub struct Item {
    /// The item's bus name, used to send it events.
    pub address: String,
    pub id: String,
    /// Hover text: the tooltip title, falling back to the item title or id.
    pub title: String,
    pub icon: Icon,
    /// DBus object path of the item's menu, needed to click entries.
    pub menu_path: Option<String>,
    pub menu: Vec<MenuEntry>,
}

#[derive(Debug, Clone)]
pub struct MenuEntry {
    pub id: i32,
    pub label: String,
    pub enabled: bool,
    pub separator: bool,
    /// `Some` for checkable entries.
    pub checked: Option<bool>,
    pub children: Vec<MenuEntry>,
}

#[derive(Debug, Default)]
pub struct Message {
    /// Non-passive items, sorted by id.
    pub items: Vec<Item>,
}

static CLIENT: OnceLock<Client> = OnceLock::new();

static SENDER: LazyLock<watch::Sender<Message>> = LazyLock::new(|| {
    let (sender, _) = watch::channel(Message::default());

    let s = sender.clone();

    tokio::spawn(async move {
        loop {
            if let Err(e) = run(&sender).await {
                eprintln!("tray: client stopped, retrying: {e}");
            }
            tokio::time::sleep(RECONNECT_DELAY).await;
        }
    });
    s
});

pub fn listen() -> watch::Receiver<Message> {
    SENDER.subscribe()
}

/// The latest snapshot's entry for `address`, if it still exists.
pub fn item(address: &str) -> Option<Item> {
    SENDER
        .borrow()
        .items
        .iter()
        .find(|item| item.address == address)
        .cloned()
}

/// Tell the item its menu is about to be shown, so lazy apps populate it.
pub async fn menu_about_to_show(address: String, menu_path: String) {
    let Some(client) = CLIENT.get() else {
        return;
    };
    if let Err(e) = client.about_to_show_menuitem(address, menu_path, 0).await {
        eprintln!("tray: about_to_show failed: {e}");
    }
}

pub async fn menu_click(address: String, menu_path: String, submenu_id: i32) {
    let Some(client) = CLIENT.get() else {
        return;
    };
    let req = ActivateRequest::MenuItem {
        address,
        menu_path,
        submenu_id,
    };
    if let Err(e) = client.activate(req).await {
        eprintln!("tray: menu click failed: {e}");
    }
}

/// Send a click to the item at `address`. Does nothing if the tray client
/// hasn't connected yet.
pub async fn activate(address: String, secondary: bool) {
    let Some(client) = CLIENT.get() else {
        return;
    };
    // The coordinates are only a hint for where the item may open a window.
    let req = if secondary {
        ActivateRequest::Secondary {
            address,
            x: 0,
            y: 0,
        }
    } else {
        ActivateRequest::Default {
            address,
            x: 0,
            y: 0,
        }
    };
    if let Err(e) = client.activate(req).await {
        eprintln!("tray: activate failed: {e}");
    }
}

async fn client() -> eyre::Result<&'static Client> {
    if let Some(client) = CLIENT.get() {
        return Ok(client);
    }
    let client = Client::new().await?;
    Ok(CLIENT.get_or_init(|| client))
}

/// Connect to the tray and push a snapshot into `sender` on every change.
async fn run(sender: &watch::Sender<Message>) -> eyre::Result<()> {
    let client = client().await?;
    let mut events = client.subscribe();
    let mut icons = IconCache::default();

    loop {
        sender.send(snapshot(client, &mut icons))?;
        match events.recv().await {
            // Lagging just means we take a fresh snapshot.
            Ok(_) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
            Err(e) => return Err(e.into()),
        }
    }
}

fn snapshot(client: &Client, icons: &mut IconCache) -> Message {
    let mut items: Vec<Item> = client
        .items()
        .lock()
        .unwrap()
        .iter()
        .filter(|(_, (item, _))| item.status != Status::Passive)
        .map(|(address, (item, menu))| Item {
            address: address.clone(),
            id: item.id.clone(),
            title: title(item),
            icon: icons.get(address, item),
            menu_path: item.menu.clone(),
            menu: menu
                .as_ref()
                .map(|menu| menu_entries(&menu.submenus))
                .unwrap_or_default(),
        })
        .collect();
    icons.retain(&items);
    items.sort_by(|a, b| a.id.cmp(&b.id).then(a.address.cmp(&b.address)));
    Message { items }
}

fn title(item: &StatusNotifierItem) -> String {
    item.tool_tip
        .as_ref()
        .map(|t| t.title.as_str())
        .filter(|t| !t.is_empty())
        .or(item.title.as_deref())
        .filter(|t| !t.is_empty())
        .unwrap_or(&item.id)
        .to_owned()
}

fn menu_entries(items: &[MenuItem]) -> Vec<MenuEntry> {
    items
        .iter()
        .filter(|item| item.visible)
        .map(|item| MenuEntry {
            id: item.id,
            label: item.label.clone().unwrap_or_default(),
            enabled: item.enabled,
            separator: item.menu_type == MenuType::Separator,
            checked: match item.toggle_type {
                ToggleType::Checkmark | ToggleType::Radio => {
                    Some(item.toggle_state == ToggleState::On)
                }
                ToggleType::CannotBeToggled => None,
            },
            children: menu_entries(&item.submenu),
        })
        .collect()
}

/// Icon handles keyed by item address. Reusing a handle lets iced keep the
/// decoded image cached instead of re-uploading it on every event.
#[derive(Default)]
struct IconCache {
    map: HashMap<String, (u64, Icon)>,
}

impl IconCache {
    fn get(&mut self, address: &str, item: &StatusNotifierItem) -> Icon {
        let (name, pixmap) = icon_source(item);

        let mut hasher = DefaultHasher::new();
        (name, pixmap, &item.icon_theme_path).hash(&mut hasher);
        let key = hasher.finish();

        match self.map.get(address) {
            Some((k, icon)) if *k == key => icon.clone(),
            _ => {
                let icon = load_icon(name, pixmap, item.icon_theme_path.as_deref());
                self.map.insert(address.to_owned(), (key, icon.clone()));
                icon
            }
        }
    }

    fn retain(&mut self, items: &[Item]) {
        self.map
            .retain(|address, _| items.iter().any(|item| &item.address == address));
    }
}

/// The name and pixmaps to draw for `item`, taking its status into account.
fn icon_source(item: &StatusNotifierItem) -> (Option<&str>, Option<&[IconPixmap]>) {
    let attention = item.status == Status::NeedsAttention;
    let name = attention
        .then_some(item.attention_icon_name.as_deref())
        .flatten()
        .or(item.icon_name.as_deref())
        .filter(|name| !name.is_empty());
    let pixmap = attention
        .then_some(item.attention_icon_pixmap.as_deref())
        .flatten()
        .or(item.icon_pixmap.as_deref())
        .filter(|pixmaps| !pixmaps.is_empty());
    (name, pixmap)
}

/// The spec says to prefer names over pixmaps, but a name is only usable if we
/// can find it on disk; pixmaps always work.
fn load_icon(name: Option<&str>, pixmaps: Option<&[IconPixmap]>, theme_path: Option<&str>) -> Icon {
    if let Some(path) = name.and_then(|name| find_icon(name, theme_path)) {
        return if path.extension().is_some_and(|ext| ext == "svg") {
            Icon::Svg(svg::Handle::from_path(path))
        } else {
            Icon::Raster(image::Handle::from_path(path))
        };
    }
    if let Some(pixmap) = pixmaps.and_then(largest) {
        return Icon::Raster(rgba_handle(pixmap));
    }
    Icon::Missing
}

fn largest(pixmaps: &[IconPixmap]) -> Option<&IconPixmap> {
    pixmaps
        .iter()
        .filter(|p| p.width > 0 && p.height > 0)
        .filter(|p| p.pixels.len() == (p.width * p.height * 4) as usize)
        .max_by_key(|p| p.width * p.height)
}

/// Pixmaps are ARGB32 in network byte order, so each pixel's bytes are
/// `[a, r, g, b]`.
fn rgba_handle(pixmap: &IconPixmap) -> image::Handle {
    let rgba: Vec<u8> = pixmap
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|[a, r, g, b]| [*r, *g, *b, *a])
        .collect();
    image::Handle::from_rgba(pixmap.width as u32, pixmap.height as u32, rgba)
}
