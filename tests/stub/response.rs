use http::{header, Method, Request, Response, StatusCode};
use http_cache_policy::{CachePolicy, Config};
use std::time::{Duration, SystemTime};

use crate::{harness, private_config, req_cache_control, request_parts, response_parts};

fn now_rfc2822() -> String {
    httpdate::fmt_http_date(SystemTime::now())
}

#[test]
fn simple_miss() {
    harness()
        .stale_and_store()
        .test_with_response(response_parts(Response::builder()));
}

#[test]
fn simple_hit() {
    harness()
        .assert_time_to_live(999999)
        .test_with_cache_control("public, max-age=999999");
}

#[test]
fn quoted_syntax() {
    harness()
        .assert_time_to_live(678)
        .test_with_cache_control("  max-age = \"678\"      ");
}

#[test]
fn iis() {
    harness()
        .assert_time_to_live(259200)
        .config(private_config())
        .test_with_cache_control("private, public, max-age=259200");
}

#[test]
fn pre_check_tolerated() {
    let now = SystemTime::now();
    let cache_control = "pre-check=0, post-check=0, no-store, no-cache, max-age=100";
    let policy = harness()
        .no_store()
        .time(now)
        .test_with_cache_control(cache_control);

    assert_eq!(
        get_cached_response(&policy, &req_cache_control("max-stale"), now).headers
            [header::CACHE_CONTROL],
        cache_control
    );
}

#[test]
fn pre_check_poison() {
    let now = SystemTime::now();
    let original_cache_control =
        "pre-check=0, post-check=0, no-cache, no-store, max-age=100, custom, foo=bar";
    let response = response_parts(
        Response::builder()
            .header(header::CACHE_CONTROL, original_cache_control)
            .header(header::PRAGMA, "no-cache"),
    );

    let policy = harness()
        .assert_time_to_live(100)
        .time(now)
        .config(Config {
            ignore_cargo_cult: true,
            ..Default::default()
        })
        .test_with_response(response);

    let res = get_cached_response(&policy, &request_parts(Request::builder()), now);
    let cache_control_header = &res.headers[header::CACHE_CONTROL].to_str().unwrap();
    assert!(!cache_control_header.contains("pre-check"));
    assert!(!cache_control_header.contains("post-check"));
    assert!(!cache_control_header.contains("no-store"));

    assert!(cache_control_header.contains("max-age=100"));
    assert!(cache_control_header.contains("custom"));
    assert!(cache_control_header.contains("foo=bar"));

    assert!(!res.headers.contains_key(header::PRAGMA));
}

#[test]
fn age_can_make_stale() {
    let response = response_parts(
        Response::builder()
            .header(header::CACHE_CONTROL, "max-age=100")
            .header(header::AGE, 101),
    );
    harness().stale_and_store().test_with_response(response);
}

#[test]
fn age_not_always_stale() {
    let response = response_parts(
        Response::builder()
            .header(header::CACHE_CONTROL, "max-age=20")
            .header(header::AGE, 15),
    );
    harness().test_with_response(response);
}

#[test]
fn bogus_age_ignored() {
    let response = response_parts(
        Response::builder()
            .header(header::CACHE_CONTROL, "max-age=20")
            .header(header::AGE, "golden"),
    );
    harness().test_with_response(response);
}

#[test]
fn cache_old_files() {
    let now = SystemTime::now();
    let response = response_parts(
        Response::builder()
            .header(header::DATE, now_rfc2822())
            .header(header::LAST_MODIFIED, "Mon, 07 Mar 2016 11:52:56 GMT"),
    );
    let policy = harness().time(now).test_with_response(response);
    assert!(policy.time_to_live(now).as_secs() > 100);
}

#[test]
fn immutable_simple_hit() {
    harness()
        .assert_time_to_live(999999)
        .test_with_cache_control("immutable, max-age=999999");
}

#[test]
fn immutable_can_expire() {
    harness()
        .stale_and_store()
        .test_with_cache_control("immutable, max-age=0");
}

#[test]
fn pragma_no_cache() {
    let response = response_parts(
        Response::builder()
            .header(header::PRAGMA, "no-cache")
            .header(header::LAST_MODIFIED, "Mon, 07 Mar 2016 11:52:56 GMT"),
    );
    harness().stale_and_store().test_with_response(response);
}

#[test]
fn blank_cache_control_and_pragma_no_cache() {
    let response = response_parts(
        Response::builder()
            .header(header::CACHE_CONTROL, "")
            .header(header::PRAGMA, "no-cache")
            .header(header::LAST_MODIFIED, "Mon, 07 Mar 2016 11:52:56 GMT"),
    );
    harness().test_with_response(response);
}

#[test]
fn no_store() {
    harness()
        .no_store()
        .test_with_cache_control("no-store, public, max-age=1");
}

#[test]
fn observe_private_cache() {
    let private_header = "private, max-age=1234";
    let response =
        response_parts(Response::builder().header(header::CACHE_CONTROL, private_header));

    let _shared = harness().no_store().test_with_response(response.clone());

    let _private = harness()
        .assert_time_to_live(1234)
        .config(private_config())
        .test_with_response(response);
}

