use anyhow::{Result, ensure};
use serde_json::Value;
use std::net::Ipv4Addr;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

fn varint(mut n: u32, out: &mut Vec<u8>) {
    loop {
        if n & !127 == 0 {
            out.push(n as u8);
            return;
        }
        out.push((n as u8 & 127) | 128);
        n >>= 7;
    }
}
async fn read_varint(stream: &mut TcpStream) -> Result<usize> {
    let mut n = 0;
    for shift in (0..35).step_by(7) {
        let b = stream.read_u8().await?;
        n |= ((b & 127) as usize) << shift;
        if b & 128 == 0 {
            return Ok(n);
        }
    }
    anyhow::bail!("Invalid status packet length")
}
pub async fn minecraft(address: Ipv4Addr) -> Result<Value> {
    tokio::time::timeout(std::time::Duration::from_secs(3), async move {
        let mut stream = TcpStream::connect((address, 25565)).await?;
        let host = address.to_string();
        let mut handshake = vec![0];
        varint(u32::MAX, &mut handshake);
        varint(host.len() as u32, &mut handshake);
        handshake.extend(host.bytes());
        handshake.extend(25565u16.to_be_bytes());
        handshake.push(1);
        let mut frame = Vec::new();
        varint(handshake.len() as u32, &mut frame);
        frame.extend(handshake);
        frame.extend([1, 0]);
        stream.write_all(&frame).await?;
        let frame_len = read_varint(&mut stream).await?;
        ensure!(frame_len <= 1048576, "Status packet too large");
        ensure!(
            read_varint(&mut stream).await? == 0,
            "Unexpected status packet"
        );
        let len = read_varint(&mut stream).await?;
        ensure!(
            len <= frame_len && len <= 1048576,
            "Invalid status JSON length"
        );
        let mut data = vec![0; len];
        stream.read_exact(&mut data).await?;
        Ok(serde_json::from_slice(&data)?)
    })
    .await?
}
