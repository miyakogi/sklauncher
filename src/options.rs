use clap::{Parser, ValueEnum};
use skim::prelude::*;

use crate::entry::OPTIONS;

#[derive(Parser)]
#[command(name = "sklauncher")]
#[command(version, about, author)]
pub struct Cli {
    /// Terminal launch command to be used for a desktop entry with Terminal=True.
    /// By default, `$TERM -e`. If `$TERM` is not defined, `alacritty -e`.
    #[arg(long, value_name = "COMMAND")]
    pub terminal_command: Option<String>,

    /// Show GenericName field of desktop entries
    #[arg(long)]
    pub show_generic_name: bool,

    /// Include GenericName field of desktop entries to match string
    #[arg(long)]
    pub match_generic_name: bool,

    /// Fuzzy Matching algorithm
    #[arg(long, default_value = "skim-v2", value_name = "ALGORITHM")]
    pub algorithm: Option<Algorithm>,

    /// Comma-separated list of sort criteria to apply when the scores are tied
    ///
    /// Each criterion should appear only once in the list
    #[arg(
        short,
        long,
        value_enum,
        default_value = "score",
        value_name = "CRITERIA"
    )]
    pub tiebreak: Option<Tiebreak>,

    /// Do not sort the search result
    #[arg(long)]
    pub no_sort: bool,

    /// Enable exact-match
    #[arg(short, long)]
    pub exact: bool,

    /// Enable regex-mode
    #[arg(long)]
    pub regex: bool,

    /// Change color theme: [BASE_SCHEME][,COLOR:ANSI]
    ///
    /// Color configuration. The name of the base color scheme is followed by custom
    /// color mappings. Ansi color code of -1 denotes terminal default foreground /
    /// background color. You can also specify 24-bit color in #rrggbb format.
    ///
    /// Example:
    ///
    ///     --color=bg+:24
    ///
    ///     --color=light,fg:232,bg:255,bg+:116,info:27
    ///
    /// BASE SCHEME:
    ///     (default: dark on 256-color terminal, otherwise 16)
    ///
    ///     dark    Color scheme for dark 256-color terminal
    ///     light   Color scheme for light 256-color terminal
    ///     16      Color scheme for 16-color terminal
    ///     bw      No colors
    ///
    /// COLOR:
    ///
    ///     fg                Text
    ///     bg                Background
    ///     matched|hl        Text of highlighted substrings
    ///     matched_bg        Background of highlighted substrings
    ///     current|fg+       Text (current line)
    ///     current_bg|bg+    Background (current line)
    ///     current_match|hl+ Text of Highlighted substrings (current line)
    ///     current_match_bg  Background of highlighted substrings (current line)
    ///     query             Text of Query (the texts after the prompt)
    ///     query_bg          Background of Query
    ///     info              Info
    ///     border            Border of the preview window and horizontal separators
    ///     prompt            Prompt
    ///     pointer|cursor    Pointer to the current line (no effect now)
    ///     marker|selected   Multi-select marker (no effect now)
    ///     spinner           Streaming input indicator (no effect now)
    ///     header            Header (no effect now)
    #[arg(
        long,
        default_value = "default",
        value_name = "COLOR",
        verbatim_doc_comment
    )]
    pub color: Option<String>,

    /// Choose the layout
    #[arg(long, value_enum, default_value = "default", value_name = "LAYOUT")]
    pub layout: Option<Layout>,

    /// Asynonum for --layout=reverse
    #[arg(long)]
    pub reverse: bool,

    /// Display window below the cursor with the given height instead of using the full screen
    #[arg(long, default_value = "100%", value_name = "HEIGHT[%]")]
    pub height: Option<String>,

    /// Minimum height when `--height` is given in percent (default: 10).
    /// Ignored when `--height` is not specified.
    #[arg(long, default_value = "10", value_name = "HEIGHT")]
    pub min_height: Option<String>,

    /// Screen margin (TRBL / TB,RL / T,RL,B / T,R,B,L)
    /// Example: --margin 1,10%
    #[arg(long, default_value = "0", verbatim_doc_comment)]
    pub margin: Option<String>,

    /// Prompt string for query
    #[arg(short, long, default_value = "> ")]
    pub prompt: Option<String>,

    /// Display info next to query
    #[arg(long)]
    pub inline_info: bool,

    /// Disable preview window
    #[arg(long)]
    pub no_preview: bool,

    /// Preview window layout
    ///
    /// format: [up|down|left|right][:SIZE[%]][:hidden][:SCROLL[-OFFSET]]
    #[arg(long, default_value = "right:50%", value_name = "PREVIEW")]
    pub preview_window: Option<String>,

    /// Accent color used in preview window
    #[arg(long, value_enum, default_value = "magenta", value_name = "COLOR")]
    pub accent_color: Option<AccentColor>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Algorithm {
    /// Skim's legacy algorithm
    SkimV1,
    /// Skim's current algorithm
    SkimV2,
    /// Clangd algorithm
    Clangd,
}

impl Algorithm {
    pub fn as_str(&self) -> &str {
        match self {
            Algorithm::SkimV1 => "skim_v1",
            Algorithm::SkimV2 => "skim_v2",
            Algorithm::Clangd => "clangd",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Tiebreak {
    /// Score of fuzzy matching algorithm (default)
    Score,
    /// Prefers line that appeared earlier in the input stream
    Index,
    /// Prefers line with matched substring closer to the beginning
    Begin,
    /// Prefers line with matched substring closer to the end
    End,
}

impl Tiebreak {
    pub fn to_string(&self) -> String {
        match self {
            Tiebreak::Score => "score".to_string(),
            Tiebreak::Index => "index".to_string(),
            Tiebreak::Begin => "begin".to_string(),
            Tiebreak::End => "end".to_string(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Layout {
    Default,
    Reverse,
    ReverseList,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum AccentColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

pub fn build_options() -> SkimOptions<'static> {
    SkimOptionsBuilder::default()
        .multi(false)
        .preview(if OPTIONS.no_preview { None } else { Some("") })
        .algorithm(FuzzyAlgorithm::of(
            OPTIONS.algorithm.unwrap_or(Algorithm::SkimV2).as_str(),
        ))
        .tiebreak(Some(
            OPTIONS.tiebreak.unwrap_or(Tiebreak::Score).to_string(),
        ))
        .nosort(OPTIONS.no_sort)
        .exact(OPTIONS.exact)
        .regex(OPTIONS.regex)
        .color(OPTIONS.color.as_deref())
        .preview_window(OPTIONS.preview_window.as_deref())
        .layout(if OPTIONS.reverse {
            "reverse"
        } else {
            match OPTIONS.layout.unwrap_or(Layout::Default) {
                Layout::Default => "default",
                Layout::Reverse => "reverse",
                Layout::ReverseList => "reverse-list",
            }
        })
        .height(OPTIONS.height.as_deref())
        .min_height(OPTIONS.min_height.as_deref())
        .margin(OPTIONS.margin.as_deref())
        .prompt(OPTIONS.prompt.as_deref())
        .inline_info(OPTIONS.inline_info)
        .build()
        .expect("Failed to build skim options")
}
