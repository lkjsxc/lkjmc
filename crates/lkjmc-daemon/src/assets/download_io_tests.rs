use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::{Arc, Barrier};
use std::thread;

use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::download_io::{download, ExpectedChecksum};

#[test]
fn truncated_download_leaves_no_final_or_partial_file() -> Result<(), String> {
    let root = root()?;
    let result = (|| {
        let (url, server) = server(b"bad".to_vec(), 10, 1)?;
        let target = root.join("server.jar");
        let error = download(&url, &target, Some(10), ExpectedChecksum::Sha256("00"))
            .err()
            .ok_or_else(|| "truncated download unexpectedly succeeded".to_string())?;
        server
            .join()
            .map_err(|_| "test server panicked".to_string())??;
        assert!(error.contains("download"));
        assert!(!target.exists());
        assert!(!has_partial(&root)?);
        Ok(())
    })();
    let _ = fs::remove_dir_all(root);
    result
}

#[test]
fn concurrent_downloads_publish_one_complete_final_file() -> Result<(), String> {
    let root = root()?;
    let result = (|| {
        let bytes = b"verified bytes".to_vec();
        let expected = format!("{:x}", Sha256::digest(&bytes));
        let (url, server) = server(bytes.clone(), bytes.len(), 1)?;
        let target = root.join("server.jar");
        let barrier = Arc::new(Barrier::new(3));
        let mut workers = Vec::new();
        for _ in 0..2 {
            let barrier = Arc::clone(&barrier);
            let url = url.clone();
            let target = target.clone();
            let expected = expected.clone();
            workers.push(thread::spawn(move || {
                barrier.wait();
                download(&url, &target, Some(14), ExpectedChecksum::Sha256(&expected))
            }));
        }
        barrier.wait();
        for worker in workers {
            worker
                .join()
                .map_err(|_| "download worker panicked".to_string())??;
        }
        server
            .join()
            .map_err(|_| "test server panicked".to_string())??;
        assert_eq!(fs::read(&target).map_err(|error| error.to_string())?, bytes);
        assert!(!has_partial(&root)?);
        Ok(())
    })();
    let _ = fs::remove_dir_all(root);
    result
}

fn root() -> Result<PathBuf, String> {
    let path = std::env::temp_dir().join(format!("lkjmc-download-test-{}", Uuid::new_v4()));
    fs::create_dir_all(&path).map_err(|error| error.to_string())?;
    Ok(path)
}

fn has_partial(root: &PathBuf) -> Result<bool, String> {
    for entry in fs::read_dir(root).map_err(|error| error.to_string())? {
        let name = entry.map_err(|error| error.to_string())?.file_name();
        if name.to_string_lossy().contains(".part") {
            return Ok(true);
        }
    }
    Ok(false)
}

fn server(
    body: Vec<u8>,
    length: usize,
    connections: usize,
) -> Result<(String, thread::JoinHandle<Result<(), String>>), String> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|error| error.to_string())?;
    let address = listener.local_addr().map_err(|error| error.to_string())?;
    let server = thread::spawn(move || {
        for _ in 0..connections {
            let (mut stream, _) = listener.accept().map_err(|error| error.to_string())?;
            let mut request = [0_u8; 1024];
            stream
                .read(&mut request)
                .map_err(|error| error.to_string())?;
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {length}\r\n\r\n"
            )
            .map_err(|error| error.to_string())?;
            stream.write_all(&body).map_err(|error| error.to_string())?;
        }
        Ok(())
    });
    Ok((format!("http://{address}/asset.jar"), server))
}
