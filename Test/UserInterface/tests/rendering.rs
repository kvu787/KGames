// Draws frames offscreen with the GPU, without a window, and checks their
// pixels: the theme's background, a card, an accent button's fill and label,
// a standard button's elevation border, that widgets draw a new state whole
// on the frame that makes it, that a press draws all it did on its own frame,
// that a selection that moves leaves its old item on that frame, that a
// release shows on its own frame, whether the PerfBar says that vertical sync
// is on, the PerfBar's colors in each theme, the page for an unsupported
// setup, and the pointer that a window draws while it has focus.

use user_interface::controls::MENU_BAR_HEIGHT;
use user_interface::graphics::{AdapterChoice, Gpu, Image, write_png};
use user_interface::tokens::PERF_BAR_HEIGHT;
use user_interface::{
    Button, Card, CheckBox, Color, ComboBox, Context, Cursor, Event, Headless, Interface, Key, ListView, MenuBar, MenuItem, MouseButton, Palette,
    Program, RadioGroup, Rect, Segmented, SelectorBar, Sense, Slider, Text, Theme, ThemeKind, ToggleSwitch, Tree, Ui, Widget, unsupported_setup,
};

struct Page;

impl Program for Page {
    type Message = ();

    fn title(&self) -> String {
        "Page".to_string()
    }

    fn update(&mut self, _message: (), _ui: &mut Context<()>) {}

    fn view(&self, ui: &mut Ui) -> Tree<()> {
        let mut tree = Tree::new();
        tree.card(ui, None, Rect::new(40.0, 40.0, 400.0, 200.0), Card::new("A card"), |_, card, area| {
            let [accent, standard] = area.content.top_strip(32.0).columns::<2>(8.0);
            card.add("Accent", accent, Button::new("Accent").accent());
            card.add("Standard", standard, Button::new("Standard"));
        });
        tree
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
        assert_eq!((image.width, image.height), (2560, 1440));
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
    color: Option<usize>,
}

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    On(bool),
    Checked(bool),
    Size(usize),
    Level(f64),
    Alignment(usize),
    Tab(usize),
    Pick(usize),
    Row(usize),
    Color(usize),
}

const SIZES: [&str; 3] = ["Small", "Medium", "Large"];
const COLORS: [Color; 4] = [Color::hex(0xD13438), Color::hex(0x107C10), Color::hex(0x0078D4), Color::hex(0x8764B8)];

fn menus() -> Vec<(&'static str, Vec<MenuItem<Msg>>)> {
    let recent = vec![MenuItem::new("First"), MenuItem::new("Second")];
    vec![("File", vec![MenuItem::new("Open"), MenuItem::submenu("Recent", recent)])]
}

impl Program for States {
    type Message = Msg;

    fn title(&self) -> String {
        "States".to_string()
    }

    fn update(&mut self, message: Msg, _ui: &mut Context<Msg>) {
        match message {
            Msg::On(on) => self.on = on,
            Msg::Checked(checked) => self.checked = checked,
            Msg::Size(size) => self.size = size,
            Msg::Level(level) => self.level = level,
            Msg::Alignment(alignment) => self.alignment = alignment,
            Msg::Tab(tab) => self.tab = tab,
            Msg::Pick(pick) => self.pick = pick,
            Msg::Row(row) => self.row = Some(row),
            Msg::Color(color) => self.color = Some(color),
        }
    }

