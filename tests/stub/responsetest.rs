use crate::Harness;
use http::{header, request, Request, Response};
use http_cache_policy::{CachePolicy, Config, ResponseLike};
use std::time::{Duration, SystemTime};

macro_rules! headers(
    { $($key:tt : $value:expr),* $(,)? } => {
        {
            let mut m = Response::builder();
            $(
                m = m.header($key, $value);
            )+
            m.body(()).unwrap()
        }
     };
);

fn req() -> request::Parts {
    Request::get("http://test.example.com/")
        .body(())
        .unwrap()
        .into_parts()
        .0
}

fn harness() -> Harness {
    Harness::default().request(req())
}

#[test]
fn weird_syntax() {
    harness()
        .assert_time_to_live(456)
        .test_with_cache_control(",,,,max-age =  456      ,");
}

#[test]
fn pre_check_poison_undefined_header() {
    let now = SystemTime::now();
    let orig_cc = "pre-check=0, post-check=0, no-cache, no-store";
    let config = Config {
        ignore_cargo_cult: true,
        ..Default::default()
    };
    let cache = harness()
        .stale_and_store()
        .config(config)
        .time(now)
        .test_with_response(headers! { "cache-control": orig_cc, "expires": "yesterday!"});

    let res = &get_cached_response(
        &cache,
        &Request::get("http://test.example.com/")
            .header("cache-control", "max-stale")
            .body(())
            .unwrap(),
        now,
    );
    let _cc = &res.headers()[header::CACHE_CONTROL];

    assert!(res.headers().get(header::EXPIRES).is_none());
}

#[test]
fn cache_with_expires() {
    let now = SystemTime::now();
    let response = headers! {
        "date": date_str(now),
        "expires": date_str(now + Duration::from_millis(2001)),
    };
    harness()
        .assert_time_to_live(2)
        .time(now)
        .test_with_response(response);
}

#[test]
fn cache_with_expires_relative_to_date() {
    let now = SystemTime::now();
    let response = headers! {
        "date": date_str(now - Duration::from_secs(30)),
        "expires": date_str(now),
    };
    harness()
        .assert_time_to_live(30)
        .time(now)
        .test_with_response(response);
}

#[test]
fn cache_with_expires_always_relative_to_date() {
    let now = SystemTime::now();
    let response = headers! {
        "date": date_str(now - Duration::from_secs(3)),
        "expires": date_str(now),
    };
    harness()
        .assert_time_to_live(3)
        .time(now)
        .test_with_response(response);
}

#[test]
fn cache_expires_no_date() {
    let now = SystemTime::now();
    let response = headers! {
        "cache-control": "public",
        "expires": date_str(now + Duration::from_secs(3600)),
    };
    let cache = harness().time(now).test_with_response(response);
    assert!(cache.time_to_live(now).as_secs() > 3595);
    assert!(cache.time_to_live(now).as_secs() < 3605);
}

#[test]
fn ages() {
    let mut now = SystemTime::now();
    let response = headers! {
        "cache-control": "max-age=100",
        "age": "50",
    };
    let cache = harness()
        .assert_time_to_live(50)
        .time(now)
        .test_with_response(response);

    now += Duration::from_secs(48);
    assert_eq!(2, cache.time_to_live(now).as_secs());
    assert!(!cache.is_stale(now));

    now += Duration::from_secs(5);
    assert!(cache.is_stale(now));
    assert_eq!(0, cache.time_to_live(now).as_secs());
}

#[test]
fn remove_hop_headers() {
    let mut now = SystemTime::now();
    let res = headers! {
        "te": "deflate",
        "date": "now",
        "custom": "header",
        "oompa": "lumpa",
        "connection": "close, oompa, header",
        "age": "10",
        "cache-control": "public, max-age=333",
    };
    let cache = harness().time(now).test_with_response(res.clone());

    now += Duration::from_millis(1005);
    let h = get_cached_response(&cache, &req(), now);
    let h = h.headers();
    assert!(!h.contains_key("connection"));
    assert!(!h.contains_key("te"));
    assert!(!h.contains_key("oompa"));
    assert_eq!(h["cache-control"], "public, max-age=333");
    assert_ne!(h["date"], "now", "updated age requires updated date");
    assert_eq!(h["custom"].to_str().unwrap(), "header");
    assert_eq!(h["age"].to_str().unwrap(), "11");
}

fn date_str(now: SystemTime) -> String {
    httpdate::fmt_http_date(now)
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
