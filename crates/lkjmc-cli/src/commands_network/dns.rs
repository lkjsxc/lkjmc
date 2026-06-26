use std::fs;
use std::net::UdpSocket;
use std::time::Duration;

use lkjmc_core::network_diagnostics::SrvRecord;

pub fn resolve_srv(name: &str) -> Vec<SrvRecord> {
    let Some(server) = nameserver() else {
        return Vec::new();
    };
    let Ok(socket) = UdpSocket::bind("0.0.0.0:0") else {
        return Vec::new();
    };
    let _ = socket.set_read_timeout(Some(Duration::from_secs(2)));
    let query = query(name);
    if socket.send_to(&query, format!("{server}:53")).is_err() {
        return Vec::new();
    }
    let mut buf = [0_u8; 1500];
    let Ok((len, _)) = socket.recv_from(&mut buf) else {
        return Vec::new();
    };
    parse_srv(&buf[..len])
}

fn nameserver() -> Option<String> {
    fs::read_to_string("/etc/resolv.conf")
        .ok()?
        .lines()
        .find_map(|line| {
            line.strip_prefix("nameserver ")
                .map(str::trim)
                .map(str::to_string)
        })
}

fn query(name: &str) -> Vec<u8> {
    let mut out = vec![0x12, 0x34, 0x01, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0];
    for label in name.trim_end_matches('.').split('.') {
        out.push(label.len() as u8);
        out.extend_from_slice(label.as_bytes());
    }
    out.extend_from_slice(&[0, 0, 33, 0, 1]);
    out
}

fn parse_srv(bytes: &[u8]) -> Vec<SrvRecord> {
    if bytes.len() < 12 {
        return Vec::new();
    }
    let qd = u16_at(bytes, 4) as usize;
    let an = u16_at(bytes, 6) as usize;
    let mut pos = 12;
    for _ in 0..qd {
        if skip_name(bytes, &mut pos).is_none() || pos + 4 > bytes.len() {
            return Vec::new();
        }
        pos += 4;
    }
    let mut records = Vec::new();
    for _ in 0..an {
        if skip_name(bytes, &mut pos).is_none() || pos + 10 > bytes.len() {
            break;
        }
        let kind = u16_at(bytes, pos);
        pos += 2;
        let class = u16_at(bytes, pos);
        pos += 6;
        let len = u16_at(bytes, pos) as usize;
        pos += 2;
        if pos + len > bytes.len() {
            break;
        }
        if kind == 33 && class == 1 && len >= 7 {
            let port = u16_at(bytes, pos + 4);
            let mut name_pos = pos + 6;
            if let Some(target) = read_name(bytes, &mut name_pos) {
                records.push(SrvRecord { target, port });
            }
        }
        pos += len;
    }
    records
}

fn skip_name(bytes: &[u8], pos: &mut usize) -> Option<()> {
    read_name(bytes, pos).map(|_| ())
}

fn read_name(bytes: &[u8], pos: &mut usize) -> Option<String> {
    let mut labels = Vec::new();
    let mut cursor = *pos;
    let mut jumped = false;
    for _ in 0..32 {
        let len = *bytes.get(cursor)?;
        if len & 0xC0 == 0xC0 {
            let next = *bytes.get(cursor + 1)? as usize;
            let offset = (((len & 0x3F) as usize) << 8) | next;
            if !jumped {
                *pos = cursor + 2;
            }
            cursor = offset;
            jumped = true;
            continue;
        }
        cursor += 1;
        if len == 0 {
            if !jumped {
                *pos = cursor;
            }
            return Some(labels.join("."));
        }
        let end = cursor + len as usize;
        labels.push(
            std::str::from_utf8(bytes.get(cursor..end)?)
                .ok()?
                .to_string(),
        );
        cursor = end;
    }
    None
}

fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([bytes[offset], bytes[offset + 1]])
}
