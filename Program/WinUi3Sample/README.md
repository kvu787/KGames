# WinUi3Sample

A sample of a real WinUI 3 app, to show what one looks like and how one is made, beside the KGames
programs, which look like WinUI 3 but are built on
[Library/UserInterface](../../Library/UserInterface/README.md). It's a small notes app, in C# and
XAML, on the Windows App SDK 2.6.5 and .NET 10, with the parts that most WinUI 3 apps have: Mica,
a title bar of its own, a menu bar, a navigation pane, pages, dialogs, and a status bar.

It isn't a KGames GUI program, so it doesn't follow their rules:

- It's built with WinUI 3, not Library/UserInterface, so it has no PerfBar, and reads input through
  XAML, not GameInput. Like any app, it takes input only while it has focus.
- WinUI 3 draws through its own compositor, which the desktop compositor composes, so there's no
  vertical sync for `F10` to switch.
- It lays itself out for any window size and DPI, as WinUI 3 apps do.

It does keep what the programs share: it opens maximized, in the light theme, `F9` switches between
the light and dark themes, `F11` switches full screen, and it takes `--display` and
`--no-activate`.

## What it shows

| Part       | What it is                                                                        |
| ---------- | --------------------------------------------------------------------------------- |
| Title bar  | WinUI 3's `TitleBar` control, over Mica: a back button, a pane toggle, the icon,  |
|            | the title and a subtitle, a search box of the notes, and a `PersonPicture`.       |
|            | Windows still draws the minimize, maximize and close buttons, in the app's theme  |
| Menu bar   | A `MenuBar` of File, Edit, View and Help, with icons, keyboard accelerators,      |
|            | separators, submenus, toggles, radio items, a disabled item, and access keys      |
| Navigation | A `NavigationView` with Home, Notes, with an `InfoBadge`, Controls and Settings,  |
|            | and a `Frame` that switches pages with a transition                               |
| Home       | A banner, cards of quick actions, an `InfoBar`, the pinned notes in a `GridView`, |
|            | and the recently edited ones in a `ListView`                                      |
| Notes      | A `CommandBar`, the notes in a `ListView`, and an editor of the selected one,     |
|            | bound two ways with `x:Bind`, its text boxes without borders by lightweight       |
|            | styling                                                                           |
| Controls   | Buttons, toggle, drop-down, split and repeat buttons, checkboxes, radio buttons,  |
|            | toggle switches, a rating, text, password, number and auto-suggest boxes, combo   |
|            | boxes, sliders, progress bars and rings, info bars, date and time pickers, a      |
|            | content dialog, a flyout, a teaching tip, a context menu, an expander, a          |
|            | selector bar, a breadcrumb bar, a tree view, a tab view and a color picker        |
| Settings   | Cards like Windows 11's settings, with combo boxes for the theme, the backdrop    |
|            | (Mica, Mica Alt, acrylic or none) and the navigation's style, toggle switches,    |
|            | a slider, and an expander about the app                                           |
| Status bar | What happened last, the note's words, the zoom, and the theme                     |
| Dialogs    | `ContentDialog`s to delete a note, of the keyboard shortcuts, and about the app,  |
|            | and the Windows App SDK's file pickers to open and save notes as text files       |

Everything works, but nothing is saved between runs, except the notes that File > Save writes.

## Input

| Input              | Action                                                      |
| ------------------ | ----------------------------------------------------------- |
| `Ctrl+N`           | Starts a note                                               |
| `Ctrl+O`           | Opens a `.txt` or `.md` file as a note                      |
| `Ctrl+S`           | Saves the note to its file, or asks where first             |
| `Ctrl+Shift+S`     | Saves the note to a file of your choice                     |
| `Ctrl+F`           | Puts the caret in the search box                            |
| `Ctrl+D`           | Pins or unpins the note                                     |
| `Ctrl+Plus`, `-`   | Zooms the editor in or out                                  |
| `Ctrl+0`           | Restores the editor's zoom                                  |
| `Ctrl+1` to `4`    | Goes to Home, Notes, Controls or Settings                   |
| `Alt`              | Shows the menu bar's access keys, like `F` for File         |
| `F1`               | Shows the keyboard shortcuts                                |
| `F9`               | Switches between the light and dark themes                  |
| `F11`              | Switches between maximized and full screen                  |
| `Ctrl+Z`, `X`, ... | Work in text boxes as usual, and so do the Edit menu's      |

