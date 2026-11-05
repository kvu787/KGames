// The showcase's state: what each of its widgets shows, which they change,
// the list's items, and a log of what changed. Panels.rs draws it.

use std::collections::VecDeque;

use json::Value;
use user_interface::{Frame, Program, Setup, theme};
use windows_api::*;
use windows_core::Result;

use crate::NAME;
use crate::panels;

// The log keeps this many lines.
const LOG_LINES: usize = 64;

// The buttons' styles, in their row's order.
pub const STYLES: [&str; 4] = ["Standard", "Accent", "Danger", "Subtle"];
pub const SIZES: [&str; 3] = ["Small", "Medium", "Large"];
pub const VIEWS: [&str; 3] = ["Day", "Week", "Month"];
pub const TABS: [&str; 3] = ["Overview", "Details", "Notes"];
pub const TAGS: [&str; 6] = ["Rust", "Win32", "Direct3D 12", "DirectWrite", "Low latency", "Fluent"];
// The colors that the color combo box offers, with what each is for in the
// theme.
pub const COLORS: [(&str, &str); 6] = [
    ("Accent", "what's chosen, and the main action"),
    ("Success", "what's on, or went well"),
    ("Caution", "what needs attention"),
    ("Critical", "what failed"),
    ("Orange", "the second of a chart's series"),
    ("Purple", "the third of a chart's series"),
];
pub const DENSITIES: [(&str, &str); 3] = [
    ("Comfortable", "rows 40 pixels tall"),
    ("Standard", "rows 36 pixels tall"),
    ("Compact", "rows 28 pixels tall"),
];
// The list's rows, and how tall they are, for each density.
pub const ROW_HEIGHTS: [f32; 3] = [40.0, 36.0, 28.0];

const ADJECTIVES: [&str; 10] = [
    "Amber", "Brisk", "Cobalt", "Dusty", "Eager", "Fuzzy", "Gentle", "Hollow", "Ivory", "Jolly",
];
const NOUNS: [&str; 12] = [
    "Anchor", "Beacon", "Comet", "Delta", "Ember", "Falcon", "Glacier", "Harbor", "Island", "Jasper", "Kettle",
    "Lantern",
];

pub struct Showcase {
    // How many times each style of button was pressed, in STYLES' order.
    pub presses: [u32; 4],
    // What the icon buttons add to and take from.
    pub count: i32,
    // Whether the controls under the first checkbox take input.
    pub enabled: bool,
    pub grid: bool,
    pub snap: bool,
    pub sound: bool,
    pub animate: bool,
    pub tooltips: bool,
    pub dark: bool,
    pub size: usize,
    pub view: usize,
    pub tab: usize,
    pub volume: f64,
    pub opacity: f64,
    pub angle: f64,
    // A slider that's always disabled.
    pub locked: f64,
    pub color: usize,
    pub density: usize,
    pub tags: [bool; 6],
    pub name: String,
    // What the greeting says, once the name is submitted.
    pub greeting: Option<String>,
    pub search: String,
    // Whether the caution info bar was closed.
    pub caution_closed: bool,
    // The list's selected item, by its index among all of them.
    pub selected: Option<usize>,
    // The item that was double-clicked last, which the list opens.
    pub opened: Option<usize>,
    pub items: Vec<String>,
    // What changed, newest first.
    pub log: VecDeque<String>,
    pub cache: panels::Cache,
}

impl Default for Showcase {
    fn default() -> Self {
        Self {
            presses: [0; 4],
            count: 3,
            enabled: true,
            grid: true,
            snap: false,
            sound: true,
            animate: true,
            tooltips: true,
            dark: false,
            size: 1,
            view: 1,
            tab: 0,
            volume: 40.0,
            opacity: 0.75,
            angle: 45.0,
            locked: 0.5,
            color: 0,
            density: 1,
            tags: [true, true, false, false, true, false],
            name: String::new(),
            greeting: None,
            search: String::new(),
            caution_closed: false,
            selected: None,
            opened: None,
            items: (0..ADJECTIVES.len() * NOUNS.len())
                .map(|index| {
                    let adjective = ADJECTIVES[index % ADJECTIVES.len()];
                    let noun = NOUNS[index / ADJECTIVES.len()];
                    format!("{:03}  {adjective} {noun}", index + 1)
                })
                .collect(),
            log: VecDeque::new(),
            cache: panels::Cache::default(),
        }
    }
}

