// Draws frames offscreen with the GPU, without a window, and checks their
// pixels: the theme's background, a card, an accent button's fill and label,
// a standard button's elevation border, that widgets draw a new state whole
// on the frame that makes it, whether the PerfBar says that vertical sync is
// on, the PerfBar's colors in each theme, and the page for an unsupported
// setup.

use user_interface::controls::MENU_BAR_HEIGHT;
use user_interface::graphics::{AdapterChoice, Gpu, Image, write_png};
use user_interface::tokens::PERF_BAR_HEIGHT;
use user_interface::{
    Button, Card, CheckBox, ComboBox, Headless, Key, ListView, MenuBar, MenuItem, Program, RadioGroup, Rect, Segmented, SelectorBar, Slider,
    Theme, ThemeKind, ToggleSwitch, Ui,
};

struct Page;

impl Program for Page {
    fn title(&self) -> String {
        "Page".to_string()
    }

    fn update(&mut self, ui: &mut Ui) {
        let card = ui.card(Rect::new(40.0, 40.0, 400.0, 200.0), Card::new("A card"));
        let [accent, standard] = card.content.top_strip(32.0).columns::<2>(8.0);
        ui.add("Accent", accent, Button::new("Accent").accent());
        ui.add("Standard", standard, Button::new("Standard"));
    }
}

fn close(a: [u8; 4], b: [u8; 4]) -> bool {
    a.iter().zip(b).take(3).all(|(a, b)| (*a as i32 - b as i32).abs() <= 2)
}

fn save(image: &Image, name: &str) {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/TestOutput");
    std::fs::create_dir_all(&folder).unwrap();
    write_png(&folder.join(format!("{name}.png")), image).unwrap();
}

#[test]
fn a_frame_draws_the_background_the_widgets_and_their_text() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    for kind in [ThemeKind::Light, ThemeKind::Dark] {
        let mut app = Headless::with_theme(Page, kind);
        let image = app.render(&mut gpu).unwrap();
        let theme = Theme::new(kind);
        save(&image, &format!("Page{}", kind.name()));
        assert_eq!((image.width, image.height), (2560, 1369));
        assert!(close(image.pixel(1000, 1000), theme.background.to_bytes()), "{:?}", image.pixel(1000, 1000));
        // The card is its fill over the background.
        assert!(close(image.pixel(400, 220), theme.card_opaque().to_bytes()), "{:?}", image.pixel(400, 220));
        // The accent button's fill, between its label and its edge.
        let button = app.target("Accent");
        let fill = image.pixel(button.x as u32 + 6, button.center().y as u32);
        assert!(close(fill, theme.accent.over(theme.card_opaque()).to_bytes()), "{fill:?}");
        // Its label is in text_on_accent: some pixel in its middle is.
        let mut found = false;
        for x in button.x as u32 + 40..button.right() as u32 - 40 {
            for y in button.y as u32 + 8..button.bottom() as u32 - 8 {
                found |= close(image.pixel(x, y), theme.text_on_accent.to_bytes());
            }
        }
        assert!(found, "the button's label didn't draw");
        // A standard button's elevation border is darker on one edge than
        // the other: the bottom in the light theme, the top in the dark one.
        let standard = app.target("Standard");
        let x = standard.center().x as u32 + 40;
        let top = image.pixel(x, standard.y as u32);
        let bottom = image.pixel(x, standard.bottom() as u32 - 1);
        let (stronger, weaker) = match kind {
            ThemeKind::Light => (bottom, top),
            ThemeKind::Dark => (top, bottom),
        };
        let brightness = |pixel: [u8; 4]| pixel[0] as i32 + pixel[1] as i32 + pixel[2] as i32;
        match kind {
            ThemeKind::Light => assert!(brightness(stronger) < brightness(weaker), "{top:?} {bottom:?}"),
            ThemeKind::Dark => assert!(brightness(stronger) > brightness(weaker), "{top:?} {bottom:?}"),
        }
    }
}

// A widget of each kind whose look follows the input, and a menu bar with a
// submenu.
#[derive(Default)]
struct States {
    on: bool,
    checked: bool,
    size: usize,
    level: f64,
    alignment: usize,
    tab: usize,
    pick: usize,
    row: Option<usize>,
}

const SIZES: [&str; 3] = ["Small", "Medium", "Large"];
const FILE: &[MenuItem] = &[MenuItem::new("Open"), MenuItem::submenu("Recent", RECENT)];
const RECENT: &[MenuItem] = &[MenuItem::new("First"), MenuItem::new("Second")];
const MENUS: &[(&str, &[MenuItem])] = &[("File", FILE)];