    fn view(&self, ui: &mut Ui) -> Tree<Msg> {
        let mut tree = Tree::new();
        let top = ui.content_top();
        tree.add("Menus", Rect::new(40.0, top + 8.0, 200.0, MENU_BAR_HEIGHT), MenuBar::new(menus()));
        tree.card(ui, None, Rect::new(40.0, top + 300.0, 480.0, 800.0), Card::new("States"), |ui, card, area| {
            let mut area = area.content;
            card.add("Button", area.cut_top(32.0, 8.0), Button::new("Button"));
            card.add("Switch", area.cut_top(32.0, 8.0), ToggleSwitch::new(self.on, "Switch").on_change(Msg::On));
            card.add("Check", area.cut_top(32.0, 8.0), CheckBox::new(self.checked, "Check").on_change(Msg::Checked));
            card.add("Size", area.cut_top(RadioGroup::<Msg>::height(SIZES.len()), 8.0), RadioGroup::new(self.size, &SIZES).on_select(Msg::Size));
            card.add("Level", area.cut_top(32.0, 8.0), Slider::new(self.level, 0.0, 1.0).on_change(Msg::Level));
            card.add("Alignment", area.cut_top(32.0, 8.0), Segmented::new(self.alignment, &["Left", "Center", "Right"]).on_select(Msg::Alignment));
            card.add("Tabs", area.cut_top(48.0, 8.0), SelectorBar::new(self.tab, &["First", "Second", "Third"]).on_select(Msg::Tab));
            card.add("Pick", area.cut_top(32.0, 8.0), ComboBox::new(self.pick, &["One", "Two", "Three"]).on_select(Msg::Pick));
            // As tall as its rows, so that it has no scroll bar, whose thumb
            // waits for the pointer to rest on it before it widens.
            let rows = area.cut_top(120.0, 8.0);
            let list = ListView::new(3, 40.0).selection(self.row).on_select(Msg::Row);
            let shown = list.rows(ui, "Rows", rows);
            card.container(Some("Rows"), rows, list, |list| {
                for index in shown.indices() {
                    list.draw(shown.content(index), Text::label(format!("Row {index}")));
                }
            });
            card.add("Colors", area.cut_top(32.0, 8.0), Palette::new(self.color, &COLORS).on_pick(Msg::Color));
        });
        tree
    }
}

