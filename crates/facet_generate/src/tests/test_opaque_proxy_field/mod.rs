#![expect(unused)]

use facet::Facet;

use crate::reflection::opaque_proxy_fixtures::{UserId, WithoutFacet};

/// FFI view. Field holds a lattice type. Serialized as `UserId`.
#[derive(Facet)]
pub struct SignInView {
    #[facet(opaque, proxy = UserId)]
    pub user_id: WithoutFacet,
}

/// Newtype wrapper around an opaque lattice field. Serialized as `UserId`.
#[derive(Facet)]
pub struct UserIdWrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

crate::test! { SignInView, UserId, UserIdWrapper for kotlin, swift, typescript, csharp }
