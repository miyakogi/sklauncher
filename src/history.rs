use std::fs;
use std::path::PathBuf;

use indexmap::map::IndexMap;

use crate::entry::Entry;

fn get_hist_file() -> PathBuf {
    let base = xdg::BaseDirectories::with_prefix("sklauncher").unwrap();
    let cache_dir = base.get_cache_home();
    if !cache_dir.is_dir() {
        fs::create_dir_all(cache_dir.as_path()).unwrap();
    }
    cache_dir.join("history.toml")
}

pub fn load_history() -> IndexMap<String, Entry> {
    let hist_file = get_hist_file();
    let contents = match fs::read_to_string(&hist_file) {
        Ok(c) => c,
        Err(_) => return IndexMap::new(),
    };
    toml::from_str(&contents).unwrap_or_else(|e| {
        eprintln!("Warning: history file is broken, starting fresh: {e}");
        IndexMap::new()
    })
}

pub fn save_history(history: &IndexMap<String, Entry>) {
    let hist_file = get_hist_file();
    let contents = toml::to_string(&history).expect("Failed to serialize history");
    // Atomic write: write to temp file, then rename
    let tmp_path = hist_file.with_extension("toml.tmp");
    fs::write(&tmp_path, &contents).expect("Failed to write history temp file");
    fs::rename(&tmp_path, &hist_file).expect("Failed to rename history file");
}