## Build and run

```powershell
Program\WinUi3Sample\HumanRun.cmd
```

`HumanRun.cmd`, which you can also double-click in File Explorer, builds the sample with `dotnet`
and opens it on the primary display. It needs the .NET 10 SDK, and downloads the Windows App SDK's
packages from NuGet the first time. The app is unpackaged and self-contained: it runs from its exe
in `target\DotNet\bin\WinUi3Sample\release_win-x64`, with the Windows App SDK's runtime beside it,
without an MSIX package or the Windows App SDK's runtime installed. It runs on the .NET 10 runtime
that comes with the SDK.

Visual Studio isn't needed to build it, but with its WinUI application development workload, it can
open `WinUi3Sample.csproj` to edit the XAML while the app runs, with XAML Hot Reload.

| Argument                 | Value                                                                  |
| ------------------------ | ---------------------------------------------------------------------- |
| `--display <identifier>` | The display to open on, as `IdentifyDisplays` prints it, or            |
|                          | `Primary`                                                              |
| `--no-activate`          | Opens beneath the other windows, without taking focus, and keeps every |
|                          | window of the app, like its menus, from taking it later                |
| `--page <page>`          | The page to open on: `Home`, `Notes`, `Controls` or `Settings`         |
| `--theme <theme>`        | `Light`, `Dark` or `System`. `Light` by default                        |
| `--open-menu <menu>`     | Opens a menu of the menu bar once the window shows, like `File`, or a  |
|                          | submenu, like `View/Theme`                                             |

[RunInBackground](../RunInBackground/README.md) runs it for coding agents, and its `--arguments`
passes it the others, like for a screenshot of a page in the dark theme with a menu open:

```powershell
Program\RunInBackground\RunInBackground.ps1 WinUi3Sample --arguments "--page Notes --theme Dark --open-menu View/Theme"
```

It has no tests or scripts: RunInBackground's scripts drive programs that are built on
Library/UserInterface.

## Layout

| Path                         | Contents                                                              |
| ---------------------------- | --------------------------------------------------------------------- |
| `Source/App.xaml`            | The app, and its resources: WinUI's controls' styles, and its own     |
| `Source/MainWindow.xaml`     | The window: the title bar, menu bar, navigation and status bar        |
| `Source/MainWindow.xaml.cs`  | What the menus and keys do, and opening the window on a display       |
| `Source/Pages`               | The Home, Notes, Controls and Settings pages                          |
| `Source/AppState.cs`         | The notes and the settings, which the window and pages bind to        |
| `Source/Note.cs`             | A note                                                                |
| `Source/SettingsCard.cs`     | A settings page's card, a templated control                           |
| `Source/StartupOptions.cs`   | The command line                                                      |
| `Source/Displays.cs`         | Finds a display by Library/Displays' identifier, like `CVVNY2`        |
| `Source/NativeWindow.cs`     | Shows the window without activating it, which WinUI has no API for    |
| `Assets`                     | The app's icon                                                        |
| `WinUi3Sample.csproj`        | The project, with the Windows App SDK's package                       |
| `Directory.Build.props`      | Builds into `target\DotNet`, instead of `bin` and `obj` folders here  |
| `Application.manifest`       | Per-monitor DPI awareness, which WinUI 3 needs                        |
| `HumanRun.ps1`               | Builds the sample and runs it on the primary display, for people      |
| `HumanRun.cmd`               | Runs `HumanRun.ps1`, for double-clicking in File Explorer             |
