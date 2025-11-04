use std::env;
use std::process::{Command, Stdio};

use indexmap::IndexMap;
use regex::Regex;

use crate::entry::{Entry, OPTIONS};
use crate::history::save_history;

lazy_static! {
    static ref RE_EXEC_OPT: Regex = Regex::new(r"\s*%\w").unwrap();
}

pub fn execute_raw(cmd: String) {
    _exec(cmd.trim());
}

pub fn execute(pathstr: String, entries: &mut IndexMap<String, Entry>) {
    let entry = entries.get_mut(&pathstr).unwrap();
    entry.count += 1;
    let entry = entry.clone();
    save_history(entries);

    if !entry.desktop {
        exec_command(entry);
    } else if !entry.terminal {
        exec_app(entry);
    } else {
        exec_term(entry);
    }
}

// Execute command from bin entry
fn exec_command(entry: Entry) {
    let cmd = entry.exec.trim();
    _exec(cmd);
}

// Run app from desktop entry, not terminal app
fn exec_app(entry: Entry) {
    let cmd = entry.exec.trim();
    _exec(&RE_EXEC_OPT.replace_all(cmd, ""));
}

// Run terminal app from desktop entry
fn exec_term(entry: Entry) {
    let cmd = RE_EXEC_OPT.replace_all(entry.exec.trim(), "").into_owned();

    let mut term_cmd: Vec<String> = Vec::new();
    match &OPTIONS.terminal_command {
        Some(val) => {
            term_cmd.extend(shlex::split(val).expect("Failed to parse --terminal-command option"))
        }
        None => match env::var_os("TERM") {
            Some(val) => term_cmd = vec![val.to_str().unwrap().to_string(), "-e".to_string()],
            None => term_cmd = vec!["alacritty".to_string(), "-e".to_string()],
        },
    }
    term_cmd.push(cmd);

    // convert Vec<String> to Iter<&str> and join to a single String
    let command = shlex::join(term_cmd.iter().map(String::as_str));
    _exec(&command);
}

fn _exec(cmd: &str) {
    Command::new("setsid")
        .arg("sh")
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start command");
}
