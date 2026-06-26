use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream, ToSocketAddrs};
use std::time::Duration;

pub fn status_ping(host: &str, port: u16, handshake_host: &str) -> Result<String, String> {
    let addr = (host, port)
        .to_socket_addrs()
        .map_err(|error| error.to_string())?
        .next()
        .ok_or_else(|| "no socket address".to_string())?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(3))
        .map_err(|error| error.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .map_err(|error| error.to_string())?;
    write_packet(&mut stream, handshake(handshake_host, port))?;
    write_packet(&mut stream, vec![0])?;
    let length = read_varint(&mut stream)?;
    let mut payload = vec![0; length as usize];
    stream
        .read_exact(&mut payload)
        .map_err(|error| error.to_string())?;
    let _ = stream.shutdown(Shutdown::Both);
    if payload.first() == Some(&0) {
        Ok("minecraft status ping returned JSON".to_string())
    } else {
        Err("unexpected status packet".to_string())
    }
}

fn handshake(host: &str, port: u16) -> Vec<u8> {
    let mut out = Vec::new();
    write_varint_to(&mut out, 0);
    write_varint_to(&mut out, 760);
    write_string(&mut out, host);
    out.extend_from_slice(&port.to_be_bytes());
    write_varint_to(&mut out, 1);
    out
}

fn write_packet(stream: &mut TcpStream, payload: Vec<u8>) -> Result<(), String> {
    let mut packet = Vec::new();
    write_varint_to(&mut packet, payload.len() as i32);
    packet.extend_from_slice(&payload);
    stream.write_all(&packet).map_err(|error| error.to_string())
}

fn write_string(out: &mut Vec<u8>, value: &str) {
    write_varint_to(out, value.len() as i32);
    out.extend_from_slice(value.as_bytes());
}

fn write_varint_to(out: &mut Vec<u8>, value: i32) {
    let mut value = value as u32;
    loop {
        let mut temp = (value & 0b0111_1111) as u8;
        value >>= 7;
        if value != 0 {
            temp |= 0b1000_0000;
        }
        out.push(temp);
        if value == 0 {
            break;
        }
    }
}

fn read_varint(stream: &mut TcpStream) -> Result<i32, String> {
    let mut result = 0;
    for index in 0..5 {
        let mut byte = [0_u8; 1];
        stream
            .read_exact(&mut byte)
            .map_err(|error| error.to_string())?;
        result |= i32::from(byte[0] & 0b0111_1111) << (7 * index);
        if byte[0] & 0b1000_0000 == 0 {
            return Ok(result);
        }
    }
    Err("varint too long".to_string())
}
