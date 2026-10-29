using System.Drawing;
using System.Drawing.Drawing2D;
using System.Windows.Forms;

namespace DisplayIdentification;

// A card in the middle of a display that shows how to select the display.
internal sealed class IdentifyForm : Form {
    private const string FontName = "Segoe UI";
    private const string TitleFontName = "Segoe UI Semibold";
    private const TextFormatFlags LineFormat = TextFormatFlags.HorizontalCenter | TextFormatFlags.VerticalCenter | TextFormatFlags.SingleLine | TextFormatFlags.EndEllipsis | TextFormatFlags.NoPadding;

    private static readonly Color BackgroundColor = Color.FromArgb(32, 32, 32);
    private static readonly Color AccentColor = Color.FromArgb(76, 194, 255);
    private static readonly Color TextColor = Color.White;
    private static readonly Color SecondaryTextColor = Color.FromArgb(170, 170, 170);

    private readonly Rectangle cardBounds;
    private readonly string title;
    private readonly string key;
    private readonly string details;

    internal IdentifyForm(IReadOnlyList<Display> displays, NativeMethods.Rect monitorBounds) {
        int width = monitorBounds.Width / 2;
        int height = monitorBounds.Height * 3 / 10;
        this.cardBounds = new Rectangle(monitorBounds.Left + ((monitorBounds.Width - width) / 2), monitorBounds.Top + ((monitorBounds.Height - height) / 2), width, height);

        Display first = displays[0];
        this.title = string.Join(" + ", displays.Select(display => display.Name.Length > 0 ? display.Name : display.Model));
        this.key = string.Join("   ", displays.Select(display => display.Key));
        this.details = $"{first.ShortDeviceName}   ·   {first.Width} × {first.Height} at ({first.X}, {first.Y})" + (first.IsPrimary ? "   ·   Primary" : "");

        this.Text = "Display Identification";
        this.FormBorderStyle = FormBorderStyle.None;
        this.StartPosition = FormStartPosition.Manual;
        this.Bounds = this.cardBounds;
        this.ShowInTaskbar = false;
        this.TopMost = true;
        this.BackColor = BackgroundColor;
        this.DoubleBuffered = true;
        this.KeyPreview = true;
    }

    protected override void OnShown(EventArgs e) {
        base.OnShown(e);

        // Undo any scaling from moving onto a display with a different DPI. The card is laid out in physical pixels.
        this.Bounds = this.cardBounds;
    }

    protected override void OnDpiChanged(DpiChangedEventArgs e) {
        e.Cancel = true;
        base.OnDpiChanged(e);
    }

    protected override void OnClick(EventArgs e) {
        base.OnClick(e);
        this.Close();
    }

    protected override void OnKeyDown(KeyEventArgs e) {
        base.OnKeyDown(e);
        if (e.KeyCode == Keys.Escape) {
            this.Close();
        }
    }

    protected override void OnPaint(PaintEventArgs e) {
        base.OnPaint(e);
        Rectangle client = this.ClientRectangle;
        int height = client.Height;

        using (Pen border = new(AccentColor, Math.Max(2, height / 60))) {
            border.Alignment = PenAlignment.Inset;
            e.Graphics.DrawRectangle(border, client);
        }

        using Font titleFont = new(TitleFontName, height / 6f, GraphicsUnit.Pixel);
        using Font keyFont = new(FontName, height / 9f, GraphicsUnit.Pixel);
        using Font detailsFont = new(FontName, height / 16f, GraphicsUnit.Pixel);
        using Font hintFont = new(FontName, height / 24f, GraphicsUnit.Pixel);
        int left = height / 10;
        int width = client.Width - (2 * left);
        TextRenderer.DrawText(e.Graphics, this.title, titleFont, new Rectangle(left, height * 10 / 100, width, height * 28 / 100), TextColor, LineFormat);
        TextRenderer.DrawText(e.Graphics, this.key, keyFont, new Rectangle(left, height * 40 / 100, width, height * 20 / 100), AccentColor, LineFormat);
        TextRenderer.DrawText(e.Graphics, this.details, detailsFont, new Rectangle(left, height * 62 / 100, width, height * 12 / 100), TextColor, LineFormat);
        TextRenderer.DrawText(e.Graphics, "Click or press Esc to close", hintFont, new Rectangle(left, height * 80 / 100, width, height * 10 / 100), SecondaryTextColor, LineFormat);
    }
}
