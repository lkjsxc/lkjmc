use super::facts::PortFacts;

pub fn allocate_backend_port(default_port: u16, facts: &PortFacts) -> Option<u16> {
    if !facts.tcp_in_use.contains(&default_port) {
        return Some(default_port);
    }
    if facts.backend_range_start > facts.backend_range_end {
        return None;
    }
    let mut port = facts.backend_range_start;
    while port <= facts.backend_range_end {
        if !facts.tcp_in_use.contains(&port) {
            return Some(port);
        }
        if port == u16::MAX {
            return None;
        }
        port += 1;
    }
    None
}
