//! Tests for the ICMP probe's pure helpers.

use netspd::engine::network::icmp::host_of_url;

#[test]
fn extracts_hosts_from_urls() {
    assert_eq!(
        host_of_url("https://host.example.com/backend/empty.php").as_deref(),
        Some("host.example.com")
    );
    assert_eq!(
        host_of_url("http://speedtest.example.org:6060/empty.php?r=1").as_deref(),
        Some("speedtest.example.org")
    );
    assert_eq!(
        host_of_url("https://192.0.2.7/empty.php").as_deref(),
        Some("192.0.2.7")
    );
}

#[test]
fn rejects_non_http_urls() {
    assert!(host_of_url("ftp://host/").is_none());
    assert!(host_of_url("host.example.com/empty.php").is_none());
    assert!(host_of_url("https:///empty.php").is_none());
}
