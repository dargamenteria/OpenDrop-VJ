//! Save/load the Playlists panel's current configuration (both decks'
//! item lists, shuffle/sequential mode, interval) to a single JSON file at
//! `<config_dir>/playlists.json`, the same directory `config.rs` uses for
//! `ui.json`. Deliberately separate from that continuous, auto-saved
//! `UiConfig` persistence: this is an explicit "save this set"/"load a
//! set" action, triggered only by the panel's own Save/Load buttons,
//! never read or written automatically at bootstrap or exit.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use opendrop_core::playlist::PlaylistMode;

/// Serde "remote type" shadow for `playlist::PlaylistMode`, same reasoning
/// as `config.rs`'s `ThemeIdWire`: `PlaylistMode` predates this module and
/// has no reason to carry a serde dependency for a persistence need that's
/// local here. Variant names/count must stay in sync with the real enum.
#[derive(Serialize, Deserialize)]
#[serde(remote = "PlaylistMode")]
enum PlaylistModeWire {
    Sequential,
    Shuffle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct SavedPlaylists {
    #[serde(with = "PlaylistModeWire")]
    pub(crate) mode: PlaylistMode,
    pub(crate) interval_sec: f64,
    pub(crate) a_items: Vec<String>,
    pub(crate) b_items: Vec<String>,
}

/// `<config_dir>/playlists.json`, or `None` if the OS gives us no home/
/// config directory at all (mirrors `config::config_file_path`'s own
/// contract).
pub(crate) fn playlists_file_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "opendrop-native").map(|dirs| dirs.config_dir().join("playlists.json"))
}

/// Writes `saved` to `path` as pretty-printed JSON, creating the parent
/// directory if needed.
pub(crate) fn save_playlists(path: &Path, saved: &SavedPlaylists) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(saved).map_err(|e| format!("failed to serialize playlists: {e}"))?;
    std::fs::write(path, json).map_err(|e| format!("failed to write {}: {e}", path.display()))
}

/// Reads and parses `path` back into a `SavedPlaylists`. Unlike
/// `config::load_config`'s degrade-to-default philosophy, a missing or
/// malformed file here is a real, user-visible error (there is no sane
/// default playlist to fall back to), surfaced by the Load button rather
/// than silently swallowed.
pub(crate) fn load_playlists(path: &Path) -> Result<SavedPlaylists, String> {
    let json = std::fs::read_to_string(path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    serde_json::from_str(&json).map_err(|e| format!("failed to parse {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> SavedPlaylists {
        SavedPlaylists {
            mode: PlaylistMode::Shuffle,
            interval_sec: 5.0,
            a_items: vec!["Alpha".to_string(), "Beta".to_string()],
            b_items: vec!["Gamma".to_string()],
        }
    }

    #[test]
    fn save_then_load_round_trips_through_the_real_filesystem() {
        let path = std::env::temp_dir().join(format!("opendrop-playlist-file-test-{}.json", std::process::id()));
        let saved = sample();
        save_playlists(&path, &saved).unwrap();
        let loaded = load_playlists(&path).unwrap();
        assert_eq!(loaded, saved);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_missing_file_is_a_readable_error_not_a_panic() {
        let path = std::env::temp_dir().join("opendrop-playlist-file-test-does-not-exist.json");
        let _ = std::fs::remove_file(&path);
        assert!(load_playlists(&path).is_err());
    }

    #[test]
    fn load_malformed_json_is_a_readable_error_not_a_panic() {
        let path = std::env::temp_dir().join(format!("opendrop-playlist-file-test-malformed-{}.json", std::process::id()));
        std::fs::write(&path, "not json at all").unwrap();
        assert!(load_playlists(&path).is_err());
        let _ = std::fs::remove_file(&path);
    }
}
