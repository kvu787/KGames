using System.ComponentModel;
using System.Runtime.InteropServices;

namespace DisplayIdentification;

// Win32 declarations. Constant names match the Windows SDK headers so that they are easy to look up.
internal static partial class NativeMethods {
    internal const int ERROR_SUCCESS = 0;
    internal const int ERROR_ACCESS_DENIED = 5;
    internal const int ERROR_INSUFFICIENT_BUFFER = 122;

    internal struct Luid {
        public uint LowPart;
        public int HighPart;

        internal readonly bool Matches(Luid other) {
            return this.LowPart == other.LowPart && this.HighPart == other.HighPart;
        }
    }

    internal struct PointL {
        public int X;
        public int Y;
    }

    internal struct Rect {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;

        internal readonly int Width => this.Right - this.Left;
        internal readonly int Height => this.Bottom - this.Top;
    }

    internal static Win32Exception Failure(string function, int error) {
        return new Win32Exception(error, $"{function} failed: {new Win32Exception(error).Message}");
    }

    internal static void ThrowIfFailed(string function, int result) {
        if (result != ERROR_SUCCESS) {
            throw Failure(function, result);
        }
    }

    [LibraryImport("kernel32.dll")]
    internal static partial nint GetConsoleWindow();
}
