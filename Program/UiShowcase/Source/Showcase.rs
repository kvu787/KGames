// What the showcase's widgets show, what its shortcuts and menus do, its
// log, and its state for scripts: the model, the messages, and `update`,
// which applies them. `cards::view` describes the page from the model.

use std::collections::VecDeque;

use user_interface::json::Value;
use user_interface::{Color, Context, DialogResult, Key, Modifiers, Program, Startup, ThemeKind, Tree, Ui};

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

// What the input on the showcase's widgets means, and what its keys and
// menus do.
#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    // A press of one of BUTTONS, by index.
    Press(usize),
    Link,
    Pin(bool),
    Add,
    Remove,
    // A menu's command, by its label.
    Command(String),
    ToggleTheme,
    ToggleTooltips,
    Tooltips(bool),
    OptionsEnabled(bool),
    Option(usize, bool),
    Filling(bool),
    Size(usize),
    Alignment(usize),
    Tool(usize),
    Volume(f64),
    Opacity(f64),
    Angle(f64),
    Role(usize),
    Name(String),
    Greet,
    CloseGreeting,
    Search(String),
    Quantity(f64),
    Scale(f64),
    Color(Color),
    Swatch(usize),
    Tab(usize),
    // A row of the list, among the items the search matches.
    Select(usize),
    Activate(usize),
    CloseCaution,
    BringBackCaution,
    Expand(bool),
    Ask,
    Answer(DialogResult),
    Reset,
    // What the interface is on changed: the theme, if it did, and the names
    // of what has focus, what's pressed, and the open popup.
    Watch(Option<ThemeKind>, [Option<String>; 3]),
}

pub struct Showcase {
    pub presses: [u32; 4],
    pub count: i32,
    pub pinned: bool,
    pub link_clicks: u32,
    pub options_enabled: bool,
    pub options: [bool; 4],
    pub tooltips: bool,
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
    // What the interface was on last frame, and its theme, to log what
    // changes.
    seen: [Option<String>; 3],
    seen_theme: ThemeKind,
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
            seen_theme: ThemeKind::Light,
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
            seen_theme: self.seen_theme,
            fill_started: time,
            ..Showcase::default()
        };
        self.log("Reset every widget");
    }

    fn toggle_theme(&mut self, ui: &mut Context<Message>) {
        ui.toggle_theme();
        // Logged here, on the frame that switches it: the theme is the one
        // the frame draws in, as the toggle leaves it.
        let kind = ui.theme().kind;
        self.seen_theme = kind;
        self.log(format!("{} theme", kind.name()));
    }

    fn toggle_tooltips(&mut self) {
        self.tooltips = !self.tooltips;
        self.log(format!("Tooltips {}", if self.tooltips { "on" } else { "off" }));
    }

    // What a menu's command does, by its label.
    fn command(&mut self, ui: &mut Context<Message>, label: &str) {
        self.menu_choice = Some(label.to_string());
        match label {
            "Reset" => self.reset(ui.time()),
            "Dark theme" => self.toggle_theme(ui),
            "Tooltips" => self.toggle_tooltips(),
            "VSync" => ui.toggle_vertical_sync(),
            "Fullscreen" => ui.toggle_fullscreen(),
            "Ask a question" => self.dialog_open = true,
            "Pinned" => self.pinned = !self.pinned,
            _ => {}
        }
        self.log(format!("Chose {label}"));
    }
}

impl Program for Showcase {
    type Message = Message;

    fn title(&self) -> String {
        "UiShowcase".to_string()
    }

    // The shortcuts, and what the interface is on, which the log tells of
    // when it changes.
    fn input(&self, ui: &Ui) -> Vec<Message> {
        let mut messages = Vec::new();
        if ui.key_pressed(Key::F1) {
            messages.push(Message::ToggleTooltips);
        }
        if ui.shortcut(Modifiers::CONTROL, Key::R) {
            messages.push(Message::Reset);
        }
        let theme = (ui.theme().kind != self.seen_theme).then_some(ui.theme().kind);
        let names = [ui.focused_name(), ui.pressed_name(), ui.popup_name()];
        if theme.is_some() || names.iter().zip(&self.seen).any(|(now, seen)| *now != seen.as_deref()) {
            messages.push(Message::Watch(theme, names.map(|name| name.map(str::to_string))));
        }
        messages
    }