impl Showcase {
    // Adds a line to the log. A line about the same thing as the newest
    // replaces it, so that dragging a slider makes one line, not one a
    // frame.
    pub fn record(&mut self, subject: &str, what: String) {
        let line = format!("{subject}: {what}");
        if self.log.front().is_some_and(|newest| newest.starts_with(&format!("{subject}: "))) {
            self.log[0] = line;
        } else {
            self.log.push_front(line);
            self.log.truncate(LOG_LINES);
        }
    }

    // The items that the search matches, by index, ignoring case.
    pub fn matches(&self) -> Vec<usize> {
        let search = self.search.trim().to_lowercase();
        (0..self.items.len())
            .filter(|&index| search.is_empty() || self.items[index].to_lowercase().contains(&search))
            .collect()
    }

    // Everything back to how it started, but the log, which says so, and
    // the theme.
    pub fn reset(&mut self) {
        let log = std::mem::take(&mut self.log);
        let dark = self.dark;
        *self = Showcase { log, dark, ..Showcase::default() };
        self.record("Everything", "reset".to_string());
    }
}

impl Program for Showcase {
    type Settings = ();
    const NAME: &'static str = NAME;

    fn new(_setup: Setup<'_, ()>) -> Result<Self> {
        Ok(Showcase::default())
    }

    fn frame(&mut self, frame: &mut Frame) -> Result<()> {
        // The program's own shortcuts: the keys that no widget took.
        if frame.shortcut(VK_F1) {
            self.tooltips = !self.tooltips;
            self.record("Tooltips", on_off(self.tooltips));
        }
        if frame.shortcut(VK_F2) {
            self.dark = !self.dark;
            self.record("Dark theme", on_off(self.dark));
        }
        if frame.control_shortcut('R' as i32) {
            self.reset();
        }
        frame.set_theme(if self.dark { &theme::DARK } else { &theme::LIGHT });
        panels::draw(self, frame);
        Ok(())
    }

    // What every widget shows, by the names of their cards' labels, for
    // scripts to save and check.
    fn state(&self) -> Value {
        let tags: Vec<&str> = TAGS
            .iter()
            .zip(self.tags)
            .filter(|(_, on)| *on)
            .map(|(tag, _)| *tag)
            .collect();
        let log: Vec<String> = self.log.iter().take(8).cloned().collect();
        let presses = STYLES
            .iter()
            .zip(self.presses)
            .map(|(style, count)| (style.to_string(), Value::from(count)))
            .collect();
        json::object([
            ("Presses", Value::Object(presses)),
            ("Count", (self.count as f64).into()),
            ("Enabled", self.enabled.into()),
            ("Grid", self.grid.into()),
            ("Snap", self.snap.into()),
            ("Sound", self.sound.into()),
            ("Animate", self.animate.into()),
            ("Tooltips", self.tooltips.into()),
            ("Dark", self.dark.into()),
            ("Size", SIZES[self.size].into()),
            ("View", VIEWS[self.view].into()),
            ("Tab", TABS[self.tab].into()),
            ("Volume", self.volume.into()),
            ("Opacity", self.opacity.into()),
            ("Angle", self.angle.into()),
            ("Color", COLORS[self.color].0.into()),
            ("Density", DENSITIES[self.density].0.into()),
            ("Tags", tags.into()),
            ("Name", self.name.clone().into()),
            ("Greeting", self.greeting.clone().into()),
            ("Search", self.search.clone().into()),
            ("CautionClosed", self.caution_closed.into()),
            ("Selected", self.selected.map(|index| self.items[index].clone()).into()),
            ("Opened", self.opened.map(|index| self.items[index].clone()).into()),
            ("Matches", self.matches().len().into()),
            ("Log", log.into()),
        ])
    }
}

pub fn on_off(on: bool) -> String {
    if on { "on" } else { "off" }.to_string()
}