#[test]
fn do_not_share_cookies() {
    let response = response_parts(
        Response::builder()
            .header(header::SET_COOKIE, "foo=bar")
            .header(header::CACHE_CONTROL, "max-age=99"),
    );

    let _shared = harness()
        .stale_and_store()
        .test_with_response(response.clone());

    let _private = harness()
        .assert_time_to_live(99)
        .config(private_config())
        .test_with_response(response);
}

#[test]
fn do_share_cookies_if_immutable() {
    let response = response_parts(
        Response::builder()
            .header(header::SET_COOKIE, "foo=bar")
            .header(header::CACHE_CONTROL, "immutable, max-age=99"),
    );
    harness()
        .assert_time_to_live(99)
        .test_with_response(response);
}

#[test]
fn cache_explicitly_public_cookie() {
    let response = response_parts(
        Response::builder()
            .header(header::SET_COOKIE, "foo=bar")
            .header(header::CACHE_CONTROL, "max-age=5, public"),
    );
    harness()
        .assert_time_to_live(5)
        .test_with_response(response);
}

#[test]
fn miss_max_age_equals_zero() {
    harness()
        .stale_and_store()
        .test_with_cache_control("public, max-age=0");
}

#[test]
fn uncacheable_503_service_unavailable() {
    let response = response_parts(
        Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .header(header::CACHE_CONTROL, "public, max-age=0"),
    );
    harness().no_store().test_with_response(response);
}

#[test]
fn cacheable_301_moved_permanently() {
    let response = response_parts(
        Response::builder()
            .status(StatusCode::MOVED_PERMANENTLY)
            .header(header::LAST_MODIFIED, "Mon, 07 Mar 2016 11:52:56 GMT"),
    );
    harness().test_with_response(response);
}

#[test]
fn uncacheable_303_see_other() {
    let response = response_parts(
        Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(header::LAST_MODIFIED, "Mon, 07 Mar 2016 11:52:56 GMT"),
    );
    harness().no_store().test_with_response(response);
}

#[test]
fn cacheable_303_see_other() {
    let response = response_parts(
        Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(header::CACHE_CONTROL, "max-age=1000"),
    );
    harness().test_with_response(response);
}

#[test]
fn uncacheable_412_precondition_failed() {
    let response = response_parts(
        Response::builder()
            .status(StatusCode::PRECONDITION_FAILED)
            .header(header::CACHE_CONTROL, "public, max-age=1000"),
    );
    harness().no_store().test_with_response(response);
}

#[test]
fn expired_expires_cache_with_max_age() {
    let response = response_parts(
        Response::builder()
            .header(header::CACHE_CONTROL, "public, max-age=9999")
            .header(header::EXPIRES, "Sat, 07 May 2016 15:35:18 GMT"),
    );
    harness()
        .assert_time_to_live(9999)
        .test_with_response(response);
}

#[test]
fn request_mismatches() {
    let now = SystemTime::now();
    let mut req = request_parts(Request::builder().uri("/test"));
    let response = response_parts(
        Response::builder()
            .header(header::CACHE_CONTROL, "public, max-age=9999")
            .header(header::EXPIRES, "Sat, 07 May 2016 15:35:18 GMT"),
    );
    let policy = harness()
        .time(now)
        .request(req.clone())
        .test_with_response(response);

    req.method = Method::POST;
    let mismatch = policy.before_request(&req, now);
    assert!(matches!(mismatch, http_cache_policy::BeforeRequest::Stale {matches, ..} if !matches));
}

#[test]
fn request_matches() {
    let now = SystemTime::now();
    let req = request_parts(Request::builder().uri("/test"));
    let policy = harness()
        .stale_and_store()
        .time(now)
        .request(req.clone())
        .test_with_cache_control("public, max-age=0");

    let mismatch = policy.before_request(&req, now);
    assert!(matches!(mismatch, http_cache_policy::BeforeRequest::Stale {matches, ..} if matches));
}

#[test]
fn expired_expires_cached_with_s_maxage() {
    let now = SystemTime::now();
    let response = response_parts(
        Response::builder()
            .header(header::CACHE_CONTROL, "public, s-maxage=9999")
            .header(header::EXPIRES, "Sat, 07 May 2016 15:35:18 GMT"),
    );

    let _shared = harness()
        .assert_time_to_live(9999)
        .time(now)
        .test_with_response(response.clone());

    let _private = harness()
        .stale_and_store()
        .time(now)
        .config(private_config())
        .test_with_response(response);
}

#[test]
fn max_age_wins_over_future_expires() {
    let in_one_hour = SystemTime::now() + Duration::from_secs(60 * 60);
    let in_one_hour = httpdate::fmt_http_date(in_one_hour);
    let response = response_parts(
        Response::builder()
            .header(header::CACHE_CONTROL, "public, max-age=333")
            .header(header::EXPIRES, in_one_hour),
    );
    harness()
        .assert_time_to_live(333)
        .test_with_response(response);
}

fn get_cached_response(
    policy: &CachePolicy,
    req: &impl http_cache_policy::RequestLike,
    now: SystemTime,
) -> http::response::Parts {
    match policy.before_request(req, now) {
        http_cache_policy::BeforeRequest::Fresh(res) => res,
        _ => panic!("stale"),
    }
}
