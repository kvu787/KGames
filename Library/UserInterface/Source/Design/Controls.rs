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

use crate::design::tokens::Curve;
use crate::drawing::geometry::Insets;

// The focus visual: FocusVisualMargin -3 puts it 3 pixels outside a control,
// and checkboxes, radio buttons and switches put it around their label too.
pub const FOCUS_VISUAL_MARGIN: Insets = Insets::all(3.0);
pub const FOCUS_VISUAL_MARGIN_WIDE: Insets = Insets::new(7.0, 3.0, 7.0, 3.0);

// Button, ToggleButton and RepeatButton: ButtonPadding, and
// ButtonBorderThemeThickness.
pub const BUTTON_PADDING: Insets = Insets::new(11.0, 5.0, 11.0, 6.0);
pub const BUTTON_BORDER: f32 = 1.0;
// Between an icon and the text after it.
pub const BUTTON_ICON_GAP: f32 = 8.0;
// A button with only an icon is square.
pub const BUTTON_ICON_ONLY_WIDTH: f32 = 32.0;
// DropDownButton's chevron: U+E96E, 8 pixels, 8 after the text.
pub const DROP_DOWN_CHEVRON_SIZE: f32 = 8.0;
pub const DROP_DOWN_CHEVRON_GAP: f32 = 8.0;

// HyperlinkButton: ButtonPadding too.
pub const HYPERLINK_PADDING: Insets = BUTTON_PADDING;

// CheckBox: CheckBoxSize, CheckBoxGlyphSize, CheckBoxHeight, and
// CheckBoxPadding, between the box and its label.
pub const CHECK_BOX_SIZE: f32 = 20.0;
pub const CHECK_BOX_GLYPH_SIZE: f32 = 12.0;
pub const CHECK_BOX_HEIGHT: f32 = 32.0;
pub const CHECK_BOX_PADDING: Insets = Insets::new(8.0, 5.0, 0.0, 0.0);

// RadioButton: the ellipse, RadioButtonCheckGlyphSize, and the glyph's size
// with the pointer over (RadioButtonCheckGlyphPointerOverSize), pressed, and
// disabled.
pub const RADIO_BUTTON_SIZE: f32 = 20.0;
pub const RADIO_BUTTON_GLYPH: f32 = 12.0;
pub const RADIO_BUTTON_GLYPH_POINTER_OVER: f32 = 14.0;
pub const RADIO_BUTTON_GLYPH_PRESSED: f32 = 10.0;
pub const RADIO_BUTTON_GLYPH_DISABLED: f32 = 14.0;
pub const RADIO_BUTTON_PADDING: Insets = Insets::new(8.0, 6.0, 0.0, 0.0);

// ToggleSwitch: the track, the knob at rest, with the pointer over, and
// pressed, when it stretches 3 pixels from its end; the knob's travel; the
// gap before its content; and ToggleSwitchTopHeaderMargin.
pub const TOGGLE_SWITCH_WIDTH: f32 = 40.0;
pub const TOGGLE_SWITCH_HEIGHT: f32 = 20.0;
pub const TOGGLE_SWITCH_KNOB: f32 = 12.0;
pub const TOGGLE_SWITCH_KNOB_POINTER_OVER: f32 = 14.0;
pub const TOGGLE_SWITCH_KNOB_PRESSED_WIDTH: f32 = 17.0;
pub const TOGGLE_SWITCH_KNOB_PRESSED_HEIGHT: f32 = 14.0;
pub const TOGGLE_SWITCH_KNOB_PRESSED_MARGIN: f32 = 3.0;
pub const TOGGLE_SWITCH_KNOB_TRAVEL: f32 = 20.0;
pub const TOGGLE_SWITCH_CONTENT_GAP: f32 = 12.0;
pub const TOGGLE_SWITCH_HEADER_GAP: f32 = 4.0;

// Slider: SliderHorizontalHeight, SliderTrackThemeHeight with its corners,
// the thumb's layout size, which its disc overhangs by 2 on each side, the
// disc's corners (SliderThumbCornerRadius), and the inner thumb's size and
// its scale per state.
pub const SLIDER_HEIGHT: f32 = 32.0;
pub const SLIDER_TRACK_HEIGHT: f32 = 4.0;
pub const SLIDER_TRACK_RADIUS: f32 = 2.0;
pub const SLIDER_THUMB_SIZE: f32 = 18.0;
pub const SLIDER_THUMB_OVERHANG: f32 = 2.0;
pub const SLIDER_THUMB_RADIUS: f32 = 10.0;
pub const SLIDER_INNER_THUMB_SIZE: f32 = 12.0;
pub const SLIDER_INNER_THUMB_SCALE: f32 = 0.86;
pub const SLIDER_INNER_THUMB_SCALE_POINTER_OVER: f32 = 1.167;
pub const SLIDER_INNER_THUMB_SCALE_PRESSED: f32 = 0.71;
pub const SLIDER_INNER_THUMB_SCALE_DISABLED: f32 = 1.167;
pub const SLIDER_HEADER_GAP: f32 = 4.0;
// The value's tooltip while the thumb is dragged: its padding, and its text,
// 15 pixels.
pub const SLIDER_TOOLTIP_PADDING: Insets = Insets::new(8.0, 3.0, 8.0, 5.0);
pub const SLIDER_TOOLTIP_FONT_SIZE: f32 = 15.0;

