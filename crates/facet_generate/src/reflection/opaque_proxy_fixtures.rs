//! Shared types for opaque + proxy reflection tests.
//!
//! For optional opaque fields, use a proxy newtype (e.g. `OptionalUserId`) rather than
//! `Option<opaque>` with a field-level proxy — reflection peels through `Option` only when
//! the inner type carries `#[facet(opaque, proxy = T)]` on its field definition.

use facet::Facet;

/// Lattice stand-in. Cannot implement `Facet` due to orphan rule.
pub struct WithoutFacet(pub String);

/// Lattice optional stand-in.
pub struct WithoutFacetOption(pub Option<WithoutFacet>);

/// Lattice list stand-in.
pub struct WithoutFacetList(pub Vec<WithoutFacet>);

/// Lattice box stand-in.
pub struct WithoutFacetBox(pub Box<WithoutFacet>);

/// Domain ID with `Facet`. Wire anchor for typegen.
#[derive(Facet)]
pub struct UserId(pub String);

impl From<UserId> for WithoutFacet {
    fn from(value: UserId) -> Self {
        Self(value.0)
    }
}

impl From<WithoutFacet> for UserId {
    fn from(value: WithoutFacet) -> Self {
        Self(value.0)
    }
}

impl From<&WithoutFacet> for UserId {
    fn from(value: &WithoutFacet) -> Self {
        Self(value.0.clone())
    }
}

/// Proxy for optional opaque lattice IDs.
#[derive(Facet)]
pub struct OptionalUserId(pub Option<UserId>);

impl From<OptionalUserId> for WithoutFacetOption {
    fn from(value: OptionalUserId) -> Self {
        WithoutFacetOption(value.0.map(|v| WithoutFacet(v.0)))
    }
}

impl From<&WithoutFacetOption> for OptionalUserId {
    fn from(value: &WithoutFacetOption) -> Self {
        OptionalUserId(value.0.as_ref().map(|v| UserId(v.0.clone())))
    }
}

/// Proxy for list opaque lattice IDs.
#[derive(Facet)]
pub struct UserIdList(pub Vec<UserId>);

impl From<UserIdList> for WithoutFacetList {
    fn from(value: UserIdList) -> Self {
        WithoutFacetList(value.0.into_iter().map(|v| WithoutFacet(v.0)).collect())
    }
}

impl From<&WithoutFacetList> for UserIdList {
    fn from(value: &WithoutFacetList) -> Self {
        UserIdList(value.0.iter().map(|v| UserId(v.0.clone())).collect())
    }
}

/// Proxy for boxed opaque lattice IDs.
#[derive(Facet)]
pub struct UserIdBox(pub Box<UserId>);

impl From<UserIdBox> for WithoutFacetBox {
    fn from(value: UserIdBox) -> Self {
        WithoutFacetBox(Box::new(WithoutFacet(value.0.0)))
    }
}

impl From<&WithoutFacetBox> for UserIdBox {
    fn from(value: &WithoutFacetBox) -> Self {
        UserIdBox(Box::new(UserId(value.0.0.clone())))
    }
}

/// Second proxy type for multi-field tests.
#[derive(Facet)]
pub struct OrgId(pub String);

impl From<OrgId> for WithoutFacet {
    fn from(value: OrgId) -> Self {
        Self(value.0)
    }
}

impl From<WithoutFacet> for OrgId {
    fn from(value: WithoutFacet) -> Self {
        Self(value.0)
    }
}

impl From<&WithoutFacet> for OrgId {
    fn from(value: &WithoutFacet) -> Self {
        Self(value.0.clone())
    }
}