impl Program for States {
    fn title(&self) -> String {
        "States".to_string()
    }

    fn update(&mut self, ui: &mut Ui) {
        let top = ui.content_top();
        ui.add("Menus", Rect::new(40.0, top + 8.0, 200.0, MENU_BAR_HEIGHT), MenuBar::new(MENUS));
        let card = ui.card(Rect::new(40.0, top + 300.0, 480.0, 800.0), Card::new("States"));
        let mut area = card.content;
        ui.add("Button", area.cut_top(32.0, 8.0), Button::new("Button"));
        ui.add("Switch", area.cut_top(32.0, 8.0), ToggleSwitch::new(&mut self.on, "Switch"));
        ui.add("Check", area.cut_top(32.0, 8.0), CheckBox::new(&mut self.checked, "Check"));
        ui.add("Size", area.cut_top(RadioGroup::height(SIZES.len()), 8.0), RadioGroup::new(&mut self.size, &SIZES));
        ui.add("Level", area.cut_top(32.0, 8.0), Slider::new(&mut self.level, 0.0, 1.0));
        ui.add("Alignment", area.cut_top(32.0, 8.0), Segmented::new(&mut self.alignment, &["Left", "Center", "Right"]));
        ui.add("Tabs", area.cut_top(48.0, 8.0), SelectorBar::new(&mut self.tab, &["First", "Second", "Third"]));
        ui.add("Pick", area.cut_top(32.0, 8.0), ComboBox::new(&mut self.pick, &["One", "Two", "Three"]));
        // As tall as its rows, so that it has no scroll bar, whose thumb
        // waits for the pointer to rest on it before it widens.
        let rows = area.cut_top(120.0, 8.0);
        ui.add("Rows", rows, ListView::new(3, 40.0, |ui, index, row, _| {
            ui.label(row, &format!("Row {index}"));
        }).selection(&mut self.row));
    }
}

#[test]
fn widgets_draw_a_new_state_whole_on_the_frame_that_makes_it() {
    // Each step is input that changes how a widget looks, and after which
    // nothing changes. The second frame after the input's is drawn, and
    // then the frame a second later, which must be the same below the
    // PerfBar, whose numbers do change. A widget that eased, faded or slid
    // to its new state, even over WinUI's fastest 83 ms, would be less than
    // half way there in the first. It's the second frame after because the
    // pointer finds its widget by the last frame's targets: what a popup
    // that closes uncovers is under the pointer a frame later.
    type Step = fn(&mut Headless<States>);
    let steps: [(&str, Step); 19] = [
        ("the pointer onto a button", |app| {
            app.move_to("Button");
        }),
        ("a press of the button", |app| {
            app.press("Button");
        }),
        ("its release", |app| {
            app.release();
        }),
        ("a press of a toggle switch", |app| {
            app.press("Switch");
        }),
        ("its release, which turns it on", |app| {
            app.release();
        }),
        ("a click of a checkbox", |app| {
            app.click("Check");
        }),
        ("a click of a radio button", |app| {
            app.click("Size/Large");
        }),
        ("a press of a slider", |app| {
            app.press("Level");
        }),
        ("its release", |app| {
            app.release();
        }),
        ("a click of a segment", |app| {
            app.click("Alignment/Right");
        }),
        ("a click of a tab", |app| {
            app.click("Tabs/Third");
        }),
        ("a click of a list's row", |app| {
            app.click("Rows/2");
        }),
        ("a combo box's list opening", |app| {
            app.click("Pick");
        }),
        ("the pointer onto its item", |app| {
            app.move_to("Pick/1");
        }),
        ("choosing the combo box's item", |app| {
            app.click("Pick/1");
        }),
        ("a menu opening", |app| {
            app.click("Menus/File");
        }),
        ("its submenu opening", |app| {
            app.move_to("Menus/File/Menu/Recent");
        }),
        ("the pointer onto the submenu's item", |app| {
            app.move_to("Menus/File/Menu/Recent/Menu/First");
        }),
        ("choosing the submenu's item", |app| {
            app.click("Menus/File/Menu/Recent/Menu/First");
        }),
    ];
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut app = Headless::new(States::default());
    app.frame();
    for (index, (step, act)) in steps.into_iter().enumerate() {
        // The input's frame and the one after it, and one more.
        act(&mut app);
        app.frame();
        let now = app.render(&mut gpu).unwrap();
        app.wait(1.0);
        let later = app.render(&mut gpu).unwrap();
        let below = (PERF_BAR_HEIGHT as u32 * now.width * 4) as usize;
        if now.pixels[below..] == later.pixels[below..] {
            continue;
        }
        save(&now, &format!("States{index}Now"));
        save(&later, &format!("States{index}Later"));
        let (x, y) = (PERF_BAR_HEIGHT as u32..now.height)
            .flat_map(|y| (0..now.width).map(move |x| (x, y)))
            .find(|&(x, y)| now.pixel(x, y) != later.pixel(x, y))
            .unwrap();
        panic!("after {step}, ({x}, {y}) was {:?} and a second later {:?}", now.pixel(x, y), later.pixel(x, y));
    }
    // The steps did what they say.
    let program = &app.program;
    assert!(program.on && program.checked);
    assert_eq!((program.size, program.alignment, program.tab, program.pick, program.row), (2, 2, 2, 1, Some(2)));
}

