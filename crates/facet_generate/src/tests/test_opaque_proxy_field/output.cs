using CommunityToolkit.Mvvm.ComponentModel;
using Facet.Runtime.Serde;
using System.Collections.Generic;
using System.Collections.ObjectModel;

namespace Example;

/// FFI view. Field holds a lattice type. Serialized as `UserId`.
public partial class SignInView : ObservableObject {
    [ObservableProperty]
    private UserId _userId;
}

/// Domain ID with `Facet`. Wire anchor for typegen.
public partial class UserId : ObservableObject {
    [ObservableProperty]
    private string _value;
}

/// Newtype wrapper around an opaque lattice field. Serialized as `UserId`.
public partial class UserIdWrapper : ObservableObject {
    [ObservableProperty]
    private UserId _value;
}
