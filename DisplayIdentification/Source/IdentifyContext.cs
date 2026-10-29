using System.Windows.Forms;

namespace DisplayIdentification;

// Shows the identification cards until one of them is closed or the time runs out.
internal sealed class IdentifyContext : ApplicationContext {
    private readonly List<IdentifyForm> forms;
    private readonly System.Windows.Forms.Timer timer = new();
    private bool closing;

    internal IdentifyContext(List<IdentifyForm> forms, TimeSpan duration) {
        this.forms = forms;
        foreach (IdentifyForm form in forms) {
            form.FormClosed += this.OnFormClosed;
            form.Show();
        }

        this.timer.Interval = (int)duration.TotalMilliseconds;
        this.timer.Tick += this.OnTimerTick;
        this.timer.Start();
    }

    protected override void Dispose(bool disposing) {
        if (disposing) {
            this.timer.Dispose();
            foreach (IdentifyForm form in this.forms) {
                form.Dispose();
            }
        }

        base.Dispose(disposing);
    }

    private void OnFormClosed(object? sender, FormClosedEventArgs e) {
        this.CloseAll();
    }

    private void OnTimerTick(object? sender, EventArgs e) {
        this.CloseAll();
    }

    private void CloseAll() {
        if (this.closing) {
            return;
        }

        this.closing = true;
        this.timer.Stop();
        foreach (IdentifyForm form in this.forms) {
            form.Close();
        }

        this.ExitThread();
    }
}
