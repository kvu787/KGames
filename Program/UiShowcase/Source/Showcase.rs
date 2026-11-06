// What the showcase's widgets show, what its shortcuts and menus do, its
// log, and its state for scripts.

use std::collections::VecDeque;

use user_interface::json::Value;
use user_interface::{Color, Key, Modifiers, Program, Startup, ThemeKind, Ui};

use crate::cards;

pub const SIZES: [&str; 3] = ["Small", "Medium", "Large"];
pub const ALIGNMENTS: [&str; 3] = ["Left", "Center", "Right"];
pub const TOOLS: [&str; 4] = ["Brush", "Fill", "Erase", "Pick"];
pub const TABS: [&str; 4] = ["Type ramp", "Icons", "Colors", "Shapes"];
pub const OPTIONS: [&str; 4] = ["Snap to grid", "Show rulers", "Autosave", "Spell check"];
pub const BUTTONS: [&str; 4] = ["Standard", "Accent", "Subtle", "Save"];
const LOG_LENGTH: usize = 200;

// The list's 120 items.
pub fn items() -> Vec<String> {
    const WORDS: [&str; 12] = ["Aurora", "Basalt", "Cobalt", "Dune", "Ember", "Fjord", "Glacier", "Harbor", "Iris", "Juniper", "Kelp", "Lagoon"];
    (0..120).map(|index| format!("{} {}", WORDS[index % WORDS.len()], index / WORDS.len() + 1)).collect()
}

pub struct Showcase {
    pub presses: [u32; 4],
    pub count: i32,
    pub pinned: bool,
    pub link_clicks: u32,
    pub options_enabled: bool,
    pub options: [bool; 4],
    pub tooltips: bool,
    pub dark: bool,
    pub filling: bool,
    pub fill_started: f64,
    pub size: usize,
    pub alignment: usize,
    pub tool: usize,
    pub volume: f32,
    pub opacity: f64,
    pub angle: f32,
    pub role: usize,
    pub name: String,
    pub search: String,
    pub greeting: Option<String>,
    pub quantity: i32,
    pub scale: f64,
    pub color: Color,
    pub tab: usize,
    pub items: Vec<String>,
    pub selected: Option<usize>,
    pub opened: Option<usize>,
    pub caution_shown: bool,
    pub expanded: bool,
    pub dialog_open: bool,
    pub dialog_result: Option<String>,
    pub menu_choice: Option<String>,
    pub log: VecDeque<String>,
    // What the interface was on last frame, to log what changes.
    seen: [Option<String>; 3],
}

impl Default for Showcase {
    fn default() -> Self {
        Self {
            presses: [0; 4],
            count: 0,
            pinned: false,
            link_clicks: 0,
            options_enabled: true,
            options: [true, false, true, false],
            tooltips: true,
            dark: false,
            filling: true,
            fill_started: 0.0,
            size: 1,
            alignment: 1,
            tool: 0,
            volume: 50.0,
            opacity: 0.75,
            angle: 180.0,
            role: 0,
            name: String::new(),
            search: String::new(),
            greeting: None,
            quantity: 3,
            scale: 1.0,
            color: Color::hex(0x1E90FF),
            tab: 0,
            items: items(),
            selected: None,
            opened: None,
            caution_shown: true,
            expanded: true,
            dialog_open: false,
            dialog_result: None,
            menu_choice: None,
            log: VecDeque::new(),
            seen: [None, None, None],
        }
    }
}

impl Showcase {
    pub fn new(_startup: &mut Startup) -> Self {
        Self::default()
    }

    pub fn log(&mut self, line: impl Into<String>) {
        self.log.push_front(line.into());
        self.log.truncate(LOG_LENGTH);
    }

    // The items that the search matches, by their index in the list.
    pub fn matches(&self) -> Vec<usize> {
        let search = self.search.trim().to_lowercase();
        (0..self.items.len()).filter(|index| self.items[*index].to_lowercase().contains(&search)).collect()
    }

    pub fn reset(&mut self, time: f64) {
        let log = std::mem::take(&mut self.log);
        let seen = std::mem::take(&mut self.seen);
        *self = Showcase {
            log,
            seen,
            fill_started: time,
            ..Showcase::default()
        };
        self.log("Reset every widget");
    }

    // What a menu's command does, by its label.
    pub fn command(&mut self, ui: &mut Ui, label: &str) {
        self.menu_choice = Some(label.to_string());
        match label {
            "Reset" => self.reset(ui.time()),
            "Dark theme" => self.dark = !self.dark,
            "Tooltips" => self.tooltips = !self.tooltips,
            "Fullscreen" => ui.toggle_fullscreen(),
            "Ask a question" => self.dialog_open = true,
            _ => {}
        }
        self.log(format!("Chose {label}"));
    }

