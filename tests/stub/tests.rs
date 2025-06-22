//! Determines whether a given HTTP response can be cached and whether a
//! cached response can be reused, following the rules specified in [RFC
//! 7234](https://httpwg.org/specs/rfc7234.html).

use http::header;
use http::Method;
use http::Request;
use http_cache_policy::*;
use std::time::SystemTime;

use crate::request_parts;
use crate::resp_cache_control;

#[test]
fn proxy_cacheable_auth_is_ok() {
    let now = SystemTime::now();
    let policy = CachePolicy::new(
        &request_parts(Request::builder().header(header::AUTHORIZATION, "test")),
        &resp_cache_control("max-age=0,s-maxage=12"),
    );

    assert!(!policy.is_stale(now));
    assert!(policy.is_storable());

    #[cfg(feature = "serde")]
    {
        let json = serde_json::to_string(&policy).unwrap();
        let policy: CachePolicy = serde_json::from_str(&json).unwrap();

        assert!(!policy.is_stale(now));
        assert!(policy.is_storable());
    }
}

#[test]
fn not_when_urls_mismatch() {
    let now = SystemTime::now();
    let policy = CachePolicy::new(
        &request_parts(Request::builder().uri("/foo")),
        &resp_cache_control("max-age=2"),
    );

    assert!(!policy
        .before_request(&request_parts(Request::builder().uri("/foo?bar")), now)
        .is_fresh());
}

#[test]
fn not_when_methods_mismatch() {
    let now = SystemTime::now();
    let policy = CachePolicy::new(
        &request_parts(Request::builder().method(Method::POST)),
        &resp_cache_control("max-age=2"),
    );

    assert!(!policy.before_request(&Request::new(()), now).is_fresh());
}

#[test]
fn not_when_methods_mismatch_head() {
    let now = SystemTime::now();
    let policy = CachePolicy::new(
        &request_parts(Request::builder().method(Method::HEAD)),
        &resp_cache_control("max-age=2"),
    );

    assert!(!policy
        .before_request(&request_parts(Request::builder()), now)
        .is_fresh());
}
