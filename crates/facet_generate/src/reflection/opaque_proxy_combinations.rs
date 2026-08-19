//! Exhaustive opaque / proxy combinations with other `facet` attributes and container shapes.
//!
//! Coverage matrix:
//! - **Containers**: struct, newtype, tuple struct (first/middle/last), enum newtype/struct/tuple variants
//! - **Attributes**: `skip`, `rename`, `rename_all`, `fg::namespace`, `fg::namespace` (cleared), `transparent`, `fg::bytes`
//! - **Codegen-only** (not exercised here): `fg::serialized_as`, `fg::branded`, `fg::public`, `fg::readonly`
//! - **Enum tagging**: external, internal (`tag`), adjacent (`tag` + `content`)
//! - **Enum shapes**: unit / newtype / struct / tuple variants; single-field struct vs newtype dispatch
//! - **Wrapped types**: `OptionalUserId`, `UserIdList`, `UserIdBox`, transparent wrappers, nested transparent
//! - **Mixed**: opaque without proxy alongside proxy, bytes + proxy, skip + proxy on same field
//! - **Deep stacks**: triple transparent peel, enum-in-enum, map values, box/option chains
//! - **Container matrix**: `Arc`, refs, arrays, sets, nested `Option`/`Box`/`Vec` through transparent peel
//! - **Negative**: opaque-only fields collapse to unit struct / unit variant

use facet::Facet;

use super::opaque_proxy_fixtures::{
    OptionalUserId, OrgId, UserId, UserIdBox, UserIdList, WithoutFacet, WithoutFacetBox,
    WithoutFacetList, WithoutFacetOption,
};
use crate::reflect;

// --- Field attributes: skip, rename, rename_all ---

#[test]
fn skip_on_opaque_proxy_field() {
    #[derive(Facet)]
    struct WithSkip {
        normal: u32,
        #[facet(skip)]
        #[facet(opaque, proxy = UserId)]
        user_id: WithoutFacet,
    }

    insta::assert_yaml_snapshot!(reflect!(WithSkip, UserId).unwrap());
}

#[test]
fn rename_on_opaque_proxy_field() {
    #[derive(Facet)]
    struct WithRename {
        #[facet(opaque, proxy = UserId, rename = "userId")]
        user_id: WithoutFacet,
    }

    insta::assert_yaml_snapshot!(reflect!(WithRename, UserId).unwrap());
}

#[test]
fn rename_all_with_opaque_proxy_field() {
    #[derive(Facet)]
    #[facet(rename_all = "camelCase")]
    struct WithRenameAll {
        #[facet(opaque, proxy = UserId)]
        owner_user_id: WithoutFacet,
    }

    insta::assert_yaml_snapshot!(reflect!(WithRenameAll, UserId).unwrap());
}

// --- Namespace annotations ---

#[test]
fn namespace_on_proxy_type() {
    use crate as fg;

    #[derive(Facet)]
    #[facet(fg::namespace = "ids")]
    struct NamespacedUserId(String);

    impl From<NamespacedUserId> for WithoutFacet {
        fn from(value: NamespacedUserId) -> Self {
            WithoutFacet(value.0)
        }
    }

    impl From<WithoutFacet> for NamespacedUserId {
        fn from(value: WithoutFacet) -> Self {
            Self(value.0)
        }
    }

    impl From<&WithoutFacet> for NamespacedUserId {
        fn from(value: &WithoutFacet) -> Self {
            Self(value.0.clone())
        }
    }

    #[derive(Facet)]
    struct Holder {
        #[facet(opaque, proxy = NamespacedUserId)]
        user_id: WithoutFacet,
    }

    insta::assert_yaml_snapshot!(reflect!(Holder, NamespacedUserId).unwrap());
}

#[test]
fn namespace_on_opaque_proxy_field() {
    use crate as fg;

    #[derive(Facet)]
    struct Holder {
        #[facet(fg::namespace = "wire")]
        #[facet(opaque, proxy = UserId)]
        user_id: WithoutFacet,
    }

    insta::assert_yaml_snapshot!(reflect!(Holder, UserId).unwrap());
}

#[test]
fn transparent_wrapper_with_namespace_and_opaque_proxy() {
    mod wrappers {
        use crate as fg;
        use facet::Facet;

        use super::super::opaque_proxy_fixtures::{UserId, WithoutFacet};

        #[derive(Facet)]
        #[facet(fg::namespace = "wrappers")]
        #[facet(transparent)]
        pub struct TransparentWrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);
    }

    #[derive(Facet)]
    struct Parent {
        wrapped: wrappers::TransparentWrapper,
    }

    insta::assert_yaml_snapshot!(reflect!(Parent, UserId).unwrap());
}

// --- Wrapped opaque field types: Option, Vec, Box ---

