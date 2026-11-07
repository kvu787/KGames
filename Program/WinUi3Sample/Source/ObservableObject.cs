using System.ComponentModel;
using System.Runtime.CompilerServices;

namespace WinUi3Sample;

// Raises PropertyChanged for x:Bind's one-way and two-way bindings, like the CommunityToolkit's
// ObservableObject, which most WinUI 3 apps use, without the dependency.
public abstract partial class ObservableObject : INotifyPropertyChanged {
    public event PropertyChangedEventHandler? PropertyChanged;

    protected bool Set<T>(ref T field, T value, [CallerMemberName] string name = "") {
        if (EqualityComparer<T>.Default.Equals(field, value)) {
            return false;
        }
        field = value;
        this.OnPropertyChanged(name);
        return true;
    }

    protected void OnPropertyChanged([CallerMemberName] string name = "") {
        this.PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(name));
    }
}
