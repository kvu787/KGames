using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace WinUi3Sample;

// A row of the settings page: an icon, a header and a description, with the setting's control as
// its content, after the Windows Community Toolkit's SettingsCard, which Windows 11's own settings
// look like. App.xaml has its template.
public sealed partial class SettingsCard : ContentControl {
    public static readonly DependencyProperty HeaderProperty =
        DependencyProperty.Register(nameof(Header), typeof(string), typeof(SettingsCard), new PropertyMetadata(""));

    public static readonly DependencyProperty DescriptionProperty =
        DependencyProperty.Register(nameof(Description), typeof(string), typeof(SettingsCard), new PropertyMetadata(""));

    public static readonly DependencyProperty GlyphProperty =
        DependencyProperty.Register(nameof(Glyph), typeof(string), typeof(SettingsCard), new PropertyMetadata(""));

    public string Header {
        get => (string)this.GetValue(HeaderProperty);
        set => this.SetValue(HeaderProperty, value);
    }

    public string Description {
        get => (string)this.GetValue(DescriptionProperty);
        set => this.SetValue(DescriptionProperty, value);
    }

    // A character of Segoe Fluent Icons.
    public string Glyph {
        get => (string)this.GetValue(GlyphProperty);
        set => this.SetValue(GlyphProperty, value);
    }
}