#[test]
fn widgets_draw_a_new_state_whole_on_the_frame_that_makes_it() {
    // Each step is input that changes how a widget looks, and after which
    // nothing changes. The frame that reads the input is drawn, and then
    // the frame a second later, which must be the same below the PerfBar,
    // whose numbers do change. A widget that eased, faded or slid to its
    // new state, even over WinUI's fastest 83 ms, would be less than half
    // way there in the first.
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
        ("a press of a toggle switch, which turns it on", |app| {
            app.press("Switch");
        }),
        ("its release", |app| {
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
        act(&mut app);
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
fn a_press_draws_all_it_did_on_its_own_frame() {
    // A press acts as the button goes down: the frame that reads it draws
    // the switch on, the box checked, the item selected or the menu open,
    // and is the same a second later, with the button still held.
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut app = Headless::new(States::default());
    app.frame();
    for name in ["Switch", "Check", "Size/Medium", "Alignment/Center", "Tabs/Second", "Rows/1", "Colors/1", "Menus/File"] {
        app.press(name);
        let now = app.render(&mut gpu).unwrap();
        app.wait(1.0);
        let later = app.render(&mut gpu).unwrap();
        let below = (PERF_BAR_HEIGHT as u32 * now.width * 4) as usize;
        if now.pixels[below..] != later.pixels[below..] {
            save(&now, "PressNow");
            save(&later, "PressLater");
            panic!("the press of {name} didn't draw all it did on its own frame");
        }
        app.release();
    }
    let program = &app.program;
    assert!(program.on && program.checked);
    assert_eq!((program.size, program.alignment, program.tab, program.row, program.color), (1, 1, 1, Some(1), Some(1)));
    assert_eq!(app.ui.popup_name(), Some("Menus/File/Menu"));
}

#[test]
fn a_selection_leaves_its_old_item_on_the_frame_that_moves_it() {
    // A press selects a list's row, a radio button, a segment, a tab or a
    // swatch. The frame that reads the input applies its message and then
    // describes every item from the model, so even when the selection moves
    // to an item after the old one, that very frame draws the old item
    // without it, the same as a second later.
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut app = Headless::new(States::default());
    app.frame();
    let moves = [("Rows/0", "Rows/2"), ("Size/Small", "Size/Large"), ("Alignment/Left", "Alignment/Right"), ("Tabs/First", "Tabs/Third"), ("Colors/0", "Colors/2")];
    for (old, new) in moves {
        app.click(old).press(new);
        let now = app.render(&mut gpu).unwrap();
        app.wait(1.0);
        let later = app.render(&mut gpu).unwrap();
        app.release();
        if let Some((x, y)) = first_difference(&now, &later, app.target(old)) {
            save(&now, "SelectionNow");
            save(&later, "SelectionLater");
            panic!("{old} still looked selected on the frame that selected {new}, at ({x}, {y})");
        }
    }
    let program = &app.program;
    assert_eq!((program.row, program.size, program.alignment, program.tab, program.color), (Some(2), 2, 2, 2, Some(2)));
}

#[test]
fn a_release_shows_on_the_frame_that_reads_it() {
    // A widget that the left button holds down looks pressed until the
    // frame that reads the release, which draws it as it looks from then
    // on, with what its press did, like a switch that's flipped or a menu
    // that's open, and not pressed for a frame more.
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut app = Headless::new(States::default());
    app.frame();
    for name in ["Button", "Switch", "Check", "Size/Medium", "Level", "Alignment/Center", "Tabs/Second", "Rows/1", "Colors/1", "Menus/File"] {
        app.press(name).queue(&[Event::ButtonReleased(MouseButton::Left)]).frame();
        let now = app.render(&mut gpu).unwrap();
        app.wait(1.0);
        let later = app.render(&mut gpu).unwrap();
        if let Some((x, y)) = first_difference(&now, &later, app.target(name)) {
            save(&now, "ReleaseNow");
            save(&later, "ReleaseLater");
            panic!("{name} didn't look let go on the frame that let go of it, at ({x}, {y})");
        }
    }
    let program = &app.program;
    assert!(program.on && program.checked);
    assert_eq!((program.size, program.alignment, program.tab, program.row, program.color), (1, 1, 1, Some(1), Some(1)));
    assert_eq!(app.ui.popup_name(), Some("Menus/File/Menu"));
}

// The first pixel in the area that differs between the two frames.
fn first_difference(now: &Image, later: &Image, area: Rect) -> Option<(u32, u32)> {
    let mut pixels = (area.y as u32..area.bottom() as u32).flat_map(|y| (area.x as u32..area.right() as u32).map(move |x| (x, y)));
    pixels.find(|&(x, y)| now.pixel(x, y) != later.pixel(x, y))
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
    type Message = ();

    fn title(&self) -> String {
        "Unsupported".to_string()
    }

    fn update(&mut self, _message: (), _ui: &mut Context<()>) {}

    fn view(&self, _ui: &mut Ui) -> Tree<()> {
        unsupported_setup(&[
            "The display is 1920 × 1080 pixels. It must be 2560 × 1440.".to_string(),
            "The window's client area is 1280 × 720 at (0, 0). It must be the whole display, 1920 × 1080 at (0, 0).".to_string(),
        ])
    }
}

const CURSORS: [Cursor; 10] = [
    Cursor::Arrow,
    Cursor::Text,
    Cursor::Hand,
    Cursor::Move,
    Cursor::ResizeHorizontal,
    Cursor::ResizeVertical,
    Cursor::ResizeDiagonal,
    Cursor::ResizeAntidiagonal,
    Cursor::Crosshair,
    Cursor::NotAllowed,
];

// A widget that asks for a cursor while the pointer is on it.
struct Shape(Cursor);

impl Widget<()> for Shape {
    fn kind(&self) -> &'static str {
        "Shape"
    }

    fn sense(&self) -> Option<Sense> {
        Some(Sense::VIEW)
    }

    fn cursor(&self, _ui: &Interface, hovered: bool, held: bool) -> Option<Cursor> {
        (hovered || held).then_some(self.0)
    }
}

// A row of widgets, one for each cursor.
struct Shapes;

impl Program for Shapes {
    type Message = ();

    fn title(&self) -> String {
        "Shapes".to_string()
    }

    fn update(&mut self, _message: (), _ui: &mut Context<()>) {}

    fn view(&self, _ui: &mut Ui) -> Tree<()> {
        let mut tree = Tree::new();
        for (index, cursor) in CURSORS.into_iter().enumerate() {
            tree.add(&format!("Shape{index}"), Rect::new(100.0 + index as f32 * 150.0, 200.0, 100.0, 100.0), Shape(cursor));
        }
        tree
    }
}

// The area of the image, each pixel made a square of `zoom` pixels.
fn crop(image: &Image, area: Rect, zoom: u32) -> Image {
    let (width, height) = (area.width as u32 * zoom, area.height as u32 * zoom);
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let at = (((area.y as u32 + y / zoom) * image.width + area.x as u32 + x / zoom) * 4) as usize;
            pixels.extend_from_slice(&image.pixels[at..at + 4]);
        }
    }
    Image { width, height, pixels }
}