// ComboBox: ComboBoxPadding, the chevron's column, its glyph and margin, and
// the keyboard focus cue: a 2-pixel ring 4 pixels outside, with corners of 7.
pub const COMBO_BOX_PADDING: Insets = Insets::new(12.0, 5.0, 0.0, 7.0);
pub const COMBO_BOX_CHEVRON_COLUMN: f32 = 38.0;
pub const COMBO_BOX_CHEVRON_SIZE: f32 = 12.0;
pub const COMBO_BOX_CHEVRON_MARGIN: f32 = 14.0;
pub const COMBO_BOX_MIN_WIDTH: f32 = 64.0;
pub const COMBO_BOX_FOCUS_MARGIN: f32 = 4.0;
pub const COMBO_BOX_FOCUS_THICKNESS: f32 = 2.0;
pub const COMBO_BOX_FOCUS_RADIUS: f32 = 7.0;
// Its popup: ComboBoxDropdownContentMargin, ComboBoxPopupThemeMinWidth, and
// ComboBoxPopupMaxNumberOfItems.
pub const COMBO_BOX_POPUP_PADDING: Insets = Insets::new(0.0, 4.0, 0.0, 4.0);
pub const COMBO_BOX_POPUP_MIN_WIDTH: f32 = 80.0;
pub const COMBO_BOX_POPUP_MAX_ITEMS: usize = 15;
// ComboBoxItem: ComboBoxItemThemePadding, its margin, its corners
// (ComboBoxItemCornerRadius), and the height its padding gives a line.
pub const COMBO_BOX_ITEM_PADDING: Insets = Insets::new(11.0, 5.0, 11.0, 7.0);
pub const COMBO_BOX_ITEM_MARGIN: Insets = Insets::new(5.0, 2.0, 5.0, 2.0);
pub const COMBO_BOX_ITEM_RADIUS: f32 = 3.0;
pub const COMBO_BOX_ITEM_HEIGHT: f32 = 32.0;

// The pill that marks what's selected in a list, a combo box's popup or a
// navigation view: 3 by 16, with round ends, and 10 long while it's pressed.
pub const SELECTION_PILL_THICKNESS: f32 = 3.0;
pub const SELECTION_PILL_LENGTH: f32 = 16.0;
pub const SELECTION_PILL_PRESSED_LENGTH: f32 = 10.0;

// TextBox: TextControlThemePadding, TextControlThemeMinHeight, the header's
// gap (TextBoxTopHeaderMargin), and its delete button, whose column is 30
// wide, with its button inset by TextBoxInnerButtonMargin.
pub const TEXT_BOX_PADDING: Insets = Insets::new(10.0, 5.0, 6.0, 6.0);
pub const TEXT_BOX_HEIGHT: f32 = 32.0;
pub const TEXT_BOX_HEADER_GAP: f32 = 8.0;
pub const TEXT_BOX_BUTTON_WIDTH: f32 = 30.0;
pub const TEXT_BOX_BUTTON_MARGIN: Insets = Insets::new(0.0, 4.0, 4.0, 4.0);
pub const TEXT_BOX_BUTTON_GLYPH: f32 = 12.0;
pub const TEXT_BOX_CARET_WIDTH: f32 = 1.0;
// AutoSuggestBox's query button: 32 wide.
pub const TEXT_BOX_QUERY_BUTTON_WIDTH: f32 = 32.0;

// NumberBox: its inline spin buttons' column, each at least 32 wide, inset
// by 4, and their glyphs.
pub const NUMBER_BOX_SPIN_COLUMN: f32 = 72.0;
pub const NUMBER_BOX_SPIN_GLYPH: f32 = 12.0;
pub const NUMBER_BOX_MIN_WIDTH: f32 = 120.0;

// ListViewItem: ListViewItemMinHeight, its content's padding, and its
// backplate's margin and corners. The pill is on the backplate's left edge.
pub const LIST_ITEM_HEIGHT: f32 = 40.0;
pub const LIST_ITEM_PADDING: Insets = Insets::new(16.0, 0.0, 12.0, 0.0);
pub const LIST_ITEM_BACKPLATE_MARGIN: Insets = Insets::new(4.0, 2.0, 4.0, 2.0);
pub const LIST_ITEM_RADIUS: f32 = 4.0;

