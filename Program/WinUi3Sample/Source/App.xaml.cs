using Microsoft.UI.Xaml;

namespace WinUi3Sample;

public partial class App : Application {
    // What the window and its pages share: the notes, and the settings.
    public static AppState State { get; } = new();

    public static MainWindow Window { get; private set; } = null!;

    public App() {
        this.InitializeComponent();
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args) {
        StartupOptions options;
        Displays.Display? display;
        try {
            options = StartupOptions.Parse(Environment.GetCommandLineArgs().Skip(1));
            display = options.Display == null ? null : Displays.Find(options.Display);
        } catch (StartupException exception) {
            // The app has no console, but whoever started it may read its standard error, like
            // RunInBackground does.
            Console.Error.WriteLine(exception.Message);
            Environment.Exit(1);
            return;
        }

        if (options.Theme is { } theme) {
            State.Theme = theme;
        }
        if (options.NoActivate) {
            // Before the first window, which WinUI makes on this thread, as it does its popups.
            try {
                NativeWindow.PreventActivation();
            } catch (StartupException exception) {
                Console.Error.WriteLine(exception.Message);
                Environment.Exit(1);
                return;
            }
        }
        Window = new MainWindow();
        Window.Open(display, options);
    }
}
