use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;

use indexmap::map::IndexMap;

use crate::entry::Entry;

fn get_hist_file() -> PathBuf {
    let base = xdg::BaseDirectories::with_prefix("sklauncher").unwrap();
    let cache_dir = base.get_cache_home();
    if !cache_dir.is_dir() {
        fs::create_dir_all(cache_dir.as_path()).unwrap();
    }
    let hist_file = cache_dir.join("history.toml");
    if !hist_file.is_file() {
        fs::write(hist_file.as_path(), b"").unwrap();
    }
    hist_file
}

pub fn load_history() -> IndexMap<String, Entry> {
    let contents = fs::read_to_string(get_hist_file()).expect("Failed to open history file");
    toml::from_str::<IndexMap<String, Entry>>(&contents).expect("History file is broken")
}

pub fn save_history(history: &IndexMap<String, Entry>) {
    let hist_file_path = get_hist_file();
    let mut file = fs::File::create(hist_file_path).expect("Failed to open history file");
    let contents = toml::to_string::<IndexMap<String, Entry>>(history)
        .expect("Failed convert history to toml format");
    file.write(contents.as_bytes())
        .expect("Failed to write history file");
}
