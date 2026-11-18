// Fluent's system icons, as glyphs of Segoe Fluent Icons, the font that
// WinUI's SymbolThemeFontFamily names. Each icon is drawn as text, at a size
// in pixels, like XAML's FontIcon.

macro_rules! icons {
    ($($name:ident = $code:expr,)*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum Icon {
            $($name,)*
        }

        impl Icon {
            pub const ALL: &'static [Icon] = &[$(Icon::$name,)*];

            pub fn glyph(self) -> char {
                let code: u32 = match self {
                    $(Icon::$name => $code,)*
                };
                char::from_u32(code).unwrap_or(' ')
            }

            // Like "ChevronDown".
            pub fn name(self) -> &'static str {
                match self {
                    $(Icon::$name => stringify!($name),)*
                }
            }
        }
    };
}

icons! {
    Accept = 0xE8FB,
    Add = 0xE710,
    AlignCenter = 0xE8E3,
    AlignLeft = 0xE8E4,
    AlignRight = 0xE8E2,
    Back = 0xE72B,
    BackToWindow = 0xE73F,
    Brush = 0xE771,
    Calendar = 0xE787,
    Camera = 0xE722,
    Car = 0xE804,
    CheckMark = 0xE73E,
    ChevronDown = 0xE70D,
    ChevronDownSmall = 0xE96E,
    ChevronLeft = 0xE76B,
    ChevronLeftSmall = 0xE96F,
    ChevronRight = 0xE76C,
    ChevronRightSmall = 0xE970,
    ChevronUp = 0xE70E,
    ChevronUpSmall = 0xE96D,
    Clear = 0xE894,
    Clock = 0xE823,
    Color = 0xE790,
    Contrast = 0xE7A1,
    Copy = 0xE8C8,
    Crop = 0xE7A8,
    Cut = 0xE8C6,
    Delete = 0xE74D,
    Dismiss = 0xE711,
    Document = 0xE8A5,
    Down = 0xE74B,
    Download = 0xE896,
    Edit = 0xE70F,
    Erase = 0xE75C,
    Error = 0xE783,
    Export = 0xEDE1,
    Eyedropper = 0xEF3C,
    Fill = 0xED62,
    Filter = 0xE71C,
    Flag = 0xE7C1,
    Folder = 0xE8B7,
    Forward = 0xE72A,
    FullScreen = 0xE740,
    Gamepad = 0xE7FC,
    Globe = 0xE774,
    Help = 0xE897,
    Hide = 0xED1A,
    Home = 0xE80F,
    Import = 0xE8B5,
    Info = 0xE946,
    Keyboard = 0xE765,
    Library = 0xE8F1,
    Light = 0xE793,
    Link = 0xE71B,
    List = 0xEA37,
    Lock = 0xE72E,
    Map = 0xE707,
    Monitor = 0xE7F4,
    Moon = 0xE708,
    More = 0xE712,
    Mouse = 0xE962,
    Move = 0xE7C2,
    Navigation = 0xE700,
    NewFolder = 0xE8F4,
    OpenFile = 0xE8E5,
    Paste = 0xE77F,
    Pause = 0xE769,
    People = 0xE716,
    Pin = 0xE718,
    Play = 0xE768,
    Redo = 0xE7A6,
    Refresh = 0xE72C,
    Remove = 0xE738,
    Rotate = 0xE7AD,
    Ruler = 0xED5E,
    Save = 0xE74E,
    SaveAs = 0xE792,
    Search = 0xE721,
    SelectAll = 0xE8B3,
    Settings = 0xE713,
    Star = 0xE734,
    StarFilled = 0xE735,
    Stop = 0xE71A,
    Sync = 0xE895,
    Tiles = 0xECA5,
    Undo = 0xE7A7,
    Unlock = 0xE785,
    Up = 0xE74A,
    Upload = 0xE898,
    View = 0xE890,
    Volume = 0xE767,
    Warning = 0xE7BA,
    ZoomIn = 0xE8A3,
    ZoomOut = 0xE71F,
    // The parts of InfoBar's and InfoBadge's status icons: a filled circle,
    // and the marks that go on it.
    StatusCircle = 0xF136,
    StatusCircleInner = 0xF137,
    StatusInfo = 0xF13F,
    StatusSuccess = 0xF13E,
    StatusWarning = 0xF13C,
    StatusError = 0xF13D,
}