#[test]
fn the_perf_bar_says_whether_vertical_sync_is_on() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut app = Headless::new(Page);
    // A second of frames, so that the PerfBar has a frame rate to show.
    app.wait(1.0);
    let on = app.render(&mut gpu).unwrap();
    app.key(Key::F10);
    let off = app.render(&mut gpu).unwrap();
    save(&on, "VerticalSyncOn");
    save(&off, "VerticalSyncOff");
    // "VSync = on" turns to "VSync = off" right after the other stats, which
    // start at the bar's left, so in its left half, and nothing else along
    // the bar changes: only "n " to "ff", two characters of Cascadia Mono, a
    // few pixels over 8 each.
    let changed: Vec<u32> = (0..off.width).filter(|&x| (0..PERF_BAR_HEIGHT as u32).any(|y| off.pixel(x, y) != on.pixel(x, y))).collect();
    let (Some(&first), Some(&last)) = (changed.first(), changed.last()) else {
        panic!("the PerfBar didn't change");
    };
    assert!(last - first < 30, "{first}..={last}");
    assert!(last < off.width / 2, "{first}..={last}");
}

#[test]
fn the_perf_bar_is_black_on_white_in_the_light_theme_and_white_on_black_in_the_dark() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    for kind in [ThemeKind::Light, ThemeKind::Dark] {
        let mut app = Headless::with_theme(Page, kind);
        let image = app.render(&mut gpu).unwrap();
        save(&image, &format!("PerfBar{}", kind.name()));
        let (background, text) = match kind {
            ThemeKind::Light => ([255, 255, 255], [0, 0, 0]),
            ThemeKind::Dark => ([0, 0, 0], [255, 255, 255]),
        };
        let rgb = |x: u32, y: u32| {
            let [r, g, b, _] = image.pixel(x, y);
            [r, g, b]
        };
        // Past its text, every row above its divider is its background.
        for y in 0..PERF_BAR_HEIGHT as u32 - 1 {
            assert_eq!(rgb(image.width - 8, y), background, "{} theme, row {y}", kind.name());
        }
        // Its text's strokes reach the text's color.
        let found = (16..400).any(|x| (0..PERF_BAR_HEIGHT as u32).any(|y| rgb(x, y) == text));
        assert!(found, "{} theme: no pixel of the text is {text:?}", kind.name());
    }
}

struct Unsupported;

impl Program for Unsupported {
    fn title(&self) -> String {
        "Unsupported".to_string()
    }

    fn update(&mut self, ui: &mut Ui) {
        ui.unsupported_setup(&[
            "The display is 1920 × 1080 pixels. It must be 2560 × 1440.".to_string(),
            "The window must be maximized, or borderless fullscreen with F11.".to_string(),
        ]);
    }
}

#[test]
fn an_unsupported_setup_says_what_is_wrong() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut app = Headless::with_size(Unsupported, ThemeKind::Light, user_interface::Size::new(1280.0, 720.0));
    let image = app.render(&mut gpu).unwrap();
    save(&image, "UnsupportedSetup");
    // The window's corner is the background, and the box's middle has the
    // critical background in it.
    let theme = Theme::light();
    assert!(close(image.pixel(5, 5), theme.background.to_bytes()));
    let critical = theme.critical_background.to_bytes();
    let found = (0..1280).step_by(8).any(|x| close(image.pixel(x, 360), critical));
    assert!(found);
}
