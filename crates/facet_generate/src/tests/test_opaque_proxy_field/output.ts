type str = string;

/// FFI view. Field holds a lattice type. Serialized as `UserId`.
export class SignInView {
    constructor (public user_id: UserId) {
    }
}

/// Domain ID with `Facet`. Wire anchor for typegen.
export class UserId {
    constructor (public value: str) {
    }
}

/// Newtype wrapper around an opaque lattice field. Serialized as `UserId`.
export class UserIdWrapper {
    constructor (public value: UserId) {
    }
}
