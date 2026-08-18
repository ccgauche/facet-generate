package com.example

/// FFI view. Field holds a lattice type. Serialized as `UserId`.
data class SignInView(
    val userId: com.example.UserId,
)

/// Domain ID with `Facet`. Wire anchor for typegen.
data class UserId(
    val value: String,
)

/// Newtype wrapper around an opaque lattice field. Serialized as `UserId`.
data class UserIdWrapper(
    val value: com.example.UserId,
)
