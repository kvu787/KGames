// Fluent's system icons, as glyphs of Segoe Fluent Icons, the font that
// WinUI's SymbolThemeFontFamily names. Each icon is drawn as text, at a size
// in pixels, like XAML's FontIcon.

#pragma once

#include <array>
#include <cstdint>
#include <string_view>

// Every icon, with its glyph's code point in the font: `icon` is called with
// each. The enum, the list of them all, and their glyphs and names are made
// from this, so that an icon is added in one place. It's one to a line,
// which clang-format would undo.
// clang-format off
#define USER_INTERFACE_ICONS(icon)  \
    icon(Accept, 0xE8FB)            \
    icon(Add, 0xE710)               \
    icon(AlignCenter, 0xE8E3)       \
    icon(AlignLeft, 0xE8E4)         \
    icon(AlignRight, 0xE8E2)        \
    icon(Back, 0xE72B)              \
    icon(BackToWindow, 0xE73F)      \
    icon(Brush, 0xE771)             \
    icon(Calendar, 0xE787)          \
    icon(Camera, 0xE722)            \
    icon(Car, 0xE804)               \
    icon(CheckMark, 0xE73E)         \
    icon(ChevronDown, 0xE70D)       \
    icon(ChevronDownSmall, 0xE96E)  \
    icon(ChevronLeft, 0xE76B)       \
    icon(ChevronLeftSmall, 0xE96F)  \
    icon(ChevronRight, 0xE76C)      \
    icon(ChevronRightSmall, 0xE970) \
    icon(ChevronUp, 0xE70E)         \
    icon(ChevronUpSmall, 0xE96D)    \
    icon(Clear, 0xE894)             \
    icon(Clock, 0xE823)             \
    icon(Color, 0xE790)             \
    icon(Contrast, 0xE7A1)          \
    icon(Copy, 0xE8C8)              \
    icon(Crop, 0xE7A8)              \
    icon(Cut, 0xE8C6)               \
    icon(Delete, 0xE74D)            \
    icon(Dismiss, 0xE711)           \
    icon(Document, 0xE8A5)          \
    icon(Down, 0xE74B)              \
    icon(Download, 0xE896)          \
    icon(Edit, 0xE70F)              \
    icon(Erase, 0xE75C)             \
    icon(Error, 0xE783)             \
    icon(Export, 0xEDE1)            \
    icon(Eyedropper, 0xEF3C)        \
    icon(Fill, 0xED62)              \
    icon(Filter, 0xE71C)            \
    icon(Flag, 0xE7C1)              \
    icon(Folder, 0xE8B7)            \
    icon(Forward, 0xE72A)           \
    icon(FullScreen, 0xE740)        \
    icon(Gamepad, 0xE7FC)           \
    icon(Globe, 0xE774)             \
    icon(Help, 0xE897)              \
    icon(Hide, 0xED1A)              \
    icon(Home, 0xE80F)              \
    icon(Import, 0xE8B5)            \
    icon(Info, 0xE946)              \
    icon(Keyboard, 0xE765)          \
    icon(Library, 0xE8F1)           \
    icon(Light, 0xE793)             \
    icon(Link, 0xE71B)              \
    icon(List, 0xEA37)              \
    icon(Lock, 0xE72E)              \
    icon(Map, 0xE707)               \
    icon(Monitor, 0xE7F4)           \
    icon(Moon, 0xE708)              \
    icon(More, 0xE712)              \
    icon(Mouse, 0xE962)             \
    icon(Move, 0xE7C2)              \
    icon(Navigation, 0xE700)        \
    icon(NewFolder, 0xE8F4)         \
    icon(OpenFile, 0xE8E5)          \
    icon(Paste, 0xE77F)             \
    icon(Pause, 0xE769)             \
    icon(People, 0xE716)            \
    icon(Pin, 0xE718)               \
    icon(Play, 0xE768)              \
    icon(Redo, 0xE7A6)              \
    icon(Refresh, 0xE72C)           \
    icon(Remove, 0xE738)            \
    icon(Rotate, 0xE7AD)            \
    icon(Ruler, 0xED5E)             \
    icon(Save, 0xE74E)              \
    icon(SaveAs, 0xE792)            \
    icon(Search, 0xE721)            \
    icon(SelectAll, 0xE8B3)         \
    icon(Settings, 0xE713)          \
    icon(Star, 0xE734)              \
    icon(StarFilled, 0xE735)        \
    icon(Stop, 0xE71A)              \
    icon(Sync, 0xE895)              \
    icon(Tiles, 0xECA5)             \
    icon(Undo, 0xE7A7)              \
    icon(Unlock, 0xE785)            \
    icon(Up, 0xE74A)                \
    icon(Upload, 0xE898)            \
    icon(View, 0xE890)              \
    icon(Volume, 0xE767)            \
    icon(Warning, 0xE7BA)           \
    icon(ZoomIn, 0xE8A3)            \
    icon(ZoomOut, 0xE71F)           \
    /* The parts of InfoBar's and InfoBadge's status icons: a filled circle,     \
       and the marks that go on it. */                                           \
    icon(StatusCircle, 0xF136)      \
    icon(StatusCircleInner, 0xF137) \
    icon(StatusInfo, 0xF13F)        \
    icon(StatusSuccess, 0xF13E)     \
    icon(StatusWarning, 0xF13C)     \
    icon(StatusError, 0xF13D)
// clang-format on

namespace user_interface {

#define USER_INTERFACE_ICON_ENUMERATOR(name, code) name,
enum class Icon : std::uint8_t { USER_INTERFACE_ICONS(USER_INTERFACE_ICON_ENUMERATOR) };
#undef USER_INTERFACE_ICON_ENUMERATOR

#define USER_INTERFACE_ICON_ITEM(name, code) Icon::name,
inline constexpr auto AllIcons = std::to_array<Icon>({USER_INTERFACE_ICONS(USER_INTERFACE_ICON_ITEM)});
#undef USER_INTERFACE_ICON_ITEM

char32_t glyph(Icon icon);

// Like "ChevronDown".
std::string_view name(Icon icon);

} // namespace user_interface