#[test]
fn option_opaque_proxy_field() {
    #[derive(Facet)]
    struct WithOption {
        #[facet(opaque, proxy = OptionalUserId)]
        user_id: WithoutFacetOption,
    }

    insta::assert_yaml_snapshot!(reflect!(WithOption, UserId, OptionalUserId).unwrap());
}

#[test]
fn vec_opaque_proxy_field() {
    #[derive(Facet)]
    struct WithVec {
        #[facet(opaque, proxy = UserIdList)]
        user_ids: WithoutFacetList,
    }

    insta::assert_yaml_snapshot!(reflect!(WithVec, UserId, UserIdList).unwrap());
}

#[test]
fn box_opaque_proxy_field() {
    #[derive(Facet)]
    struct WithBox {
        #[facet(opaque, proxy = UserIdBox)]
        user_id: WithoutFacetBox,
    }

    insta::assert_yaml_snapshot!(reflect!(WithBox, UserId, UserIdBox).unwrap());
}

#[test]
fn option_transparent_wrapper_opaque_proxy() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct TransparentWrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    struct WithOption {
        user_id: Option<TransparentWrapper>,
    }

    insta::assert_yaml_snapshot!(reflect!(WithOption, UserId).unwrap());
}

#[test]
fn vec_transparent_wrapper_opaque_proxy() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct TransparentWrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    struct WithVec {
        user_ids: Vec<TransparentWrapper>,
    }

    insta::assert_yaml_snapshot!(reflect!(WithVec, UserId).unwrap());
}

// --- Tuple struct positions and arity ---

#[test]
fn tuple_struct_option_transparent_wrapper_opaque_proxy() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct TransparentWrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    struct WithOption(Option<TransparentWrapper>);

    insta::assert_yaml_snapshot!(reflect!(WithOption, UserId).unwrap());
}

