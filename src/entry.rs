use std::cmp::Ordering;
use std::env;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;

use clap::Parser;
use indexmap::map::IndexMap;
use lenient_bool::LenientBool;
use regex::Regex;
use serde::{Deserialize, Serialize};
use skim::prelude::*;

use crate::history::{load_history, save_history};
use crate::options::{AccentColor, Cli};

lazy_static! {
    static ref RE_WHATIS: Regex = Regex::new(r"(?m)^.*?\s+-\s+").unwrap();
    pub static ref OPTIONS: Cli = Cli::parse();
    static ref MATCH_GENERIC_NAME: bool = OPTIONS.match_generic_name;
    static ref SHOW_GENERIC_NAME: bool = OPTIONS.show_generic_name;
    static ref ACCENT_COLOR: u8 = get_accent_color();
}

fn get_app_dirs() -> Vec<PathBuf> {
    let app_dirs_base = xdg::BaseDirectories::with_prefix("applications").unwrap();
    let mut app_dirs = vec![app_dirs_base.get_data_home()];
    app_dirs.extend(app_dirs_base.get_data_dirs());
    app_dirs
        .into_iter()
        .filter(|d| d.is_dir())
        .collect::<Vec<PathBuf>>()
}

fn get_paths() -> Vec<PathBuf> {
    let mut result: Vec<PathBuf> = Vec::new();
    match env::var_os("PATH") {
        Some(paths) => {
            for path in env::split_paths(&paths) {
                if path.is_dir() {
                    result.push(path);
                }
            }
        }
        None => eprintln!("$PATH is not defined in the environment"),
    }
    result
}

fn get_mtime(file: &PathBuf) -> f64 {
    fs::metadata(file)
        .expect("Failed to check metadata")
        .modified()
        .unwrap()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Entry {
    path: String,
    mtime: Option<f64>,
    name: String,
    pub exec: String,
    generic_name: Option<String>,
    comment: Option<String>,
    pub terminal: bool,
    pub desktop: bool,
    pub count: u32,
}

type EntryMap = IndexMap<String, Entry>;

fn get_accent_color() -> u8 {
    match OPTIONS.accent_color.unwrap_or(AccentColor::Magenta) {
        AccentColor::Black => 0,
        AccentColor::Red => 1,
        AccentColor::Green => 2,
        AccentColor::Yellow => 3,
        AccentColor::Blue => 4,
        AccentColor::Magenta => 5,
        AccentColor::Cyan => 6,
        AccentColor::White => 7,
    }
}

impl Entry {
    pub fn new() -> Self {
        Entry {
            path: "".to_string(),
            mtime: None,
            name: "".to_string(),
            exec: "".to_string(),
            generic_name: None,
            comment: None,
            terminal: false,
            desktop: false,
            count: 0,
        }
    }
}

impl SkimItem for Entry {
    fn text(&self) -> Cow<str> {
        if self.desktop {
            if !*MATCH_GENERIC_NAME {
                return Cow::Borrowed(&self.name);
            }
            match &self.generic_name {
                Some(gname) => Cow::Owned(format!("{}, {}", self.name, gname)),
                None => Cow::Borrowed(&self.name),
            }
        } else {
            Cow::Borrowed(&self.name)
        }
    }

    fn display<'a>(&self, context: DisplayContext<'a>) -> AnsiString<'a> {
        // Shift highlight char position by icon width
        let icon = if self.desktop {
            "\u{f108}  "
        } else {
            "\u{f120}  "
        };
        let icon_shift: usize = 3;
        let text;
        if *SHOW_GENERIC_NAME {
            match &self.generic_name {
                Some(gname) => text = format!("{}{}, {}", icon, self.name, gname),
                None => text = format!("{}{}", icon, self.name),
            }
        } else {
            text = format!("{}{}", icon, self.name);
        }
        match context.matches {
            Matches::CharIndices(indices) => {
                if indices.is_empty() {
                    return AnsiString::new_string(text, vec![]);
                }
                let fragments = indices
                    .iter()
                    .map(|&i| {
                        (
                            context.highlight_attr,
                            ((i + icon_shift) as u32, (1 + i + icon_shift) as u32),
                        )
                    })
                    .collect();
                AnsiString::new_string(text, fragments)
            }
            Matches::CharRange(s, e) => {
                let empty = s == e;
                let start = if empty { s } else { s + icon_shift };
                let end = if empty { e } else { e + icon_shift + 1 };
                AnsiString::new_string(
                    text,
                    vec![(context.highlight_attr, (start as u32, end as u32))],
                )
            }
            Matches::ByteRange(start, end) => {
                let s = context.text[start..end].chars().count();
                let e = s + context.text[start..end].chars().count();
                let empty = s == e;
                let start = if empty { s } else { s + icon_shift };
                let end = if empty { e } else { e + icon_shift + 1 };
                AnsiString::new_string(
                    text,
                    vec![(context.highlight_attr, (start as u32, end as u32))],
                )
            }
            Matches::None => AnsiString::new_string(text, vec![]),
        }
    }

    fn output(&self) -> Cow<str> {
        Cow::Borrowed(&self.path)
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        let mut text = String::new();
        write!(text, "\x1b[3{}m{}\x1b[m", *ACCENT_COLOR, self.name).unwrap();
        if self.desktop {
            match &self.generic_name {
                Some(gname) => write!(text, " | {}", gname).unwrap(),
                None => {}
            }
            match &self.comment {
                Some(comment) => write!(text, "\n{}", comment).unwrap(),
                None => {}
            }
        } else {
            let output = Command::new("whatis")
                .arg("--long")
                .arg(&self.path)
                .output()
                .unwrap_or_else(|_| panic!("Failed to read man of command: {}", self.path));
            if output.status.success() {
                let comment = String::from_utf8(output.stdout).unwrap();
                write!(text, "\n{}", RE_WHATIS.replace_all(&comment, "")).unwrap();
            }
        }
        ItemPreview::AnsiText(text)
    }
}

