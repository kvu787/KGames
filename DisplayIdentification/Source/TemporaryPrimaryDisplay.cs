using System.ComponentModel;

namespace DisplayIdentification;

// Makes a display primary until it is restored or disposed.
internal sealed class TemporaryPrimaryDisplay : IDisposable {
    private readonly Lock sync = new();
    private readonly Display original;
    private readonly Display temporary;
    private readonly bool ownsPendingRestore;
    private bool restored;

    internal TemporaryPrimaryDisplay(Display original, Display temporary) {
        this.original = original;
        this.temporary = temporary;

        // An existing pending restore belongs to another launch that is still running, and it already has the original primary
        // display, so it is kept.
        this.ownsPendingRestore = PendingRestore.Load() == null;
        if (this.ownsPendingRestore) {
            PendingRestore.Create(original, temporary).Save();
        }

        try {
            PrimaryDisplay.Set(temporary, persist: false);
        } catch {
            this.DeleteOwnPendingRestore();
            throw;
        }
    }

    public void Dispose() {
        this.Restore();
    }

    // Safe to call more than once and from another thread.
    internal void Restore() {
        lock (this.sync) {
            if (this.restored) {
                return;
            }

            this.restored = true;
            try {
                // If the primary display changed again in the meantime, by the user or by another launch, keep that change.
                Display? primary = Display.GetAll().Find(display => display.IsPrimary);
                if (primary != null && primary.HasSameSource(this.temporary)) {
                    PrimaryDisplay.Set(this.original, persist: false);
                    Log.Info($"Made {this.original.Key} the primary display again.");
                } else {
                    Log.Info("The primary display was changed by something else in the meantime, so it is kept.");
                }

                this.DeleteOwnPendingRestore();
            } catch (Exception exception) when (exception is Win32Exception or CommandException) {
                Log.Error($"Couldn't make {this.original.Key} the primary display again: {exception.Message} Run \"DisplayIdentification restore\" to try again.");
            }
        }
    }

    private void DeleteOwnPendingRestore() {
        if (this.ownsPendingRestore) {
            PendingRestore.Delete();
        }
    }
}
