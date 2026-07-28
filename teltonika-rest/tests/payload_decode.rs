//! Decode tests over the vendored spec's own examples. No network.

use teltonika_rest::api::system::UsageStatus;

#[test]
fn decodes_spec_example() {
    let json = serde_json::json!({
        "memory": {
            "ram_buffered": 8.35, "ram_total": 254.57, "ram_used": 159.42,
            "flash_total": 97.59, "ram_free": 95.15, "flash_free": 97.07,
            "flash_percentage": 0.53, "flash_used": 0.52,
            "ram_percentage": 62.62, "ram_shared": 0.28
        },
        "uptime": "08h 09m 16s",
        "loadavg": 0.02,
        "localtime": 1_645_461_193i64,
        "load": { "min5": 0.101_564, "min15": 0.042_481, "min1": 0.076_661 },
        "uptime_seconds": 29_356
    });

    let s: UsageStatus = serde_json::from_value(json).unwrap();
    assert_eq!(s.uptime_seconds, Some(29_356));
    assert_eq!(s.memory.unwrap().ram_total, Some(254.57));
    assert!(s.load.unwrap().min1.unwrap() < 1.0);
}

#[test]
fn decodes_spec_x_example_with_divergent_load_scale() {
    let json = serde_json::json!({
        "memory": { "ram_total": 254.57, "ram_used": 162.78, "ram_percentage": 63.94 },
        "uptime": "07h 56m 02s",
        "loadavg": 2.5,
        "localtime": 1_645_460_400i64,
        "load": { "min5": 1056, "min15": 320, "min1": 4800 },
        "uptime_seconds": 28_562
    });

    let s: UsageStatus = serde_json::from_value(json).unwrap();
    assert_eq!(s.load.unwrap().min1, Some(4800.0));
    // loadavg and load.min1 disagree in the vendor's own example.
    assert_eq!(s.loadavg, Some(2.5));
}

/// `?data=uptime` -- the endpoint's subset params mean whole groups vanish.
/// This is the case that justifies every field being Option.
#[test]
fn decodes_subset_response() {
    let json = serde_json::json!({ "uptime": "08h 09m 16s" });

    let s: UsageStatus = serde_json::from_value(json).unwrap();
    assert_eq!(s.uptime.as_deref(), Some("08h 09m 16s"));
    assert!(s.memory.is_none());
    assert!(s.load.is_none());
    assert!(s.uptime_seconds.is_none());
}

/// Unknown fields must not break decoding -- firmware bumps add fields.
#[test]
fn tolerates_unknown_fields() {
    let json = serde_json::json!({
        "uptime_seconds": 100,
        "some_future_field": { "nested": true }
    });

    let s: UsageStatus = serde_json::from_value(json).unwrap();
    assert_eq!(s.uptime_seconds, Some(100));
}
