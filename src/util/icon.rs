use std::{
    fs,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

const EXTENSIONS: [&str; 2] = ["svg", "png"];

/// Icon theme directories (each containing e.g. `hicolor/`), in search order.
static THEME_DIRS: LazyLock<Vec<PathBuf>> = LazyLock::new(|| {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let data_home = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| home.as_ref().map(|h| h.join(".local/share")));
    let data_dirs = std::env::var_os("XDG_DATA_DIRS")
        .map(|dirs| std::env::split_paths(&dirs).collect::<Vec<_>>())
        .unwrap_or_default();

    let mut dirs: Vec<PathBuf> = data_home
        .into_iter()
        .chain(data_dirs)
        .chain(home.map(|h| h.join(".nix-profile/share")))
        .chain([PathBuf::from("/run/current-system/sw/share")])
        .map(|d| d.join("icons"))
        .filter(|d| d.is_dir())
        .collect();
    dirs.dedup();
    dirs
});

fn find_in_dir(dir: &Path, name: &str) -> Option<PathBuf> {
    EXTENSIONS
        .iter()
        .map(|ext| dir.join(format!("{name}.{ext}")))
        .find(|path| path.is_file())
}

/// Sort key for a theme size directory: `scalable` first, then largest
/// pixel size first, then anything unrecognized.
fn size_rank(dir: &Path) -> (u8, i64) {
    let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if name == "scalable" {
        return (0, 0);
    }
    let px = name
        .split(['x', '@'])
        .next()
        .and_then(|n| n.parse::<i64>().ok());
    match px {
        Some(px) => (1, -px),
        None => (2, 0),
    }
}

/// Search a theme directory like `.../icons/hicolor`, which contains
/// `<size>/<context>/<name>.<ext>`, preferring larger icons.
fn find_in_theme(theme: &Path, name: &str) -> Option<PathBuf> {
    let mut sizes: Vec<PathBuf> = fs::read_dir(theme)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    sizes.sort_by_key(|p| size_rank(p));

    sizes
        .iter()
        .flat_map(|size| fs::read_dir(size).ok())
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find_map(|context| find_in_dir(&context, name))
}

/// Search a `share` directory: its hicolor theme, then pixmaps.
fn find_in_share(share: &Path, name: &str) -> Option<PathBuf> {
    find_in_theme(&share.join("icons/hicolor"), name)
        .or_else(|| find_in_dir(&share.join("pixmaps"), name))
}

/// Find an icon file by freedesktop icon name.
///
/// `extra_root` is searched first, both directly and as an icon theme root.
/// This is what `StatusNotifierItem`s pass as `IconThemePath`.
pub fn find_icon(name: &str, extra_root: Option<&str>) -> Option<PathBuf> {
    // Some apps pass a full path instead of a name.
    let as_path = Path::new(name);
    if as_path.is_absolute() && as_path.is_file() {
        return Some(as_path.to_path_buf());
    }

    if let Some(root) = extra_root.map(Path::new)
        && let Some(path) = find_in_dir(root, name)
            .or_else(|| find_in_theme(&root.join("hicolor"), name))
            .or_else(|| find_in_theme(root, name))
    {
        return Some(path);
    }

    if let Some(path) = THEME_DIRS
        .iter()
        .find_map(|dir| find_in_theme(&dir.join("hicolor"), name))
    {
        return Some(path);
    }
    if let Some(path) = find_in_dir(Path::new("/run/current-system/sw/share/pixmaps"), name) {
        return Some(path);
    }

    // On NixOS, try to find icon in the app's nix store path
    let bin_path = std::process::Command::new("which")
        .arg(name)
        .output()
        .ok()
        .filter(|out| out.status.success())?;
    let bin_path = String::from_utf8_lossy(&bin_path.stdout).trim().to_string();
    let resolved = fs::canonicalize(&bin_path).ok()?;
    // Go up to the nix store package root (2 levels up from bin/)
    let store_path = resolved.ancestors().nth(2)?;

    find_in_share(&store_path.join("share"), name)
        // lib/{app}/logo/ is used by kitty
        .or_else(|| find_in_dir(&store_path.join(format!("lib/{name}/logo")), name))
}

pub fn load_icon_data_url(path: &PathBuf) -> Option<String> {
    let data = fs::read(path).ok()?;
    let ext = path.extension()?.to_str()?;

    let mime = match ext {
        "svg" => "image/svg+xml",
        "png" => "image/png",
        _ => return None,
    };

    Some(format!("data:{};base64,{}", mime, BASE64.encode(&data)))
}
