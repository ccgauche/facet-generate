use std::mem::MaybeUninit;

use facet::{Facet, PtrConst, PtrUninit, Type, UserType};

use super::opaque_proxy_fixtures::{UserId, WithoutFacet};

#[derive(Facet)]
struct WithOpaqueProxy {
    #[facet(opaque, proxy = UserId)]
    user_id: WithoutFacet,
}

fn user_id_field_proxy() -> &'static facet::ProxyDef {
    let shape = WithOpaqueProxy::SHAPE;
    let Type::User(UserType::Struct(struct_type)) = &shape.ty else {
        panic!("expected struct shape");
    };
    struct_type.fields[0]
        .proxy()
        .expect("opaque proxy field should have a proxy definition")
}

#[test]
fn proxy_convert_out_succeeds() {
    let proxy_def = user_id_field_proxy();

    let view = WithOpaqueProxy {
        user_id: WithoutFacet("user-42".to_string()),
    };
    // OpaqueBorrow is repr(transparent). Same address as the field.
    let field_ptr = PtrConst::new_sized(std::ptr::addr_of!(view.user_id));

    let mut proxy_storage = MaybeUninit::<UserId>::uninit();
    let proxy_ptr = PtrUninit::from_maybe_uninit(&mut proxy_storage);

    let proxy_result = unsafe { (proxy_def.convert_out)(field_ptr, proxy_ptr) };
    match proxy_result {
        Ok(_) => {}
        Err(err) => panic!("convert_out should succeed, got: {err}"),
    }
    let proxy = unsafe { proxy_storage.assume_init() };
    assert_eq!(proxy.0, "user-42");
}

#[test]
fn proxy_convert_in_succeeds() {
    let proxy_def = user_id_field_proxy();
    let proxy = UserId("user-42".to_string());
    let proxy_const = PtrConst::new_sized(&proxy as *const UserId);
    let mut target_storage = MaybeUninit::<WithoutFacet>::uninit();
    let target_ptr = PtrUninit::from_maybe_uninit(&mut target_storage);

    let target_result = unsafe { (proxy_def.convert_in)(proxy_const, target_ptr) };
    match target_result {
        Ok(_) => {}
        Err(err) => panic!("convert_in should succeed, got: {err}"),
    }
    // convert_in moves out of the proxy.
    std::mem::forget(proxy);
    let target = unsafe { target_storage.assume_init() };
    assert_eq!(target.0, "user-42");
}

#[derive(Facet)]
#[facet(transparent)]
struct TransparentOpaqueProxy(#[facet(opaque, proxy = UserId)] WithoutFacet);

#[test]
fn nested_transparent_proxy_convert_out_succeeds() {
    let shape = TransparentOpaqueProxy::SHAPE;
    let Type::User(UserType::Struct(struct_type)) = &shape.ty else {
        panic!("expected struct shape");
    };
    let proxy_def = struct_type.fields[0]
        .proxy()
        .expect("nested opaque proxy field should have a proxy definition");

    let view = TransparentOpaqueProxy(WithoutFacet("nested-user".to_string()));
    let field_ptr = PtrConst::new_sized(std::ptr::addr_of!(view.0));
    let mut proxy_storage = MaybeUninit::<UserId>::uninit();
    let proxy_ptr = PtrUninit::from_maybe_uninit(&mut proxy_storage);

    unsafe { (proxy_def.convert_out)(field_ptr, proxy_ptr) }
        .expect("nested proxy convert_out should succeed");
    let proxy = unsafe { proxy_storage.assume_init() };
    assert_eq!(proxy.0, "nested-user");
}

// Wire errors fail during proxy deserialization. Before convert_in runs.

struct BadTarget(String);

#[derive(Facet)]
struct BadProxy(String);

impl TryFrom<BadProxy> for BadTarget {
    type Error = &'static str;

    fn try_from(value: BadProxy) -> Result<Self, Self::Error> {
        if value.0.is_empty() {
            Err("empty id is invalid")
        } else {
            Ok(Self(value.0))
        }
    }
}

impl TryFrom<&BadTarget> for BadProxy {
    type Error = &'static str;

    fn try_from(value: &BadTarget) -> Result<Self, Self::Error> {
        if value.0.is_empty() {
            Err("empty id is invalid")
        } else {
            Ok(BadProxy(value.0.clone()))
        }
    }
}

#[derive(Facet)]
struct WithFailingProxy {
    #[facet(opaque, proxy = BadProxy)]
    user_id: BadTarget,
}

#[test]
fn proxy_convert_surfaces_try_from_errors() {
    let shape = WithFailingProxy::SHAPE;
    let Type::User(UserType::Struct(struct_type)) = &shape.ty else {
        panic!("expected struct shape");
    };
    let proxy_def = struct_type.fields[0].proxy().expect("proxy should be set");

    // convert_out fails when TryFrom rejects the value.
    let bad_view = WithFailingProxy {
        user_id: BadTarget(String::new()),
    };
    let field_ptr = PtrConst::new_sized(std::ptr::addr_of!(bad_view.user_id));
    let mut proxy_storage = MaybeUninit::<BadProxy>::uninit();
    let proxy_ptr = PtrUninit::from_maybe_uninit(&mut proxy_storage);

    let out_err = unsafe { (proxy_def.convert_out)(field_ptr, proxy_ptr) }
        .expect_err("convert_out should fail for empty id");
    assert!(
        out_err.contains("empty id is invalid"),
        "unexpected error: {out_err}"
    );

    // convert_in fails when TryFrom rejects the value.
    let bad_proxy = BadProxy(String::new());
    let proxy_const = PtrConst::new_sized(&bad_proxy as *const BadProxy);
    let mut target_storage = MaybeUninit::<BadTarget>::uninit();
    let target_ptr = PtrUninit::from_maybe_uninit(&mut target_storage);

    let in_err = unsafe { (proxy_def.convert_in)(proxy_const, target_ptr) }
        .expect_err("convert_in should fail for empty id");
    std::mem::forget(bad_proxy);
    assert!(
        in_err.contains("empty id is invalid"),
        "unexpected error: {in_err}"
    );
}
