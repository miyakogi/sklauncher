#[macro_use]
extern crate lazy_static;

use skim::prelude::*;

mod entry;
mod exec;
mod history;
mod options;

use entry::load_entries;
use exec::{execute, execute_raw};
use options::build_options;

fn main() {
    let mut entries = load_entries();
    let options = build_options();

    let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();
    let mut tmp_entries = entries.clone();
    tmp_entries.sort_by(|_k1, v1, _k2, v2| {
        // sort entries by count
        v2.count.cmp(&v1.count)
    });
    for (_k, entry) in tmp_entries.into_iter() {
        drop(tx_item.send(Arc::new(entry)));
    }
    drop(tx_item);

    let output = Skim::run_with(&options, Some(rx_item));

    // error
    if output.is_none() {
        std::process::exit(135);
    }

    // aborted (maybe Esc key is pressed)
    let output = output.unwrap();
    if output.is_abort {
        std::process::exit(130);
    }

    // selected, execute command
    if output.selected_items.is_empty() {
        execute_raw(output.query);
    } else {
        let filestr = output.selected_items[0].output().to_string();
        execute(filestr, &mut entries);
    }
}
