
/// FFI view. Field holds a lattice type. Serialized as `UserId`.
public struct SignInView {
    public var userId: UserId

    public init(userId: UserId) {
        self.userId = userId
    }
}

/// Domain ID with `Facet`. Wire anchor for typegen.
public struct UserId {
    public var value: String

    public init(value: String) {
        self.value = value
    }
}

/// Newtype wrapper around an opaque lattice field. Serialized as `UserId`.
public struct UserIdWrapper {
    public var value: UserId

    public init(value: UserId) {
        self.value = value
    }
}
