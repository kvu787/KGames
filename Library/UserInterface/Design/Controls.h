// WinUI 3's sizes for each control, from its theme resources and templates in
// microsoft/microsoft-ui-xaml, like ButtonPadding, CheckBoxSize and
// SliderHorizontalThumbWidth. Each is named after its control, and its
// comment says which resource or template value it is. Thicknesses are left,
// top, right, bottom.
//
// XAML lays a line of 14-pixel Segoe UI Variable out 19 pixels tall, from
// the font's metrics, which is how ButtonPadding makes a 32-pixel button.
// The type ramp's lines are 20; widgets center their text in what WinUI's
// sizes give them.

#pragma once

#include "UserInterface/Design/Tokens.h"
#include "UserInterface/Drawing/Geometry.h"

#include <cstddef>

namespace user_interface::controls {

// The focus visual: FocusVisualMargin -3 puts it 3 pixels outside a control,
// and checkboxes, radio buttons and switches put it around their label too.
inline constexpr Insets FocusVisualMargin = Insets::all(3.0f);
inline constexpr Insets FocusVisualMarginWide = Insets(7.0f, 3.0f, 7.0f, 3.0f);

// Button, ToggleButton and RepeatButton: ButtonPadding, and
// ButtonBorderThemeThickness.
inline constexpr Insets ButtonPadding = Insets(11.0f, 5.0f, 11.0f, 6.0f);
inline constexpr float ButtonBorder = 1.0f;
// Between an icon and the text after it.
inline constexpr float ButtonIconGap = 8.0f;
// A button with only an icon is square.
inline constexpr float ButtonIconOnlyWidth = 32.0f;
// DropDownButton's chevron: U+E96E, 8 pixels, 8 after the text.
inline constexpr float DropDownChevronSize = 8.0f;
inline constexpr float DropDownChevronGap = 8.0f;

// HyperlinkButton: ButtonPadding too.
inline constexpr Insets HyperlinkPadding = ButtonPadding;

// CheckBox: CheckBoxSize, CheckBoxGlyphSize, CheckBoxHeight, and
// CheckBoxPadding, between the box and its label.
inline constexpr float CheckBoxSize = 20.0f;
inline constexpr float CheckBoxGlyphSize = 12.0f;
inline constexpr float CheckBoxHeight = 32.0f;
inline constexpr Insets CheckBoxPadding = Insets(8.0f, 5.0f, 0.0f, 0.0f);

// RadioButton: the ellipse, RadioButtonCheckGlyphSize, and the glyph's size
// with the pointer over (RadioButtonCheckGlyphPointerOverSize), pressed, and
// disabled.
inline constexpr float RadioButtonSize = 20.0f;
inline constexpr float RadioButtonGlyph = 12.0f;
inline constexpr float RadioButtonGlyphPointerOver = 14.0f;
inline constexpr float RadioButtonGlyphPressed = 10.0f;
inline constexpr float RadioButtonGlyphDisabled = 14.0f;
inline constexpr Insets RadioButtonPadding = Insets(8.0f, 6.0f, 0.0f, 0.0f);

// ToggleSwitch: the track, the knob at rest, with the pointer over, and
// pressed, when it stretches 3 pixels from its end; the knob's travel; the
// gap before its content; and ToggleSwitchTopHeaderMargin.
inline constexpr float ToggleSwitchWidth = 40.0f;
inline constexpr float ToggleSwitchHeight = 20.0f;
inline constexpr float ToggleSwitchKnob = 12.0f;
inline constexpr float ToggleSwitchKnobPointerOver = 14.0f;
inline constexpr float ToggleSwitchKnobPressedWidth = 17.0f;
inline constexpr float ToggleSwitchKnobPressedHeight = 14.0f;
inline constexpr float ToggleSwitchKnobPressedMargin = 3.0f;
inline constexpr float ToggleSwitchKnobTravel = 20.0f;
inline constexpr float ToggleSwitchContentGap = 12.0f;
inline constexpr float ToggleSwitchHeaderGap = 4.0f;

// Slider: SliderHorizontalHeight, SliderTrackThemeHeight with its corners,
// the thumb's layout size, which its disc overhangs by 2 on each side, the
// disc's corners (SliderThumbCornerRadius), and the inner thumb's size and
// its scale per state.
inline constexpr float SliderHeight = 32.0f;
inline constexpr float SliderTrackHeight = 4.0f;
inline constexpr float SliderTrackRadius = 2.0f;
inline constexpr float SliderThumbSize = 18.0f;
inline constexpr float SliderThumbOverhang = 2.0f;
inline constexpr float SliderThumbRadius = 10.0f;
inline constexpr float SliderInnerThumbSize = 12.0f;
inline constexpr float SliderInnerThumbScale = 0.86f;
inline constexpr float SliderInnerThumbScalePointerOver = 1.167f;
inline constexpr float SliderInnerThumbScalePressed = 0.71f;
inline constexpr float SliderInnerThumbScaleDisabled = 1.167f;
inline constexpr float SliderHeaderGap = 4.0f;
// The value's tooltip while the thumb is dragged: its padding, and its text,
// 15 pixels.
inline constexpr Insets SliderTooltipPadding = Insets(8.0f, 3.0f, 8.0f, 5.0f);
inline constexpr float SliderTooltipFontSize = 15.0f;

// ComboBox: ComboBoxPadding, the chevron's column, its glyph and margin, and
// the keyboard focus cue: a 2-pixel ring 4 pixels outside, with corners of 7.
inline constexpr Insets ComboBoxPadding = Insets(12.0f, 5.0f, 0.0f, 7.0f);
inline constexpr float ComboBoxChevronColumn = 38.0f;
inline constexpr float ComboBoxChevronSize = 12.0f;
inline constexpr float ComboBoxChevronMargin = 14.0f;
inline constexpr float ComboBoxMinWidth = 64.0f;
inline constexpr float ComboBoxFocusMargin = 4.0f;
inline constexpr float ComboBoxFocusThickness = 2.0f;
inline constexpr float ComboBoxFocusRadius = 7.0f;
// Its popup: ComboBoxDropdownContentMargin, ComboBoxPopupThemeMinWidth, and
// ComboBoxPopupMaxNumberOfItems.
inline constexpr Insets ComboBoxPopupPadding = Insets(0.0f, 4.0f, 0.0f, 4.0f);
inline constexpr float ComboBoxPopupMinWidth = 80.0f;
inline constexpr size_t ComboBoxPopupMaxItems = 15;
// ComboBoxItem: ComboBoxItemThemePadding, its margin, its corners
// (ComboBoxItemCornerRadius), and the height its padding gives a line.
inline constexpr Insets ComboBoxItemPadding = Insets(11.0f, 5.0f, 11.0f, 7.0f);
inline constexpr Insets ComboBoxItemMargin = Insets(5.0f, 2.0f, 5.0f, 2.0f);
inline constexpr float ComboBoxItemRadius = 3.0f;
inline constexpr float ComboBoxItemHeight = 32.0f;

// The pill that marks what's selected in a list, a combo box's popup or a
// navigation view: 3 by 16, with round ends, and 10 long while it's pressed.
inline constexpr float SelectionPillThickness = 3.0f;
inline constexpr float SelectionPillLength = 16.0f;
inline constexpr float SelectionPillPressedLength = 10.0f;

// TextBox: TextControlThemePadding, TextControlThemeMinHeight, the header's
// gap (TextBoxTopHeaderMargin), and its delete button, whose column is 30
// wide, with its button inset by TextBoxInnerButtonMargin.
inline constexpr Insets TextBoxPadding = Insets(10.0f, 5.0f, 6.0f, 6.0f);
inline constexpr float TextBoxHeight = 32.0f;
inline constexpr float TextBoxHeaderGap = 8.0f;
inline constexpr float TextBoxButtonWidth = 30.0f;
inline constexpr Insets TextBoxButtonMargin = Insets(0.0f, 4.0f, 4.0f, 4.0f);
inline constexpr float TextBoxButtonGlyph = 12.0f;
inline constexpr float TextBoxCaretWidth = 1.0f;
// AutoSuggestBox's query button: 32 wide.
inline constexpr float TextBoxQueryButtonWidth = 32.0f;

// NumberBox: its inline spin buttons' column, each at least 32 wide, inset
// by 4, and their glyphs.
inline constexpr float NumberBoxSpinColumn = 72.0f;
inline constexpr float NumberBoxSpinGlyph = 12.0f;
inline constexpr float NumberBoxMinWidth = 120.0f;

// ListViewItem: ListViewItemMinHeight, its content's padding, and its
// backplate's margin and corners. The pill is on the backplate's left edge.
inline constexpr float ListItemHeight = 40.0f;
inline constexpr Insets ListItemPadding = Insets(16.0f, 0.0f, 12.0f, 0.0f);
inline constexpr Insets ListItemBackplateMargin = Insets(4.0f, 2.0f, 4.0f, 2.0f);
inline constexpr float ListItemRadius = 4.0f;

// TreeViewItem and the left pane's NavigationViewItem: their heights.
inline constexpr float TreeItemHeight = 28.0f;
inline constexpr float NavigationItemHeight = 36.0f;

// SelectorBar: SelectorBarPadding, SelectorBarItemPadding, and the pill
// under the selected item.
inline constexpr Insets SelectorBarPadding = Insets(0.0f, 4.0f, 0.0f, 4.0f);
inline constexpr Insets SelectorBarItemPadding = Insets(12.0f, 10.0f, 12.0f, 7.0f);
inline constexpr float SelectorBarPillWidth = 16.0f;
inline constexpr float SelectorBarPillHeight = 3.0f;
inline constexpr float SelectorBarHeight = 48.0f;

// InfoBar: InfoBarMinHeight, InfoBarContentRootPadding, the icon's size and
// InfoBarIconMargin, the title's and message's margins on one line and
// wrapped, InfoBarPanelMargin, and the close button with its margin.
inline constexpr float InfoBarMinHeight = 48.0f;
inline constexpr float InfoBarPaddingLeft = 16.0f;
inline constexpr float InfoBarIconSize = 16.0f;
inline constexpr Insets InfoBarIconMargin = Insets(0.0f, 16.0f, 14.0f, 16.0f);
inline constexpr float InfoBarTextTop = 14.0f;
inline constexpr float InfoBarMessageGap = 12.0f;
inline constexpr float InfoBarWrappedMessageGap = 4.0f;
inline constexpr float InfoBarWrappedBottom = 18.0f;
inline constexpr float InfoBarPanelMarginRight = 16.0f;
inline constexpr float InfoBarCloseButtonSize = 38.0f;
inline constexpr float InfoBarCloseButtonMargin = 5.0f;
inline constexpr float InfoBarCloseGlyph = 16.0f;

// InfoBadge: InfoBadgeMaxHeight, the value's font size and margin, and the
// dot.
inline constexpr float InfoBadgeHeight = 16.0f;
inline constexpr float InfoBadgeFontSize = 11.0f;
inline constexpr Insets InfoBadgeTextMargin = Insets(4.0f, 0.0f, 4.0f, 2.0f);
inline constexpr float InfoBadgeDot = 6.0f;

// ToolTip: ToolTipBorderPadding, ToolTipMaxWidth, and how far it is from the
// pointer (DEFAULT_MOUSE_OFFSET) or from the control the keyboard is on.
inline constexpr Insets TooltipPadding = Insets(9.0f, 6.0f, 9.0f, 8.0f);
inline constexpr float TooltipMaxWidth = 320.0f;
inline constexpr float TooltipOffset = 20.0f;

// ProgressBar: ProgressBarTrackHeight, ProgressBarMinHeight, and their
// corners, and the KeySpline that the indeterminate bar's two indicators
// cross it along.
inline constexpr float ProgressBarTrackHeight = 1.0f;
inline constexpr float ProgressBarTrackRadius = 0.5f;
inline constexpr float ProgressBarHeight = 3.0f;
inline constexpr float ProgressBarRadius = 1.5f;
inline constexpr tokens::Curve ProgressBarCurve = tokens::Curve(0.4f, 0.0f, 0.6f, 1.0f);
// ProgressRing: its default size, and its ring's radius and stroke as parts
// of its size, from its animation's 80-pixel canvas, and the curve of its
// rotation and trim over each half of its 2-second cycle.
inline constexpr float ProgressRingSize = 32.0f;
inline constexpr float ProgressRingRadius = 0.4375f;
inline constexpr float ProgressRingStroke = 0.09375f;
inline constexpr tokens::Curve ProgressRingCurve = tokens::Curve(0.167f, 0.167f, 0.833f, 0.833f);

// ScrollBar: ScrollBarSize, its thumb's width when expanded and collapsed,
// how far a collapsed thumb sits toward the edge, the shortest thumb, its
// corners, and how long the pointer rests before it expands, and leaves
// before it collapses.
inline constexpr float ScrollBarSize = 12.0f;
inline constexpr float ScrollBarThumbExpanded = 6.0f;
inline constexpr float ScrollBarThumbCollapsed = 2.0f;
inline constexpr float ScrollBarThumbOffset = 2.0f;
inline constexpr float ScrollBarMinThumb = 30.0f;
inline constexpr float ScrollBarRadius = 3.0f;
inline constexpr double ScrollBarExpandDelay = 0.4;
inline constexpr double ScrollBarContractDelay = 0.5;
// ScrollViewerScrollBarMargin.
inline constexpr float ScrollViewerBarMargin = 1.0f;

// MenuFlyout: MenuFlyoutPresenterThemePadding and FlyoutThemeMinWidth;
// MenuFlyoutItem: MenuFlyoutItemThemePadding, MenuFlyoutItemMargin, the
// height its padding gives a line, the text's indent after an icon or a
// check (MenuFlyoutItemPlaceholderThemeThickness) or both, the gap before a
// keyboard accelerator and a submenu's chevron, and the separator's.
inline constexpr Insets MenuFlyoutPadding = Insets(0.0f, 2.0f, 0.0f, 2.0f);
inline constexpr float MenuFlyoutMinWidth = 96.0f;
inline constexpr Insets MenuItemPadding = Insets(11.0f, 8.0f, 11.0f, 9.0f);
inline constexpr Insets MenuItemMargin = Insets(4.0f, 2.0f, 4.0f, 2.0f);
inline constexpr float MenuItemHeight = 36.0f;
inline constexpr float MenuItemIconSize = 16.0f;
inline constexpr float MenuItemPlaceholder = 28.0f;
inline constexpr float MenuItemAcceleratorGap = 24.0f;
inline constexpr float MenuItemCheckGlyph = 12.0f;
inline constexpr float MenuItemChevronSize = 12.0f;
inline constexpr float MenuSeparatorHeight = 3.0f;
// MenuBar: MenuBarHeight, MenuBarItemMargin, MenuBarItemButtonPadding.
inline constexpr float MenuBarHeight = 40.0f;
inline constexpr Insets MenuBarItemMargin = Insets::all(4.0f);
inline constexpr Insets MenuBarItemPadding = Insets(10.0f, 4.0f, 10.0f, 4.0f);

// Flyout: FlyoutContentPadding, and its narrowest and widest.
inline constexpr Insets FlyoutPadding = Insets(16.0f, 15.0f, 16.0f, 17.0f);
inline constexpr float FlyoutMinWidth = 96.0f;
inline constexpr float FlyoutMaxWidth = 456.0f;

// ContentDialog: its narrowest, widest and shortest, ContentDialogPadding,
// the title's margin under it, and ContentDialogButtonSpacing.
inline constexpr float DialogMinWidth = 320.0f;
inline constexpr float DialogMaxWidth = 548.0f;
inline constexpr float DialogMinHeight = 184.0f;
inline constexpr float DialogPadding = 24.0f;
inline constexpr float DialogTitleGap = 12.0f;
inline constexpr float DialogButtonGap = 8.0f;

// Expander: ExpanderMinHeight, ExpanderHeaderPadding, the chevron's button
// (ExpanderChevronButtonSize) and ExpanderChevronMargin, and
// ExpanderContentPadding.
inline constexpr float ExpanderHeaderHeight = 48.0f;
inline constexpr float ExpanderHeaderPaddingLeft = 16.0f;
inline constexpr float ExpanderChevronButton = 32.0f;
inline constexpr Insets ExpanderChevronMargin = Insets(20.0f, 0.0f, 8.0f, 0.0f);
inline constexpr float ExpanderChevronGlyph = 12.0f;
inline constexpr float ExpanderContentPadding = 16.0f;

// AppBarButton with its label on the right, and a compact one with only its
// icon, as a toolbar has.
inline constexpr float AppBarButtonIcon = 16.0f;
inline constexpr float AppBarButtonSize = 40.0f;

// ColorPicker: the spectrum's smallest and largest, and its channel
// sliders' height, corners, and inner thumb.
inline constexpr float ColorSpectrumMin = 256.0f;
inline constexpr float ColorSpectrumMax = 336.0f;
inline constexpr float ColorSliderHeight = 12.0f;
inline constexpr float ColorSliderRadius = 6.0f;
inline constexpr float ColorSliderInnerThumb = 10.0f;
inline constexpr float ColorSpectrumRing = 16.0f;

} // namespace user_interface::controls