pub fn entry_cmp(_k1: &String, v1: &Entry, _k2: &String, v2: &Entry) -> Ordering {
    v1.name.cmp(&v2.name)
}

pub fn load_bin_entries(history: &EntryMap) -> EntryMap {
    let mut result: EntryMap = IndexMap::new();
    let paths = get_paths();
    for dir in paths.iter() {
        let mut entries: EntryMap = IndexMap::new();
        for file in dir
            .read_dir()
            .unwrap()
            .map(|f| f.expect("Failed to read file").path())
        {
            if !file.is_file() {
                continue;
            }
            entries.insert(
                file.to_str().unwrap().to_string(),
                load_bin_entry(&file, history),
            );
        }
        entries.sort_by(entry_cmp);
        result.extend(entries);
    }
    result
}

fn load_bin_entry(file: &PathBuf, history: &EntryMap) -> Entry {
    let mut entry = Entry::new();
    let filestr = file.to_str().unwrap().to_string();
    let filename = file.file_name().unwrap().to_str().unwrap().to_string();
    if let Some(e) = history.get(&filestr) {
        entry.count = e.count;
    }
    entry.path = filestr;
    entry.name = filename.clone();
    entry.exec = filename.clone();
    entry
}

pub fn load_desktop_entries(history: &EntryMap) -> EntryMap {
    let mut result: EntryMap = IndexMap::new();
    let app_dirs = get_app_dirs();
    for dir in app_dirs.iter() {
        let entries = load_desktop_entry_dir(dir, history);
        result.extend(entries);
    }
    result.sort_by(entry_cmp);
    result
}

fn load_desktop_entry_dir(dir: &PathBuf, history: &EntryMap) -> EntryMap {
    let mut entries: EntryMap = IndexMap::new();
    for path in dir
        .read_dir()
        .unwrap()
        .map(|f| f.expect("Failed to read file").path())
    {
        if path.is_dir() {
            entries.extend(load_desktop_entry_dir(&path, history));
        } else {
            let file = path;
            match file.extension() {
                Some(ext) => {
                    if ext != "desktop" {
                        continue;
                    }
                }
                None => continue,
            }
            match load_desktop_entry_file(&file, history) {
                Some(entry) => {
                    entries.insert(file.to_str().unwrap().to_string(), entry);
                }
                None => continue,
            }
        }
    }
    entries
}

fn load_desktop_entry_file(file: &PathBuf, history: &EntryMap) -> Option<Entry> {
    // check file modified time and if it's not modified since prev access, return cached entry
    let count;
    let mtime = get_mtime(file);
    let filestr = file.to_str().unwrap().to_string();
    if history.contains_key(&filestr) {
        if history.get(&filestr).unwrap().mtime.unwrap() == mtime {
            return Some(history.get(&filestr).unwrap().clone());
        } else {
            count = history.get(&filestr).unwrap().count;
        }
    } else {
        count = 0;
    }

    // desktop entry file is modified or added. load it.
    let conf = match ini::Ini::load_from_file(file) {
        Ok(c) => c,
        Err(_) => return None,
    };
    let section = match conf.section(Some("Desktop Entry")) {
        Some(s) => s,
        None => return None,
    };

    // create new entry from desktop entry
    let mut entry = Entry::new();
    entry.desktop = true;
    entry.path = filestr;
    entry.count = count;
    entry.mtime = Some(mtime);
    match section.get("Name") {
        Some(name) => entry.name = name.to_string(),
        _ => return None,
    }
    match section.get("Exec") {
        Some(exec) => entry.exec = exec.to_string(),
        _ => return None,
    }
    match section.get("GenericName") {
        Some(gname) => entry.generic_name = Some(gname.to_string()),
        None => entry.generic_name = None,
    }
    match section.get("Comment") {
        Some(comment) => entry.comment = Some(comment.to_string()),
        None => entry.comment = None,
    }
    match section.get("Terminal") {
        Some(terminal) => entry.terminal = terminal.parse::<LenientBool>().unwrap().into(),
        None => entry.terminal = false,
    }
    Some(entry)
}

pub fn load_entries() -> EntryMap {
    let history: EntryMap = load_history();
    let mut entries: EntryMap = load_desktop_entries(&history);
    entries.extend(load_bin_entries(&history));
    save_history(&entries);

    entries
}
