use http::{header, Method, Request, Response};
use http_cache_semantics::CachePolicy;
use std::time::Duration;
use std::time::SystemTime;

use crate::private_opts;
use crate::request_parts;
use crate::response_parts;

#[test]
fn when_urls_match() {
    let now = SystemTime::now();
    let response = &response_parts(
        Response::builder()
            .status(200)
            .header(header::CACHE_CONTROL, "max-age=2"),
    );

    let policy = CachePolicy::new(&request_parts(Request::builder()), response);

    assert!(policy
        .before_request(&mut request_parts(Request::builder()), now)
        .is_fresh());
}

#[test]
fn when_expires_is_present() {
    let now = SystemTime::now();
    let two_seconds_later = httpdate::fmt_http_date(now + Duration::from_secs(2));
    let response = &response_parts(
        Response::builder()
            .status(302)
            .header(header::EXPIRES, two_seconds_later),
    );

    let policy = CachePolicy::new(&request_parts(Request::builder()), response);

    assert!(policy
        .before_request(&mut request_parts(Request::builder()), now)
        .is_fresh());
}

#[test]
fn when_methods_match() {
    let now = SystemTime::now();
    let response = &response_parts(Response::builder().header(header::CACHE_CONTROL, "max-age=2"));
    let policy = CachePolicy::new(&request_parts(Request::builder()), response);

    assert!(policy
        .before_request(&request_parts(Request::builder()), now)
        .is_fresh());
}

#[test]
fn must_revalidate_allows_not_revalidating_fresh() {
    let now = SystemTime::now();
    let response = &response_parts(
        Response::builder()
            .status(200)
            .header(header::CACHE_CONTROL, "max-age=200, must-revalidate"),
    );
    let policy = CachePolicy::new(
        &request_parts(Request::builder().method(Method::GET)),
        response,
    );

    assert!(policy
        .before_request(&request_parts(Request::builder().method(Method::GET)), now)
        .is_fresh());

    let later = now + std::time::Duration::from_secs(300);
    assert!(!policy
        .before_request(
            &request_parts(Request::builder().method(Method::GET)),
            later
        )
        .is_fresh());
}

#[test]
fn must_revalidate_disallows_stale() {
    let now = SystemTime::now();
    let response = &response_parts(
        Response::builder()
            .status(200)
            .header(header::CACHE_CONTROL, "max-age=200, must-revalidate"),
    );
    let policy = CachePolicy::new(
        &request_parts(Request::builder().method(Method::GET)),
        response,
    );

    let later = now + std::time::Duration::from_secs(300);
    assert!(!policy
        .before_request(
            &request_parts(Request::builder().method(Method::GET)),
            later
        )
        .is_fresh());

    let later = now + std::time::Duration::from_secs(300);
    assert!(!policy
        .before_request(
            &request_parts(
                Request::builder()
                    .header("cache-control", "max-stale")
                    .method(Method::GET)
            ),
            later
        )
        .is_fresh());
}

#[test]
fn not_when_hosts_mismatch() {
    let now = SystemTime::now();
    let response = &response_parts(
        Response::builder()
            .status(200)
            .header(header::CACHE_CONTROL, "max-age=2"),
    );
    let policy = CachePolicy::new(
        &request_parts(Request::builder().header(header::HOST, "foo")),
        response,
    );

    assert!(policy
        .before_request(
            &request_parts(Request::builder().header(header::HOST, "foo")),
            now
        )
        .is_fresh());

    assert!(!policy
        .before_request(
            &request_parts(Request::builder().header(header::HOST, "foofoo")),
            now
        )
        .is_fresh());
}

#[test]
fn when_methods_match_head() {
    let now = SystemTime::now();
    let response = &response_parts(Response::builder().header(header::CACHE_CONTROL, "max-age=2"));
    let policy = CachePolicy::new(
        &request_parts(Request::builder().method(Method::HEAD)),
        response,
    );

    assert!(policy
        .before_request(&request_parts(Request::builder().method(Method::HEAD)), now)
        .is_fresh());
}

#[test]
fn not_when_proxy_revalidating() {
    let now = SystemTime::now();
    let response = &response_parts(
        Response::builder().header(header::CACHE_CONTROL, "max-age=2, proxy-revalidate "),
    );
    let policy = CachePolicy::new(&request_parts(Request::builder()), response);

    assert!(!policy
        .before_request(&request_parts(Request::builder()), now)
        .is_fresh());
}

#[test]
fn when_not_a_proxy_revalidating() {
    let now = SystemTime::now();
    let response = &response_parts(
        Response::builder().header(header::CACHE_CONTROL, "max-age=2, proxy-revalidate "),
    );
    let policy = CachePolicy::new_options(
        &request_parts(Request::builder()),
        response,
        now,
        private_opts(),
    );

    assert!(policy
        .before_request(&request_parts(Request::builder()), now)
        .is_fresh());
}

#[test]
fn not_when_no_cache_requesting() {
    let now = SystemTime::now();
    let response = &response_parts(Response::builder().header(header::CACHE_CONTROL, "max-age=2"));
    let policy = CachePolicy::new(&request_parts(Request::builder()), response);

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
