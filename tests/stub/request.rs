use std::time::SystemTime;

use http::{header, Method, Request, Response};

use crate::harness;
use crate::private_opts;
use crate::req_cache_control;
use crate::request_parts;
use crate::resp_cache_control;
use crate::response_parts;

fn public_cacheable_response() -> http::response::Parts {
    response_parts(Response::builder().header(header::CACHE_CONTROL, "public, max-age=222"))
}

fn cacheable_response() -> http::response::Parts {
    response_parts(Response::builder().header(header::CACHE_CONTROL, "max-age=111"))
}

#[test]
fn no_store_kills_cache() {
    harness()
        .no_store()
        .request(req_cache_control("no-store"))
        .test_with_response(public_cacheable_response());
}

#[test]
fn post_not_cacheable_by_default() {
    harness()
        .no_store()
        .request(request_parts(Request::builder().method(Method::POST)))
        .test_with_cache_control("public");
}

#[test]
fn post_cacheable_explicitly() {
    harness()
        .request(request_parts(Request::builder().method(Method::POST)))
        .test_with_response(public_cacheable_response());
}

#[test]
fn public_cacheable_auth_is_ok() {
    harness()
        .request(request_parts(
            Request::builder().header(header::AUTHORIZATION, "test"),
        ))
        .test_with_response(public_cacheable_response());
}

#[test]
fn private_auth_is_ok() {
    harness()
        .options(private_opts())
        .request(request_parts(
            Request::builder().header(header::AUTHORIZATION, "test"),
        ))
        .test_with_response(cacheable_response());
}

#[test]
fn revalidate_auth_is_ok() {
    harness()
        .request(request_parts(
            Request::builder().header(header::AUTHORIZATION, "test"),
        ))
        .test_with_cache_control("max-age=80, must-revalidate");
}

#[test]
fn auth_prevents_caching_by_default() {
    harness()
        .no_store()
        .request(request_parts(
            Request::builder().header(header::AUTHORIZATION, "test"),
        ))
        .test_with_response(cacheable_response());
}

#[test]
fn no_cache_bypasses_cache() {
    let now = SystemTime::now();
    let policy = harness().time(now).test_with_response(cacheable_response());
    // an innocuous cache-control directive is still fresh...
    assert!(policy
        .before_request(&req_cache_control("no-transform"), now)
        .is_fresh());
    // ...while `no-cache` is not
    assert!(!policy
        .before_request(&req_cache_control("no-cache"), now)
        .is_fresh());

    // And again with an immutable response
    let policy = harness()
        .time(now)
        .test_with_response(resp_cache_control("immutable, max-age=3600"));
    assert!(policy
        .before_request(&req_cache_control("no-transform"), now)
        .is_fresh());
    assert!(!policy
        .before_request(&req_cache_control("no-cache"), now)
        .is_fresh());
}