// TreeViewItem and the left pane's NavigationViewItem: their heights.
pub const TREE_ITEM_HEIGHT: f32 = 28.0;
pub const NAVIGATION_ITEM_HEIGHT: f32 = 36.0;

// SelectorBar: SelectorBarPadding, SelectorBarItemPadding, and the pill
// under the selected item.
pub const SELECTOR_BAR_PADDING: Insets = Insets::new(0.0, 4.0, 0.0, 4.0);
pub const SELECTOR_BAR_ITEM_PADDING: Insets = Insets::new(12.0, 10.0, 12.0, 7.0);
pub const SELECTOR_BAR_PILL_WIDTH: f32 = 16.0;
pub const SELECTOR_BAR_PILL_HEIGHT: f32 = 3.0;
pub const SELECTOR_BAR_HEIGHT: f32 = 48.0;

// InfoBar: InfoBarMinHeight, InfoBarContentRootPadding, the icon's size and
// InfoBarIconMargin, the title's and message's margins on one line and
// wrapped, InfoBarPanelMargin, and the close button with its margin.
pub const INFO_BAR_MIN_HEIGHT: f32 = 48.0;
pub const INFO_BAR_PADDING_LEFT: f32 = 16.0;
pub const INFO_BAR_ICON_SIZE: f32 = 16.0;
pub const INFO_BAR_ICON_MARGIN: Insets = Insets::new(0.0, 16.0, 14.0, 16.0);
pub const INFO_BAR_TEXT_TOP: f32 = 14.0;
pub const INFO_BAR_MESSAGE_GAP: f32 = 12.0;
pub const INFO_BAR_WRAPPED_MESSAGE_GAP: f32 = 4.0;
pub const INFO_BAR_WRAPPED_BOTTOM: f32 = 18.0;
pub const INFO_BAR_PANEL_MARGIN_RIGHT: f32 = 16.0;
pub const INFO_BAR_CLOSE_BUTTON_SIZE: f32 = 38.0;
pub const INFO_BAR_CLOSE_BUTTON_MARGIN: f32 = 5.0;
pub const INFO_BAR_CLOSE_GLYPH: f32 = 16.0;

// InfoBadge: InfoBadgeMaxHeight, the value's font size and margin, and the
// dot.
pub const INFO_BADGE_HEIGHT: f32 = 16.0;
pub const INFO_BADGE_FONT_SIZE: f32 = 11.0;
pub const INFO_BADGE_TEXT_MARGIN: Insets = Insets::new(4.0, 0.0, 4.0, 2.0);
pub const INFO_BADGE_DOT: f32 = 6.0;

// ToolTip: ToolTipBorderPadding, ToolTipMaxWidth, and how far it is from the
// pointer (DEFAULT_MOUSE_OFFSET) or from the control the keyboard is on.
pub const TOOLTIP_PADDING: Insets = Insets::new(9.0, 6.0, 9.0, 8.0);
pub const TOOLTIP_MAX_WIDTH: f32 = 320.0;
pub const TOOLTIP_OFFSET: f32 = 20.0;

// ProgressBar: ProgressBarTrackHeight, ProgressBarMinHeight, and their
// corners, and the KeySpline that the indeterminate bar's two indicators
// cross it along.
pub const PROGRESS_BAR_TRACK_HEIGHT: f32 = 1.0;
pub const PROGRESS_BAR_TRACK_RADIUS: f32 = 0.5;
pub const PROGRESS_BAR_HEIGHT: f32 = 3.0;
pub const PROGRESS_BAR_RADIUS: f32 = 1.5;
pub const PROGRESS_BAR_CURVE: Curve = Curve(0.4, 0.0, 0.6, 1.0);
// ProgressRing: its default size, and its ring's radius and stroke as parts
// of its size, from its animation's 80-pixel canvas, and the curve of its
// rotation and trim over each half of its 2-second cycle.
pub const PROGRESS_RING_SIZE: f32 = 32.0;
pub const PROGRESS_RING_RADIUS: f32 = 0.4375;
pub const PROGRESS_RING_STROKE: f32 = 0.09375;
pub const PROGRESS_RING_CURVE: Curve = Curve(0.167, 0.167, 0.833, 0.833);

// ScrollBar: ScrollBarSize, its thumb's width when expanded and collapsed,
// how far a collapsed thumb sits toward the edge, the shortest thumb, its
// corners, and how long the pointer rests before it expands, and leaves
// before it collapses.
pub const SCROLL_BAR_SIZE: f32 = 12.0;
pub const SCROLL_BAR_THUMB_EXPANDED: f32 = 6.0;
pub const SCROLL_BAR_THUMB_COLLAPSED: f32 = 2.0;
pub const SCROLL_BAR_THUMB_OFFSET: f32 = 2.0;
pub const SCROLL_BAR_MIN_THUMB: f32 = 30.0;
pub const SCROLL_BAR_RADIUS: f32 = 3.0;
pub const SCROLL_BAR_EXPAND_DELAY: f64 = 0.4;
pub const SCROLL_BAR_CONTRACT_DELAY: f64 = 0.5;
// ScrollViewerScrollBarMargin.
pub const SCROLL_VIEWER_BAR_MARGIN: f32 = 1.0;

