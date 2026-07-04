//! Tests for the Ookla and Fast.com providers' parsing.

use std::str::FromStr;

use netspd::engine::providers::{parse_ookla_server_list, parse_targets, ProviderKind};

#[test]
fn ookla_list_maps_hosts_to_legacy_endpoints() {
    let json = r#"[
        {
            "url": "http://speedtest.example.in:8080/speedtest/upload.php",
            "name": "Mumbai",
            "sponsor": "Example ISP",
            "host": "speedtest.example.in.prod.hosts.ooklaserver.net:8080",
            "distance": 1
        },
        {"name": "No host entry", "sponsor": "X"}
    ]"#;
    let servers = parse_ookla_server_list(json).unwrap_or_default();
    assert_eq!(servers.len(), 1);
    let server = &servers[0];
    assert_eq!(server.name, "Mumbai (Example ISP)");
    assert_eq!(
        server.endpoints.ping,
        "https://speedtest.example.in.prod.hosts.ooklaserver.net:8080/speedtest/latency.txt"
    );
    assert!(server.endpoints.download.ends_with("random4000x4000.jpg"));
    assert!(server.endpoints.upload.ends_with("upload.php"));
    assert_eq!(
        server.description,
        "speedtest.example.in.prod.hosts.ooklaserver.net"
    );
}

#[test]
fn fast_targets_get_range_urls() {
    let json = r#"{
        "targets": [
            {
                "url": "https://ipv4-c089-sin001-ix.1.oca.nflxvideo.net/speedtest?c=in&e=99&t=tok",
                "location": {"city": "Singapore", "country": "SG"}
            },
            {"url": "http://insecure.example.com/speedtest"},
            {"location": {"city": "No url", "country": "XX"}}
        ]
    }"#;
    let servers = parse_targets(json).unwrap_or_default();
    assert_eq!(servers.len(), 1);
    let server = &servers[0];
    assert_eq!(server.name, "Fast.com — Singapore, SG");
    // Ranges are inserted before the signed query string.
    assert_eq!(
        server.endpoints.ping,
        "https://ipv4-c089-sin001-ix.1.oca.nflxvideo.net/speedtest/range/0-0?c=in&e=99&t=tok"
    );
    assert!(server
        .endpoints
        .download
        .contains("/speedtest/range/0-26214400?"));
    assert_eq!(server.endpoints.download, server.endpoints.upload);
}

#[test]
fn malformed_provider_responses_are_errors() {
    assert!(parse_ookla_server_list("nope").is_err());
    assert!(parse_targets("{\"targets\": 5}").is_err());
}

#[test]
fn custom_provider_requires_servers() {
    use netspd::engine::providers::create;
    assert!(create(ProviderKind::Custom, Vec::new()).is_err());

    let server = netspd::engine::models::Server::from_base(
        "LAN box",
        "http://192.168.1.50:8080/",
        "download",
        "upload",
        "ping",
    );
    assert!(create(ProviderKind::Custom, vec![server]).is_ok());
}

#[test]
fn provider_kind_parses_from_cli_strings() {
    assert_eq!(
        ProviderKind::from_str("ookla").ok(),
        Some(ProviderKind::Ookla)
    );
    assert_eq!(
        ProviderKind::from_str("Fast.com").ok(),
        Some(ProviderKind::Fast)
    );
    assert_eq!(
        ProviderKind::from_str("LIBRESPEED").ok(),
        Some(ProviderKind::Librespeed)
    );
    assert_eq!(
        ProviderKind::from_str("custom").ok(),
        Some(ProviderKind::Custom)
    );
    assert!(ProviderKind::from_str("bogus").is_err());
}
