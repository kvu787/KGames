// KGames' user interface library, for Win32, Direct3D 12 and DirectWrite,
// after Microsoft's Fluent 2 design system as WinUI 3 implements it on
// Windows 11, with WinUI 3's own values. See README.md.
//
// - Application: what a program is, its settings, the main loop, and its
//   PerfBar's stats.
// - Platform: the window, the clipboard, the clock, and GameInput's devices.
// - Input: keys, buttons and the gamepad, and a frame's input from GameInput.
// - Design: Fluent's theme, tokens and icons, and WinUI's controls' sizes.
// - Drawing: geometry, the painter, text, and the GPU renderer.
// - Core: a frame: the view's tree, its widgets' targets, the input on the
//   tree the user saw, the messages, focus, popups and tooltips.
// - Widgets: WinUI's controls, cards, and the page's bars.
// - Testing: scripts, and running a program without a window.
//
// This includes all of it, which is what a program includes. Everything is
// in the namespace user_interface, but the design system, as AGENTS.md names
// it, which is in user_interface::tokens and user_interface::controls, the
// clock that input, frames and the screen's flips are timed on, in
// user_interface::clock, and the stats with the input latency that every
// program's PerfBar shows, in user_interface::performance.

#pragma once

#include "UserInterface/Application/Attempt.h"
#include "UserInterface/Application/Overlays.h"
#include "UserInterface/Application/Performance.h"
#include "UserInterface/Application/Program.h"
#include "UserInterface/Application/Run.h"
#include "UserInterface/Application/Settings.h"
#include "UserInterface/Core/Interaction.h"
#include "UserInterface/Core/Interface.h"
#include "UserInterface/Core/Tree.h"
#include "UserInterface/Design/Controls.h"
#include "UserInterface/Design/Icons.h"
#include "UserInterface/Design/Theme.h"
#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Geometry.h"
#include "UserInterface/Drawing/Painter.h"
#include "UserInterface/Drawing/Renderer.h"
#include "UserInterface/Drawing/Text.h"
#include "UserInterface/Input/Devices.h"
#include "UserInterface/Input/Events.h"
#include "UserInterface/Input/Reader.h"
#include "UserInterface/Platform/Clipboard.h"
#include "UserInterface/Platform/Clock.h"
#include "UserInterface/Platform/InputDevices.h"
#include "UserInterface/Platform/Window.h"
#include "UserInterface/Testing/Headless.h"
#include "UserInterface/Testing/Script.h"
#include "UserInterface/Widgets/Buttons.h"
#include "UserInterface/Widgets/Card.h"
#include "UserInterface/Widgets/ColorPicker.h"
#include "UserInterface/Widgets/ComboBox.h"
#include "UserInterface/Widgets/Dialog.h"
#include "UserInterface/Widgets/Expander.h"
#include "UserInterface/Widgets/InfoBar.h"
#include "UserInterface/Widgets/InfoCard.h"
#include "UserInterface/Widgets/Layout.h"
#include "UserInterface/Widgets/ListView.h"
#include "UserInterface/Widgets/Menu.h"
#include "UserInterface/Widgets/Navigation.h"
#include "UserInterface/Widgets/NumberBox.h"
#include "UserInterface/Widgets/Palette.h"
#include "UserInterface/Widgets/Progress.h"
#include "UserInterface/Widgets/Scrolling.h"
#include "UserInterface/Widgets/Shell.h"
#include "UserInterface/Widgets/Slider.h"
#include "UserInterface/Widgets/Text.h"
#include "UserInterface/Widgets/TextBox.h"
#include "UserInterface/Widgets/Toggles.h"
#include "UserInterface/Widgets/ToolTip.h"
#include "UserInterface/Widgets/Widget.h"

// The libraries that programs use with the interface.
#include "GameInput/GameInput.h"
#include "Graphics/Graphics.h"
#include "Json/Json.h"