    fn shortcuts(&mut self, ui: &mut Ui) {
        if ui.key_pressed(Key::F1) {
            self.tooltips = !self.tooltips;
            self.log(format!("Tooltips {}", if self.tooltips { "on" } else { "off" }));
        }
        if ui.key_pressed(Key::F2) {
            self.dark = !self.dark;
            self.log(format!("{} theme", if self.dark { "Dark" } else { "Light" }));
        }
        if ui.shortcut(Modifiers::CONTROL, Key::R) {
            self.reset(ui.time());
        }
        self.apply(ui);
    }

    // The theme and tooltips that the switches, menus and shortcuts chose.
    fn apply(&self, ui: &mut Ui) {
        ui.set_tooltips_enabled(self.tooltips);
        ui.set_theme(if self.dark { ThemeKind::Dark } else { ThemeKind::Light });
    }

    // Logs when what the interface is on changes.
    fn watch(&mut self, ui: &Ui) {
        let names = [ui.focused_name(), ui.pressed_name(), ui.popup_name()];
        for (index, verb) in ["Focus on", "Press on", "Opened"].iter().enumerate() {
            if names[index] != self.seen[index].as_deref() {
                if let Some(now) = names[index] {
                    self.log(format!("{verb} {now}"));
                }
                self.seen[index] = names[index].map(str::to_string);
            }
        }
    }
}

impl Program for Showcase {
    fn title(&self) -> String {
        "UiShowcase".to_string()
    }

    fn update(&mut self, ui: &mut Ui) {
        self.shortcuts(ui);
        cards::draw(self, ui);
        // What the cards' switches changed shows from the next frame on.
        self.apply(ui);
        self.watch(ui);
    }

    fn state(&self) -> Value {
        let name_of = |names: &[&str], index: usize| Value::from(names[index]);
        let optional = |index: Option<usize>| index.map_or(Value::Null, Value::from);
        let text = |text: &Option<String>| text.clone().map_or(Value::Null, Value::from);
        Value::Object(vec![
            (
                "Presses".to_string(),
                Value::Object(BUTTONS.iter().zip(self.presses).map(|(name, count)| (name.to_string(), Value::from(count))).collect()),
            ),
            ("Count".to_string(), Value::from(self.count as i64)),
            ("Pinned".to_string(), Value::from(self.pinned)),
            ("LinkClicks".to_string(), Value::from(self.link_clicks)),
            ("OptionsEnabled".to_string(), Value::from(self.options_enabled)),
            ("Options".to_string(), Value::from(self.options.to_vec())),
            ("Tooltips".to_string(), Value::from(self.tooltips)),
            ("Dark".to_string(), Value::from(self.dark)),
            ("Filling".to_string(), Value::from(self.filling)),
            ("Size".to_string(), name_of(&SIZES, self.size)),
            ("Alignment".to_string(), name_of(&ALIGNMENTS, self.alignment)),
            ("Tool".to_string(), name_of(&TOOLS, self.tool)),
            ("Volume".to_string(), Value::from(self.volume as f64)),
            ("Opacity".to_string(), Value::from((self.opacity * 1000.0).round() / 1000.0)),
            ("Angle".to_string(), Value::from((self.angle as f64 * 10.0).round() / 10.0)),
            ("Role".to_string(), Value::from(cards::ROLES[self.role])),
            ("Name".to_string(), Value::from(self.name.as_str())),
            ("Search".to_string(), Value::from(self.search.as_str())),
            ("Greeting".to_string(), text(&self.greeting)),
            ("Quantity".to_string(), Value::from(self.quantity as i64)),
            ("Scale".to_string(), Value::from((self.scale * 100.0).round() / 100.0)),
            ("Color".to_string(), Value::from(self.color.to_hex())),
            ("Tab".to_string(), name_of(&TABS, self.tab)),
            ("Selected".to_string(), optional(self.selected)),
            ("Opened".to_string(), optional(self.opened)),
            ("Matches".to_string(), Value::from(self.matches().len())),
            ("CautionShown".to_string(), Value::from(self.caution_shown)),
            ("Expanded".to_string(), Value::from(self.expanded)),
            ("DialogOpen".to_string(), Value::from(self.dialog_open)),
            ("DialogResult".to_string(), text(&self.dialog_result)),
            ("MenuChoice".to_string(), text(&self.menu_choice)),
            ("Log".to_string(), Value::from(self.log.iter().take(10).cloned().collect::<Vec<String>>())),
        ])
    }
}