#[test]
fn tuple_struct_three_fields_proxy_middle() {
    #[derive(Facet)]
    struct Triple(u8, #[facet(opaque, proxy = UserId)] WithoutFacet, u32);

    insta::assert_yaml_snapshot!(reflect!(Triple, UserId).unwrap());
}

#[test]
fn tuple_struct_skip_and_opaque_proxy() {
    #[derive(Facet)]
    struct Triple(
        u8,
        #[facet(skip)] u16,
        #[facet(opaque, proxy = UserId)] WithoutFacet,
    );

    insta::assert_yaml_snapshot!(reflect!(Triple, UserId).unwrap());
}

// --- Multiple proxy fields ---

#[test]
fn struct_two_opaque_proxy_fields() {
    #[derive(Facet)]
    struct WithTwo {
        #[facet(opaque, proxy = UserId)]
        user_id: WithoutFacet,
        #[facet(opaque, proxy = OrgId)]
        org_id: WithoutFacet,
    }

    insta::assert_yaml_snapshot!(reflect!(WithTwo, UserId, OrgId).unwrap());
}

// --- Struct field shape: nested newtype ---

#[test]
fn struct_field_nested_newtype_opaque_proxy() {
    #[derive(Facet)]
    struct Inner(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    struct Outer {
        inner: Inner,
    }

    insta::assert_yaml_snapshot!(reflect!(Outer, UserId).unwrap());
}

// --- Enum: struct variants ---

#[test]
fn enum_struct_variant_only_opaque_proxy() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        SignIn {
            #[facet(opaque, proxy = UserId)]
            user_id: WithoutFacet,
        },
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_struct_variant_mixed_opaque_proxy_and_normal() {
    struct OtherOpaque;

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        SignIn {
            session_id: u64,
            #[facet(opaque)]
            hidden: OtherOpaque,
            #[facet(opaque, proxy = UserId)]
            user_id: WithoutFacet,
        },
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_struct_variant_skip_on_opaque_proxy_field() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        SignIn {
            #[facet(skip)]
            #[facet(opaque, proxy = UserId)]
            user_id: WithoutFacet,
        },
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_struct_variant_rename_on_opaque_proxy_field() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        SignIn {
            #[facet(opaque, proxy = UserId, rename = "userId")]
            user_id: WithoutFacet,
        },
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_struct_variant_only_opaque_without_proxy() {
    struct OtherOpaque;
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Hidden {
            #[facet(opaque)]
            hidden: OtherOpaque,
        },
    }

    insta::assert_yaml_snapshot!(reflect!(Event).unwrap());
}

// --- Enum: tuple variants ---

#[test]
fn enum_tuple_variant_three_fields_proxy_middle() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Triple(u8, #[facet(opaque, proxy = UserId)] WithoutFacet, u32),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_tuple_variant_all_opaque_without_proxy() {
    struct OtherOpaque;
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Hidden(#[facet(opaque)] OtherOpaque, #[facet(opaque)] OtherOpaque),
    }

    insta::assert_yaml_snapshot!(reflect!(Event).unwrap());
}

#[test]
fn enum_tuple_variant_two_fields_only_opaque_without_proxy() {
    struct OtherOpaque;
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Hidden(String, #[facet(opaque)] OtherOpaque),
    }

    insta::assert_yaml_snapshot!(reflect!(Event).unwrap());
}

// --- Enum: newtype variants ---

#[test]
fn enum_newtype_variant_option_opaque_proxy() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        MaybeUser(#[facet(opaque, proxy = OptionalUserId)] WithoutFacetOption),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId, OptionalUserId).unwrap());
}

#[test]
fn enum_newtype_variant_box_opaque_proxy() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        User(#[facet(opaque, proxy = UserIdBox)] WithoutFacetBox),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId, UserIdBox).unwrap());
}

// --- Enum: skip variant / rename variant ---

#[test]
fn enum_skip_variant_with_opaque_proxy_field() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Visible(u32),
        #[facet(skip)]
        Hidden {
            #[facet(opaque, proxy = UserId)]
            user_id: WithoutFacet,
        },
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_rename_variant_with_opaque_proxy_newtype() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        #[facet(rename = "UserSignedIn")]
        SignIn(#[facet(opaque, proxy = UserId)] WithoutFacet),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

// --- Transparent on container with opaque proxy inner ---

#[test]
fn transparent_newtype_opaque_proxy_direct() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct UserIdView(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    struct Parent {
        id: UserIdView,
    }

    insta::assert_yaml_snapshot!(reflect!(Parent, UserId).unwrap());
}

// --- Unit struct edge: proxy-only vs opaque-only ---

#[test]
fn struct_only_opaque_without_proxy_becomes_unit() {
    struct OtherOpaque;
    #[derive(Facet)]
    struct OnlyOpaque {
        #[facet(opaque)]
        hidden: OtherOpaque,
    }

    insta::assert_yaml_snapshot!(reflect!(OnlyOpaque).unwrap(), @"
    ? namespace: ROOT
      name: OnlyOpaque
    : UNITSTRUCT: []
    ");
}

#[test]
fn struct_variant_only_opaque_without_proxy_becomes_unit() {
    struct OtherOpaque;
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Hidden {
            #[facet(opaque)]
            hidden: OtherOpaque,
        },
    }

    insta::assert_yaml_snapshot!(reflect!(Event).unwrap(), @"
    ? namespace: ROOT
      name: Event
    : ENUM:
        - 0:
            Hidden:
              - UNIT
              - []
        - EXTERNAL
        - []
    ");
}

// --- Type-level attributes on proxy types ---

#[test]
fn rename_on_proxy_type_with_opaque_field() {
    #[derive(Facet)]
    #[facet(rename = "WireUserId")]
    struct RenamedUserId(String);

    impl From<RenamedUserId> for WithoutFacet {
        fn from(value: RenamedUserId) -> Self {
            WithoutFacet(value.0)
        }
    }

    impl From<WithoutFacet> for RenamedUserId {
        fn from(value: WithoutFacet) -> Self {
            Self(value.0)
        }
    }

    impl From<&WithoutFacet> for RenamedUserId {
        fn from(value: &WithoutFacet) -> Self {
            Self(value.0.clone())
        }
    }

    #[derive(Facet)]
    struct Holder {
        #[facet(opaque, proxy = RenamedUserId)]
        user_id: WithoutFacet,
    }

    insta::assert_yaml_snapshot!(reflect!(Holder, RenamedUserId).unwrap());
}

#[test]
fn namespace_cleared_on_proxy_type() {
    use crate as fg;

    #[derive(Facet)]
    #[facet(fg::namespace)]
    struct RootUserId(String);

    impl From<RootUserId> for WithoutFacet {
        fn from(value: RootUserId) -> Self {
            WithoutFacet(value.0)
        }
    }

    impl From<WithoutFacet> for RootUserId {
        fn from(value: WithoutFacet) -> Self {
            Self(value.0)
        }
    }

    impl From<&WithoutFacet> for RootUserId {
        fn from(value: &WithoutFacet) -> Self {
            Self(value.0.clone())
        }
    }

    #[derive(Facet)]
    struct Holder {
        #[facet(opaque, proxy = RootUserId)]
        user_id: WithoutFacet,
    }

    insta::assert_yaml_snapshot!(reflect!(Holder, RootUserId).unwrap());
}

#[test]
fn nested_transparent_wrappers_opaque_proxy() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Inner(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[facet(transparent)]
    struct Outer(Inner);

    #[derive(Facet)]
    struct Parent {
        id: Outer,
    }

    insta::assert_yaml_snapshot!(reflect!(Parent, UserId).unwrap());
}

#[test]
fn enum_rename_all_struct_variant_with_opaque_proxy() {
    #[derive(Facet)]
    #[facet(rename_all = "camelCase")]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        SignIn {
            session_token: u64,
            #[facet(opaque, proxy = UserId)]
            owner_user_id: WithoutFacet,
        },
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn struct_with_bytes_and_opaque_proxy_fields() {
    use crate as fg;

    #[derive(Facet)]
    struct Mixed {
        #[facet(fg::bytes)]
        payload: Vec<u8>,
        #[facet(opaque, proxy = UserId)]
        user_id: WithoutFacet,
    }

    insta::assert_yaml_snapshot!(reflect!(Mixed, UserId).unwrap());
}

#[test]
fn enum_adjacent_tagging_with_opaque_proxy_struct_variant() {
    #[derive(Facet)]
    #[facet(tag = "type", content = "value")]
    #[repr(C)]
    #[allow(dead_code)]
    enum Message {
        SignIn {
            #[facet(opaque, proxy = UserId)]
            user_id: WithoutFacet,
        },
    }

    insta::assert_yaml_snapshot!(reflect!(Message, UserId).unwrap());
}

#[test]
fn enum_internal_tagging_newtype_opaque_proxy() {
    #[derive(Facet)]
    #[facet(tag = "type")]
    #[repr(C)]
    #[allow(dead_code)]
    enum Message {
        User(#[facet(opaque, proxy = UserId)] WithoutFacet),
    }

    insta::assert_yaml_snapshot!(reflect!(Message, UserId).unwrap());
}

#[test]
fn rename_all_override_on_renamed_opaque_proxy_field() {
    #[derive(Facet)]
    #[facet(rename_all = "camelCase")]
    struct Holder {
        #[facet(opaque, proxy = UserId, rename = "ownerId")]
        owner_user_id: WithoutFacet,
    }

    insta::assert_yaml_snapshot!(reflect!(Holder, UserId).unwrap());
}

#[test]
fn tuple_variant_skip_one_opaque_proxy_one_normal() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Pair(
            u32,
            #[facet(skip)] u64,
            #[facet(opaque, proxy = UserId)] WithoutFacet,
        ),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

// --- Enum: struct vs tuple vs newtype dispatch ---

#[test]
fn enum_all_variant_kinds_with_opaque_proxy() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Unit,
        Newtype(#[facet(opaque, proxy = UserId)] WithoutFacet),
        Struct {
            #[facet(opaque, proxy = UserId)]
            user_id: WithoutFacet,
        },
        Tuple(u32, #[facet(opaque, proxy = UserId)] WithoutFacet),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_newtype_single_field_opaque_proxy_dispatches_as_newtype() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        // Tuple field name "0" → newtype variant, not struct variant.
        Only(#[facet(opaque, proxy = UserId)] WithoutFacet),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_struct_single_named_field_opaque_proxy_dispatches_as_struct() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        // Named field → struct variant, not newtype.
        SignIn {
            #[facet(opaque, proxy = UserId)]
            user_id: WithoutFacet,
        },
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_tuple_multi_field_opaque_proxy_dispatches_as_tuple() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Pair(#[facet(opaque, proxy = UserId)] WithoutFacet, u32),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_struct_variant_transparent_wrapper_opaque_proxy_field() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        SignIn { wrapper: Wrapper },
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_tuple_variant_transparent_wrapper_opaque_proxy_field() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Pair(Wrapper, u32),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_struct_variant_tuple_field_and_opaque_proxy_field() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Data {
            meta: (u32, u32),
            #[facet(opaque, proxy = UserId)]
            user_id: WithoutFacet,
        },
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_tuple_variant_with_unit_field() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Mixed(u32, ()),
    }

    insta::assert_yaml_snapshot!(reflect!(Event).unwrap());
}

#[test]
fn enum_tuple_variant_option_field() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Maybe(u32, Option<u32>),
    }

    insta::assert_yaml_snapshot!(reflect!(Event).unwrap());
}

#[test]
fn enum_tuple_variant_vec_opaque_proxy() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Users(#[facet(opaque, proxy = UserIdList)] WithoutFacetList),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId, UserIdList).unwrap());
}

// --- Enum: rename / rename_all / namespace on type ---

#[test]
fn enum_type_rename_with_opaque_proxy_variants() {
    #[derive(Facet)]
    #[facet(rename = "WireEvent")]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        User(#[facet(opaque, proxy = UserId)] WithoutFacet),
        SignIn {
            #[facet(opaque, proxy = UserId)]
            user_id: WithoutFacet,
        },
        Pair(u32, #[facet(opaque, proxy = UserId)] WithoutFacet),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_namespace_on_type_with_opaque_proxy_variants() {
    use crate as fg;

    #[derive(Facet)]
    #[facet(fg::namespace = "events")]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        User(#[facet(opaque, proxy = UserId)] WithoutFacet),
        SignIn {
            #[facet(opaque, proxy = UserId)]
            user_id: WithoutFacet,
        },
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_rename_all_tuple_variant_opaque_proxy() {
    #[derive(Facet)]
    #[facet(rename_all = "camelCase")]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        SignInPair(u32, #[facet(opaque, proxy = UserId)] WithoutFacet),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_rename_all_newtype_variant_opaque_proxy() {
    #[derive(Facet)]
    #[facet(rename_all = "camelCase")]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        OwnerUser(#[facet(opaque, proxy = UserId)] WithoutFacet),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_rename_all_with_variant_rename_override_opaque_proxy() {
    #[derive(Facet)]
    #[facet(rename_all = "camelCase")]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        #[facet(rename = "USER")]
        OwnerUser(#[facet(opaque, proxy = UserId)] WithoutFacet),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn rename_all_snake_case_on_opaque_proxy_field() {
    #[derive(Facet)]
    #[facet(rename_all = "snake_case")]
    struct Holder {
        #[facet(opaque, proxy = UserId)]
        owner_user_id: WithoutFacet,
    }

    insta::assert_yaml_snapshot!(reflect!(Holder, UserId).unwrap());
}

// --- Enum tagging + variant shape ---

#[test]
fn enum_adjacent_tagging_tuple_variant_opaque_proxy() {
    #[derive(Facet)]
    #[facet(tag = "type", content = "value")]
    #[repr(C)]
    #[allow(dead_code)]
    enum Message {
        Pair(u32, #[facet(opaque, proxy = UserId)] WithoutFacet),
    }

    insta::assert_yaml_snapshot!(reflect!(Message, UserId).unwrap());
}

#[test]
fn enum_internal_tagging_struct_variant_opaque_proxy() {
    #[derive(Facet)]
    #[facet(tag = "type")]
    #[repr(C)]
    #[allow(dead_code)]
    enum Message {
        SignIn {
            #[facet(opaque, proxy = UserId)]
            user_id: WithoutFacet,
        },
    }

    insta::assert_yaml_snapshot!(reflect!(Message, UserId).unwrap());
}

// --- Enum skip paths ---

#[test]
fn enum_skip_newtype_variant_with_opaque_proxy() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Visible(u32),
        #[facet(skip)]
        Hidden(#[facet(opaque, proxy = UserId)] WithoutFacet),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_struct_variant_only_skipped_opaque_proxy_becomes_unit() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        SignIn {
            #[facet(skip)]
            #[facet(opaque, proxy = UserId)]
            user_id: WithoutFacet,
        },
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_tuple_variant_only_skipped_opaque_proxy_becomes_unit() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Pair(
            #[facet(skip)]
            #[facet(opaque, proxy = UserId)]
            WithoutFacet,
            u32,
        ),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

// --- Namespace on struct-variant proxy field ---

#[test]
fn enum_struct_variant_field_namespace_with_opaque_proxy() {
    use crate as fg;

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        SignIn {
            #[facet(fg::namespace = "wire")]
            #[facet(opaque, proxy = UserId)]
            user_id: WithoutFacet,
        },
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

// Special field handlers must preserve the namespace context of their caller.

#[test]
fn struct_option_transparent_proxy_respects_field_namespace() {
    use crate as fg;

    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    struct Holder {
        #[facet(fg::namespace = "wire")]
        user_id: Option<Wrapper>,
    }

    insta::assert_yaml_snapshot!(reflect!(Holder, UserId).unwrap());
}

#[test]
fn tuple_struct_option_transparent_proxy_respects_field_namespace() {
    use crate as fg;

    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    struct Holder(#[facet(fg::namespace = "wire")] Option<Wrapper>);

    insta::assert_yaml_snapshot!(reflect!(Holder, UserId).unwrap());
}

#[test]
fn enum_struct_option_transparent_proxy_respects_field_namespace() {
    use crate as fg;

    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        SignIn {
            #[facet(fg::namespace = "wire")]
            user_id: Option<Wrapper>,
        },
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_tuple_option_transparent_proxy_inherits_enum_namespace() {
    use crate as fg;

    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[facet(fg::namespace = "wire")]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        SignIn(Option<Wrapper>),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

// --- Transparent bytes wrapper alongside opaque proxy (reflection only) ---

#[test]
fn struct_transparent_bytes_wrapper_and_opaque_proxy() {
    use crate as fg;

    #[derive(Facet)]
    #[facet(transparent)]
    struct Payload(#[facet(fg::bytes)] Vec<u8>);

    #[derive(Facet)]
    struct Mixed {
        payload: Payload,
        #[facet(opaque, proxy = UserId)]
        user_id: WithoutFacet,
    }

    insta::assert_yaml_snapshot!(reflect!(Mixed, UserId).unwrap());
}

// --- Regular (non-opaque) field using proxy newtype ---

#[test]
fn struct_field_optional_proxy_newtype() {
    #[derive(Facet)]
    struct Holder {
        user_id: OptionalUserId,
    }

    insta::assert_yaml_snapshot!(reflect!(Holder, UserId, OptionalUserId).unwrap());
}

// --- Deep nesting, composition, and cross-attribute stacks ---

#[test]
fn nested_transparent_three_layers_opaque_proxy() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Inner(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[facet(transparent)]
    struct Middle(Inner);

    #[derive(Facet)]
    #[facet(transparent)]
    struct Outer(Middle);

    #[derive(Facet)]
    struct Parent {
        id: Outer,
    }

    insta::assert_yaml_snapshot!(reflect!(Parent, UserId).unwrap());
}

#[test]
fn enum_newtype_variant_transparent_wrapper_opaque_proxy() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        User(Wrapper),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_tuple_variant_two_opaque_proxy_fields() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Pair(
            #[facet(opaque, proxy = UserId)] WithoutFacet,
            #[facet(opaque, proxy = OrgId)] WithoutFacet,
        ),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId, OrgId).unwrap());
}

#[test]
fn enum_struct_variant_option_transparent_wrapper_opaque_proxy() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        SignIn { user_id: Option<Wrapper> },
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn box_transparent_wrapper_opaque_proxy_field() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    struct Holder {
        user_id: Box<Wrapper>,
    }

    insta::assert_yaml_snapshot!(reflect!(Holder, UserId).unwrap());
}

#[test]
fn option_box_transparent_wrapper_opaque_proxy() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    struct Holder {
        user_id: Option<Box<Wrapper>>,
    }

    insta::assert_yaml_snapshot!(reflect!(Holder, UserId).unwrap());
}

#[test]
fn enum_adjacent_tagging_transparent_wrapper_tuple_variant() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[facet(tag = "type", content = "value")]
    #[repr(C)]
    #[allow(dead_code)]
    enum Message {
        Pair(Wrapper, u32),
    }

    insta::assert_yaml_snapshot!(reflect!(Message, UserId).unwrap());
}

#[test]
fn enum_internal_tagging_transparent_wrapper_newtype_variant() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[facet(tag = "type")]
    #[repr(C)]
    #[allow(dead_code)]
    enum Message {
        User(Wrapper),
    }

    insta::assert_yaml_snapshot!(reflect!(Message, UserId).unwrap());
}

#[test]
fn enum_tuple_variant_unit_and_opaque_proxy() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Mixed((), #[facet(opaque, proxy = UserId)] WithoutFacet, u32),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_tuple_variant_option_transparent_wrapper() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Maybe(Option<Wrapper>, u32),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn namespace_cleared_on_opaque_proxy_field() {
    use crate as fg;

    #[derive(Facet)]
    struct Holder {
        #[facet(fg::namespace)]
        #[facet(opaque, proxy = UserId)]
        user_id: WithoutFacet,
    }

    insta::assert_yaml_snapshot!(reflect!(Holder, UserId).unwrap());
}

#[test]
fn enum_rename_all_kebab_case_opaque_proxy() {
    #[derive(Facet)]
    #[facet(rename_all = "kebab-case")]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        SignIn {
            session_token: u64,
            #[facet(opaque, proxy = UserId)]
            owner_user_id: WithoutFacet,
        },
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_struct_variant_nested_newtype_opaque_proxy_field() {
    #[derive(Facet)]
    struct Inner(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        SignIn { inner: Inner },
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_newtype_variant_nested_newtype_opaque_proxy() {
    #[derive(Facet)]
    struct Inner(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        User(Inner),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn struct_field_nested_enum_with_opaque_proxy_variant() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum InnerEvent {
        User(#[facet(opaque, proxy = UserId)] WithoutFacet),
    }

    #[derive(Facet)]
    struct Holder {
        event: InnerEvent,
    }

    insta::assert_yaml_snapshot!(reflect!(Holder, UserId).unwrap());
}

#[test]
fn enum_variant_holding_enum_with_opaque_proxy() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum InnerEvent {
        User(#[facet(opaque, proxy = UserId)] WithoutFacet),
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum OuterEvent {
        Wrapped(InnerEvent),
    }

    insta::assert_yaml_snapshot!(reflect!(OuterEvent, UserId).unwrap());
}

#[test]
fn skip_on_transparent_wrapper_field() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    struct Holder {
        label: String,
        #[facet(skip)]
        user_id: Wrapper,
    }

    insta::assert_yaml_snapshot!(reflect!(Holder, UserId).unwrap());
}

#[test]
fn enum_struct_variant_bytes_transparent_and_opaque_proxy() {
    use crate as fg;

    #[derive(Facet)]
    #[facet(transparent)]
    struct Payload(#[facet(fg::bytes)] Vec<u8>);

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Data {
            payload: Payload,
            #[facet(opaque, proxy = UserId)]
            user_id: WithoutFacet,
        },
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn vec_nested_transparent_wrappers_opaque_proxy() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Inner(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[facet(transparent)]
    struct Outer(Inner);

    #[derive(Facet)]
    struct Holder {
        ids: Vec<Outer>,
    }

    insta::assert_yaml_snapshot!(reflect!(Holder, UserId).unwrap());
}

#[test]
fn tuple_struct_nested_tuple_struct_opaque_proxy() {
    #[derive(Facet)]
    struct Inner(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    struct Outer(Inner, u32);

    insta::assert_yaml_snapshot!(reflect!(Outer, UserId).unwrap());
}

#[test]
fn enum_skip_tuple_variant_with_opaque_proxy() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Visible(u32),
        #[facet(skip)]
        Hidden(#[facet(opaque, proxy = UserId)] WithoutFacet, u32),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn struct_mixed_transparent_bytes_wrapper_opaque_proxy_and_rust_tuple() {
    use crate as fg;

    #[derive(Facet)]
    #[facet(transparent)]
    struct Payload(#[facet(fg::bytes)] Vec<u8>);

    #[derive(Facet)]
    struct Mixed {
        meta: (u32, u32),
        payload: Payload,
        #[facet(opaque, proxy = UserId)]
        user_id: WithoutFacet,
    }

    insta::assert_yaml_snapshot!(reflect!(Mixed, UserId).unwrap());
}

#[test]
fn enum_struct_variant_rust_tuple_and_nested_transparent_opaque_proxy() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Data {
            meta: (u32, Option<u32>),
            wrapper: Wrapper,
        },
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_adjacent_tagging_all_variant_kinds_opaque_proxy() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[facet(tag = "type", content = "value")]
    #[repr(C)]
    #[allow(dead_code)]
    enum Message {
        Unit,
        User(Wrapper),
        SignIn {
            #[facet(opaque, proxy = UserId)]
            user_id: WithoutFacet,
        },
        Pair(u32, #[facet(opaque, proxy = UserId)] WithoutFacet),
    }

    insta::assert_yaml_snapshot!(reflect!(Message, UserId).unwrap());
}

#[test]
fn struct_field_hash_map_with_transparent_wrapper_values() {
    use std::collections::HashMap;

    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    struct Holder {
        users_by_label: HashMap<String, Wrapper>,
    }

    insta::assert_yaml_snapshot!(reflect!(Holder, UserId).unwrap());
}

// --- Maximum-depth paths: wrappers through every container kind ---

#[test]
fn enum_newtype_variant_option_transparent_wrapper_opaque_proxy() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        MaybeUser(Option<Wrapper>),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn nested_option_transparent_wrapper_opaque_proxy() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    struct Holder {
        user_id: Option<Option<Wrapper>>,
    }

    insta::assert_yaml_snapshot!(reflect!(Holder, UserId).unwrap());
}

#[test]
fn vec_option_transparent_wrapper_opaque_proxy() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    struct Holder {
        maybe_users: Vec<Option<Wrapper>>,
    }

    insta::assert_yaml_snapshot!(reflect!(Holder, UserId).unwrap());
}

#[test]
fn arc_transparent_wrapper_opaque_proxy() {
    use std::sync::Arc;

    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    struct Holder {
        user_id: Arc<Wrapper>,
    }

    insta::assert_yaml_snapshot!(reflect!(Holder, UserId).unwrap());
}

#[test]
fn reference_transparent_wrapper_opaque_proxy() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    struct Holder<'a> {
        user_id: &'a Wrapper,
    }

    insta::assert_yaml_snapshot!(reflect!(Holder, UserId).unwrap());
}

#[test]
fn array_transparent_wrapper_opaque_proxy() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    struct Holder {
        ids: [Wrapper; 2],
    }

    insta::assert_yaml_snapshot!(reflect!(Holder, UserId).unwrap());
}

#[test]
fn box_nested_transparent_three_layers_opaque_proxy() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Inner(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[facet(transparent)]
    struct Middle(Inner);

    #[derive(Facet)]
    #[facet(transparent)]
    struct Outer(Middle);

    #[derive(Facet)]
    struct Holder {
        user_id: Box<Outer>,
    }

    insta::assert_yaml_snapshot!(reflect!(Holder, UserId).unwrap());
}

#[test]
fn enum_struct_variant_single_transparent_wrapper_field() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        SignIn { user_id: Wrapper },
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_tuple_variant_single_transparent_wrapper_field() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Only(Wrapper),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_tuple_variant_two_transparent_wrapper_fields() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Pair(Wrapper, Wrapper),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_struct_variant_two_transparent_proxy_fields() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct UserWrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[facet(transparent)]
    struct OrgWrapper(#[facet(opaque, proxy = OrgId)] WithoutFacet);

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Pair {
            user_id: UserWrapper,
            org_id: OrgWrapper,
        },
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId, OrgId).unwrap());
}

#[test]
fn tree_enum_with_opaque_proxy_in_struct_variant() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Tree {
        Leaf {
            #[facet(opaque, proxy = UserId)]
            owner_id: WithoutFacet,
        },
        Branch {
            children: Vec<Tree>,
        },
    }

    insta::assert_yaml_snapshot!(reflect!(Tree, UserId).unwrap());
}

#[test]
fn enum_wrapped_nested_enum_with_opaque_proxy() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum InnerEvent {
        User(#[facet(opaque, proxy = UserId)] WithoutFacet),
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum OuterEvent {
        Wrapped(InnerEvent),
        Direct { inner: InnerEvent },
    }

    insta::assert_yaml_snapshot!(reflect!(OuterEvent, UserId).unwrap());
}

#[test]
fn enum_tuple_variant_rust_tuple_transparent_wrapper_and_proxy() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Triple(
            (u32, Option<u32>),
            Wrapper,
            #[facet(opaque, proxy = UserId)] WithoutFacet,
        ),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn skip_on_transparent_wrapper_in_enum_struct_variant() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        SignIn {
            label: String,
            #[facet(skip)]
            user_id: Wrapper,
        },
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_adjacent_tagging_rename_all_kebab_transparent_and_proxy() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[facet(tag = "type", content = "value")]
    #[facet(rename_all = "kebab-case")]
    #[repr(C)]
    #[allow(dead_code)]
    enum Message {
        SignIn {
            session_token: u64,
            owner_user_id: Wrapper,
            #[facet(opaque, proxy = UserId)]
            actor_id: WithoutFacet,
        },
    }

    insta::assert_yaml_snapshot!(reflect!(Message, UserId).unwrap());
}

#[test]
fn transparent_wrapper_over_optional_proxy_newtype() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(OptionalUserId);

    #[derive(Facet)]
    struct Holder {
        user_id: Wrapper,
    }

    insta::assert_yaml_snapshot!(reflect!(Holder, UserId, OptionalUserId).unwrap());
}

#[test]
fn struct_field_reference_option_transparent_wrapper() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    struct Holder<'a> {
        user_id: &'a Option<Wrapper>,
    }

    insta::assert_yaml_snapshot!(reflect!(Holder, UserId).unwrap());
}

#[test]
fn enum_struct_variant_hash_map_and_option_box_transparent() {
    use std::collections::HashMap;

    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Data {
            users: HashMap<String, Wrapper>,
            backup: Option<Box<Wrapper>>,
        },
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn enum_internal_tagging_tuple_variant_transparent_and_proxy() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[facet(tag = "type")]
    #[repr(C)]
    #[allow(dead_code)]
    enum Message {
        Pair(Wrapper, #[facet(opaque, proxy = UserId)] WithoutFacet),
    }

    insta::assert_yaml_snapshot!(reflect!(Message, UserId).unwrap());
}

#[test]
fn vec_box_nested_transparent_opaque_proxy() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Inner(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[facet(transparent)]
    struct Outer(Inner);

    #[derive(Facet)]
    #[allow(clippy::vec_box)]
    struct Holder {
        ids: Vec<Box<Outer>>,
    }

    insta::assert_yaml_snapshot!(reflect!(Holder, UserId).unwrap());
}

#[test]
fn enum_newtype_variant_nested_transparent_three_layers() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Inner(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    #[facet(transparent)]
    struct Middle(Inner);

    #[derive(Facet)]
    #[facet(transparent)]
    struct Outer(Middle);

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        User(Outer),
    }

    insta::assert_yaml_snapshot!(reflect!(Event, UserId).unwrap());
}

#[test]
fn struct_mixed_every_container_kind_with_transparent_proxy() {
    use std::collections::HashMap;
    use std::sync::Arc;

    #[derive(Facet)]
    #[facet(transparent)]
    struct Wrapper(#[facet(opaque, proxy = UserId)] WithoutFacet);

    #[derive(Facet)]
    struct KitchenSink {
        plain: Wrapper,
        optional: Option<Wrapper>,
        boxed: Box<Wrapper>,
        shared: Arc<Wrapper>,
        slice: [Wrapper; 1],
        list: Vec<Wrapper>,
        maybe_list: Vec<Option<Wrapper>>,
        nested_map: HashMap<String, Option<Box<Wrapper>>>,
        map: HashMap<String, Wrapper>,
    }

    insta::assert_yaml_snapshot!(reflect!(KitchenSink, UserId).unwrap());
}

// --- Negative / limitation cases ---

#[test]
fn struct_only_opaque_option_without_proxy_becomes_unit() {
    #[derive(Facet)]
    struct OnlyOpaque {
        #[facet(opaque)]
        hidden: WithoutFacetOption,
    }

    insta::assert_yaml_snapshot!(reflect!(OnlyOpaque).unwrap(), @"
    ? namespace: ROOT
      name: OnlyOpaque
    : UNITSTRUCT: []
    ");
}

#[test]
fn enum_newtype_variant_option_opaque_without_proxy() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Event {
        Maybe(#[facet(opaque)] WithoutFacetOption),
    }

    insta::assert_yaml_snapshot!(reflect!(Event).unwrap(), @"
    ? namespace: ROOT
      name: Event
    : ENUM:
        - 0:
            Maybe:
              - UNIT
              - []
        - EXTERNAL
        - []
    ");
}