// MenuFlyout: MenuFlyoutPresenterThemePadding and FlyoutThemeMinWidth;
// MenuFlyoutItem: MenuFlyoutItemThemePadding, MenuFlyoutItemMargin, the
// height its padding gives a line, the text's indent after an icon or a
// check (MenuFlyoutItemPlaceholderThemeThickness) or both, the gap before a
// keyboard accelerator and a submenu's chevron, and the separator's.
pub const MENU_FLYOUT_PADDING: Insets = Insets::new(0.0, 2.0, 0.0, 2.0);
pub const MENU_FLYOUT_MIN_WIDTH: f32 = 96.0;
pub const MENU_ITEM_PADDING: Insets = Insets::new(11.0, 8.0, 11.0, 9.0);
pub const MENU_ITEM_MARGIN: Insets = Insets::new(4.0, 2.0, 4.0, 2.0);
pub const MENU_ITEM_HEIGHT: f32 = 36.0;
pub const MENU_ITEM_ICON_SIZE: f32 = 16.0;
pub const MENU_ITEM_PLACEHOLDER: f32 = 28.0;
pub const MENU_ITEM_ACCELERATOR_GAP: f32 = 24.0;
pub const MENU_ITEM_CHECK_GLYPH: f32 = 12.0;
pub const MENU_ITEM_CHEVRON_SIZE: f32 = 12.0;
pub const MENU_SEPARATOR_HEIGHT: f32 = 3.0;
// MenuBar: MenuBarHeight, MenuBarItemMargin, MenuBarItemButtonPadding.
pub const MENU_BAR_HEIGHT: f32 = 40.0;
pub const MENU_BAR_ITEM_MARGIN: Insets = Insets::all(4.0);
pub const MENU_BAR_ITEM_PADDING: Insets = Insets::new(10.0, 4.0, 10.0, 4.0);

// Flyout: FlyoutContentPadding, and its narrowest and widest.
pub const FLYOUT_PADDING: Insets = Insets::new(16.0, 15.0, 16.0, 17.0);
pub const FLYOUT_MIN_WIDTH: f32 = 96.0;
pub const FLYOUT_MAX_WIDTH: f32 = 456.0;

// ContentDialog: its narrowest, widest and shortest, ContentDialogPadding,
// the title's margin under it, and ContentDialogButtonSpacing.
pub const DIALOG_MIN_WIDTH: f32 = 320.0;
pub const DIALOG_MAX_WIDTH: f32 = 548.0;
pub const DIALOG_MIN_HEIGHT: f32 = 184.0;
pub const DIALOG_PADDING: f32 = 24.0;
pub const DIALOG_TITLE_GAP: f32 = 12.0;
pub const DIALOG_BUTTON_GAP: f32 = 8.0;

// Expander: ExpanderMinHeight, ExpanderHeaderPadding, the chevron's button
// (ExpanderChevronButtonSize) and ExpanderChevronMargin, and
// ExpanderContentPadding.
pub const EXPANDER_HEADER_HEIGHT: f32 = 48.0;
pub const EXPANDER_HEADER_PADDING_LEFT: f32 = 16.0;
pub const EXPANDER_CHEVRON_BUTTON: f32 = 32.0;
pub const EXPANDER_CHEVRON_MARGIN: Insets = Insets::new(20.0, 0.0, 8.0, 0.0);
pub const EXPANDER_CHEVRON_GLYPH: f32 = 12.0;
pub const EXPANDER_CONTENT_PADDING: f32 = 16.0;

// AppBarButton with its label on the right, and a compact one with only its
// icon, as a toolbar has.
pub const APP_BAR_BUTTON_ICON: f32 = 16.0;
pub const APP_BAR_BUTTON_SIZE: f32 = 40.0;

// ColorPicker: the spectrum's smallest and largest, and its channel
// sliders' height, corners, and inner thumb.
pub const COLOR_SPECTRUM_MIN: f32 = 256.0;
pub const COLOR_SPECTRUM_MAX: f32 = 336.0;
pub const COLOR_SLIDER_HEIGHT: f32 = 12.0;
pub const COLOR_SLIDER_RADIUS: f32 = 6.0;
pub const COLOR_SLIDER_INNER_THUMB: f32 = 10.0;
pub const COLOR_SPECTRUM_RING: f32 = 16.0;
