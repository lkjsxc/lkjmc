use super::literal_loopback_socket;

#[test]
fn listener_accepts_only_documented_ipv4_loopback_socket() {
    for value in [
        "localhost:8765",
        "127.0.0.2:8765",
        "127.1.2.3:8765",
        "0.0.0.0:8765",
        "[::1]:8765",
        "[::]:8765",
        "[::ffff:127.0.0.1]:8765",
        "127.0.0.1:0",
    ] {
        assert!(!literal_loopback_socket(value), "accepted {value}");
    }
    assert!(literal_loopback_socket("127.0.0.1:8765"));
}
