//! Tests for provider parsing and server URL normalization.

use netspd::engine::models::Server;
use netspd::engine::network::info::info_url;
use netspd::engine::providers::{builtin_servers, parse_server_list};

#[test]
fn parses_valid_server_list() {
    let json = r#"[
        {
            "id": 1,
            "name": "Frankfurt, Germany",
            "server": "//fra.example.net/backend/",
            "dlURL": "garbage.php",
            "ulURL": "empty.php",
            "pingURL": "empty.php",
            "getIpURL": "getIP.php"
        }
    ]"#;
    let servers = parse_server_list(json).unwrap_or_default();
    assert_eq!(servers.len(), 1);
    let server = &servers[0];
    assert_eq!(server.name, "Frankfurt, Germany");
    assert_eq!(server.description, "fra.example.net");
    assert_eq!(
        server.endpoints.download,
        "https://fra.example.net/backend/garbage.php?ckSize=100"
    );
    assert_eq!(
        server.endpoints.upload,
        "https://fra.example.net/backend/empty.php"
    );
    assert_eq!(
        server.endpoints.ping,
        "https://fra.example.net/backend/empty.php"
    );
}

#[test]
fn skips_incomplete_entries() {
    let json = r#"[
        {"id": 1, "name": "No endpoints", "server": "//host/"},
        {
            "name": "Complete",
            "server": "https://ok.example.com",
            "dlURL": "garbage.php?ckSize=50",
            "ulURL": "empty.php",
            "pingURL": "empty.php"
        }
    ]"#;
    let servers = parse_server_list(json).unwrap_or_default();
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].name, "Complete");
    // Existing ckSize query is preserved, not duplicated.
    assert_eq!(
        servers[0].endpoints.download,
        "https://ok.example.com/garbage.php?ckSize=50"
    );
}

#[test]
fn malformed_json_is_an_error() {
    assert!(parse_server_list("not json").is_err());
    assert!(parse_server_list("{\"object\": true}").is_err());
}

#[test]
fn builtin_servers_are_valid() {
    let servers = builtin_servers();
    assert!(!servers.is_empty());
    for server in &servers {
        assert!(server.endpoints.ping.starts_with("https://"));
        assert!(server.endpoints.download.contains("ckSize="));
        assert!(!server.name.is_empty());
    }
}

#[test]
fn info_url_is_derived_from_ping_url() {
    assert_eq!(
        info_url("https://host.example.com/backend/empty.php"),
        "https://host.example.com/backend/getIP.php?isp=true"
    );
    // Query strings on the ping URL do not leak into the info URL.
    assert_eq!(
        info_url("https://host.example.com/empty.php?r=1"),
        "https://host.example.com/getIP.php?isp=true"
    );
}

#[test]
fn server_from_base_normalizes_urls() {
    let server = Server::from_base(
        "Test",
        "host.example.org/backend",
        "garbage.php?ckSize=100",
        "empty.php",
        "/empty.php",
    );
    assert_eq!(
        server.endpoints.download,
        "https://host.example.org/backend/garbage.php?ckSize=100"
    );
    // Leading slashes on paths do not double up.
    assert_eq!(
        server.endpoints.ping,
        "https://host.example.org/backend/empty.php"
    );
    assert_eq!(server.description, "host.example.org");
}