// Images side by side, which are all as high.
fn side_by_side(images: &[Image]) -> Image {
    let height = images[0].height;
    let width = images.iter().map(|image| image.width).sum();
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for image in images {
            let row = (y * image.width * 4) as usize;
            pixels.extend_from_slice(&image.pixels[row..row + (image.width * 4) as usize]);
        }
    }
    Image { width, height, pixels }
}

// The window draws its own pointer while it keeps Windows' cursor, hidden,
// in the shape that the widget under it asks for, in the theme's accent with
// an edge in text_on_accent, unlike Windows' cursors, and only near its hot
// spot. Once it lets the cursor go, as focus goes, Windows' cursor shows,
// and the window draws none.
#[test]
fn the_pointer_is_drawn_in_the_shape_that_the_widget_under_it_asks_for_while_the_window_keeps_the_cursor() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut problems = Vec::new();
    for kind in [ThemeKind::Light, ThemeKind::Dark] {
        let mut app = Headless::with_theme(Shapes, kind);
        app.draw_pointer(true);
        let theme = Theme::new(kind);
        let (fill, edge) = (theme.accent.to_bytes(), theme.text_on_accent.to_bytes());
        let mut shapes = Vec::new();
        for (index, cursor) in CURSORS.into_iter().enumerate() {
            let name = format!("Shape{index}");
            app.draw_pointer(true).move_to(&name);
            assert_eq!(app.ui.cursor(), cursor);
            let with = app.render(&mut gpu).unwrap();
            app.draw_pointer(false);
            let without = app.render(&mut gpu).unwrap();
            let at = app.target(&name).center();
            let near = Rect::new(at.x - 13.0, at.y - 13.0, 31.0, 34.0);
            // Below the PerfBar, whose stats change from frame to frame.
            let mut changed = Vec::new();
            for y in PERF_BAR_HEIGHT as u32..with.height {
                for x in 0..with.width {
                    if with.pixel(x, y) != without.pixel(x, y) {
                        changed.push((x, y));
                    }
                }
            }
            let outside: Vec<_> = changed.iter().filter(|(x, y)| !near.contains(user_interface::Point::new(*x as f32, *y as f32))).collect();
            let filled = changed.iter().any(|&(x, y)| close(with.pixel(x, y), fill));
            let edged = changed.iter().any(|&(x, y)| close(with.pixel(x, y), edge));
            if changed.is_empty() || !outside.is_empty() || !filled || !edged {
                problems.push(format!(
                    "{} theme, {cursor:?}: {} pixels, {:?} away from the hot spot, filled {filled}, edged {edged}",
                    kind.name(),
                    changed.len(),
                    &outside[..outside.len().min(5)],
                ));
            }
            shapes.push(crop(&with, near, 6));
        }
        // Saved before the checks, to see what failed.
        save(&side_by_side(&shapes), &format!("Pointers{}", kind.name()));
    }
    assert!(problems.is_empty(), "{problems:#?}");
}

// The pointer is anywhere, in parts of a pixel, and it's drawn where it is,
// like a triangle whose corners are anywhere, not on the pixels' grid: half a
// pixel to the right, it's drawn neither where it was nor a whole pixel
// further.
#[test]
fn the_pointer_is_drawn_where_it_is_in_parts_of_a_pixel() {
    let mut gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    let mut app = Headless::new(Shapes);
    app.draw_pointer(true);
    let at = app.target("Shape0").center();
    app.move_point(at);
    let whole = app.render(&mut gpu).unwrap();
    app.move_point(user_interface::Point::new(at.x + 0.5, at.y));
    let half = app.render(&mut gpu).unwrap();
    save(&crop(&half, Rect::new(at.x - 4.0, at.y - 4.0, 24.0, 24.0), 8), "PointerHalfPixel");
    let area = Rect::new(at.x - 4.0, at.y - 4.0, 24.0, 24.0);
    let differs = |shift: u32| {
        (area.y as u32..area.bottom() as u32).any(|y| (area.x as u32..area.right() as u32).any(|x| half.pixel(x, y) != whole.pixel(x - shift, y)))
    };
    assert!(differs(0), "drawn where it was, on the pixel it's in");
    assert!(differs(1), "drawn a whole pixel further");
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
