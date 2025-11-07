use http::{header, Method, Request, Response};
use std::time::Duration;
use std::time::SystemTime;

use crate::harness;
use crate::private_config;
use crate::req_cache_control;
use crate::request_parts;
use crate::response_parts;

#[test]
fn when_expires_is_present() {
    let now = SystemTime::now();
    let two_seconds_later = httpdate::fmt_http_date(now + Duration::from_secs(2));
    let response = response_parts(
        Response::builder()
            .status(302)
            .header(header::EXPIRES, two_seconds_later),
    );
    harness().time(now).test_with_response(response);
}

#[test]
fn must_revalidate_allows_not_revalidating_fresh() {
    let now = SystemTime::now();
    let policy = harness()
        .time(now)
        .test_with_cache_control("max-age=200, must-revalidate");

    let later = now + Duration::from_secs(300);
    assert!(!policy
        .before_request(&request_parts(Request::builder()), later)
        .is_fresh());
}

#[test]
fn must_revalidate_disallows_stale() {
    let now = SystemTime::now();
    let policy = harness()
        .time(now)
        .test_with_cache_control("max-age=200, must-revalidate");

    let later = now + Duration::from_secs(300);
    assert!(!policy
        .before_request(&request_parts(Request::builder()), later)
        .is_fresh());

    let later = now + Duration::from_secs(300);
    assert!(!policy
        .before_request(&req_cache_control("max-stale"), later)
        .is_fresh());
}

#[test]
fn not_when_hosts_mismatch() {
    let now = SystemTime::now();
    let request = request_parts(Request::builder().header(header::HOST, "foo"));
    let policy = harness()
        .time(now)
        .request(request)
        .test_with_cache_control("max-age=2");
    assert!(!policy
        .before_request(
            &request_parts(Request::builder().header(header::HOST, "foofoo")),
            now
        )
        .is_fresh());
}

#[test]
fn when_methods_match_head() {
    harness()
        .request(request_parts(Request::builder().method(Method::HEAD)))
        .test_with_cache_control("max-age=2");
}

#[test]
fn not_when_proxy_revalidating() {
    harness()
        .stale_and_store()
        .test_with_cache_control("max-age=2, proxy-revalidate");
}

#[test]
fn when_not_a_proxy_revalidating() {
    harness()
        .config(private_config())
        .test_with_cache_control("max-age=2, proxy-revalidate");
}

#[test]
fn not_when_no_cache_requesting() {
    let now = SystemTime::now();
    let policy = harness().time(now).test_with_cache_control("max-age=2");

    assert!(policy
        .before_request(
            &request_parts(Request::builder().header(header::CACHE_CONTROL, "fine")),
            now
        )
        .is_fresh());

    assert!(!policy
        .before_request(
            &request_parts(Request::builder().header(header::CACHE_CONTROL, "no-cache")),
            now
        )
        .is_fresh());

    assert!(!policy
        .before_request(
            &request_parts(Request::builder().header(header::PRAGMA, "no-cache")),
            now
        )
        .is_fresh());
}