    fn update(&mut self, message: Message, ui: &mut Context<Message>) {
        match message {
            Message::Press(index) => {
                self.presses[index] += 1;
                if index < 3 {
                    self.log(format!("Pressed {}", BUTTONS[index]));
                }
            }
            Message::Link => {
                self.link_clicks += 1;
                self.log("Followed the link");
            }
            Message::Pin(pinned) => {
                self.pinned = pinned;
                self.log(if pinned { "Pinned" } else { "Unpinned" });
            }
            Message::Add => self.count += 1,
            Message::Remove => self.count -= 1,
            Message::Command(label) => self.command(ui, &label),
            Message::ToggleTheme => self.toggle_theme(ui),
            Message::ToggleTooltips => self.toggle_tooltips(),
            Message::Tooltips(on) => self.tooltips = on,
            Message::OptionsEnabled(enabled) => {
                self.options_enabled = enabled;
                self.log(format!("Options {}", if enabled { "enabled" } else { "disabled" }));
            }
            Message::Option(index, on) => self.options[index] = on,
            Message::Filling(filling) => {
                self.filling = filling;
                self.fill_started = ui.time();
            }
            Message::Size(size) => self.size = size,
            Message::Alignment(alignment) => self.alignment = alignment,
            Message::Tool(tool) => {
                self.tool = tool;
                self.log(format!("Tool: {}", TOOLS[tool]));
            }
            Message::Volume(volume) => self.volume = volume as f32,
            Message::Opacity(opacity) => self.opacity = opacity,
            Message::Angle(angle) => self.angle = angle as f32,
            Message::Role(role) => {
                self.role = role;
                self.log(format!("Role: {}", cards::ROLES[role]));
            }
            Message::Name(name) => self.name = name,
            Message::Greet => {
                let name = self.name.trim();
                self.greeting = Some(if name.is_empty() { "Hello, whoever you are!".to_string() } else { format!("Hello, {name}!") });
            }
            Message::CloseGreeting => self.greeting = None,
            Message::Search(search) => self.search = search,
            Message::Quantity(quantity) => self.quantity = quantity as i32,
            Message::Scale(scale) => self.scale = scale,
            Message::Color(color) => {
                self.color = color;
                self.log(format!("Color {}", color.to_hex()));
            }
            Message::Swatch(index) => self.color = Color::hex(cards::PALETTE[index]),
            Message::Tab(tab) => self.tab = tab,
            Message::Select(row) => self.selected = self.matches().get(row).copied(),
            Message::Activate(row) => {
                if let Some(item) = self.matches().get(row).copied() {
                    self.opened = Some(item);
                    self.log(format!("Opened {}", self.items[item]));
                }
            }
            Message::CloseCaution => {
                self.caution_shown = false;
                self.log("Closed the caution");
            }
            Message::BringBackCaution => self.caution_shown = true,
            Message::Expand(expanded) => self.expanded = expanded,
            Message::Ask => self.dialog_open = true,
            Message::Answer(result) => {
                let answer = match result {
                    DialogResult::Primary => "Save",
                    DialogResult::Secondary => "Don't save",
                    DialogResult::Close => "Cancel",
                };
                self.dialog_result = Some(answer.to_string());
                self.dialog_open = false;
                self.log(format!("Answered {answer}"));
            }
            Message::Reset => self.reset(ui.time()),
            Message::Watch(theme, names) => {
                if let Some(theme) = theme
                    && theme != self.seen_theme
                {
                    self.seen_theme = theme;
                    self.log(format!("{} theme", theme.name()));
                }
                for (index, verb) in ["Focus on", "Press on", "Opened"].iter().enumerate() {
                    if names[index] != self.seen[index] {
                        if let Some(now) = &names[index] {
                            self.log(format!("{verb} {now}"));
                        }
                        self.seen[index] = names[index].clone();
                    }
                }
            }
        }
    }

    fn view(&self, ui: &mut Ui) -> Tree<Message> {
        cards::view(self, ui)
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
