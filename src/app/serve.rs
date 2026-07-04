//! The built-in speed test server (`netspd serve`).
//!
//! A deliberately small HTTP/1.1 listener so one static binary can play
//! both roles: run `netspd serve` in one pod and point another pod's
//! `netspd --url http://that-pod:9516` at it — no external backend, no
//! extra image. The endpoints are also LibreSpeed-path compatible
//! (`empty.php`, `garbage.php`), so existing clients work against it
//! too.
//!
//! Endpoints:
//! - `GET /ping` (alias `/empty.php`) — instant tiny 200;
//! - `GET /download` (alias `/garbage.php`) — streams an incompressible
//!   payload; `?bytes=N` or LibreSpeed's `?ckSize=<MB>` set the size;
//! - `POST /upload` (alias `/empty.php`) — drains the body, returns 200.

use std::net::SocketAddr;

use anyhow::Context;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

/// The default port; unassigned by IANA and easy to remember (`9516`
/// spells "n-e-t" poorly, but it's stable).
pub const DEFAULT_PORT: u16 = 9516;

/// Default download payload per request when no size is requested.
const DEFAULT_DOWNLOAD_BYTES: u64 = 100 * 1024 * 1024;

/// Cap on requested download size, per request (workers loop anyway).
const MAX_DOWNLOAD_BYTES: u64 = 1024 * 1024 * 1024;

/// Upper bound on request head size.
const MAX_HEAD_BYTES: usize = 8 * 1024;

/// Write chunk for download streaming.
const CHUNK_BYTES: usize = 64 * 1024;

/// Runs the server until Ctrl+C.
pub async fn run(bind: &str, port: u16) -> anyhow::Result<()> {
    let address: SocketAddr = format!("{bind}:{port}")
        .parse()
        .with_context(|| format!("invalid bind address {bind:?}"))?;
    let listener = TcpListener::bind(address)
        .await
        .with_context(|| format!("failed to bind {address}"))?;
    eprintln!(
        "netspd {} — serving speed tests on http://{address}",
        env!("CARGO_PKG_VERSION")
    );
    eprintln!("point a client at it:  netspd --url http://<this-host>:{port}");

    let payload = incompressible_chunk();
    loop {
        tokio::select! {
            result = listener.accept() => {
                let (stream, _) = result.context("accept failed")?;
                let payload = payload.clone();
                tokio::spawn(async move {
                    // Connection errors are the client's problem.
                    let _ = handle_connection(stream, &payload).await;
                });
            }
            result = tokio::signal::ctrl_c() => {
                result.context("signal handler failed")?;
                eprintln!("shutting down");
                return Ok(());
            }
        }
    }
}

/// Serves HTTP/1.1 requests on one connection until it closes.
async fn handle_connection(stream: TcpStream, payload: &[u8]) -> std::io::Result<()> {
    stream.set_nodelay(true)?;
    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);

    loop {
        let Some(request) = read_request(&mut reader).await? else {
            return Ok(()); // clean close
        };
        match (request.method.as_str(), request.path.as_str()) {
            ("GET", "/ping") | ("GET", "/empty.php") | ("HEAD", _) => {
                write_half
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok")
                    .await?;
            }
            ("GET", "/download") | ("GET", "/garbage.php") => {
                let total = requested_bytes(&request.query);
                let head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {total}\r\n\r\n"
                );
                write_half.write_all(head.as_bytes()).await?;
                let mut remaining = total;
                while remaining > 0 {
                    let take = (remaining as usize).min(payload.len());
                    write_half.write_all(&payload[..take]).await?;
                    remaining -= take as u64;
                }
            }
            ("POST", "/upload") | ("POST", "/empty.php") => {
                if request.chunked {
                    drain_chunked_body(&mut reader).await?;
                } else {
                    drain_body(&mut reader, request.content_length).await?;
                }
                write_half
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok")
                    .await?;
            }
            _ => {
                write_half
                    .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n")
                    .await?;
            }
        }
        write_half.flush().await?;
    }
}

/// One parsed request head.
struct Request {
    method: String,
    path: String,
    query: String,
    content_length: u64,
    chunked: bool,
}

/// Reads and parses a request head; `None` on a cleanly closed
/// connection.
async fn read_request(
    reader: &mut BufReader<tokio::net::tcp::OwnedReadHalf>,
) -> std::io::Result<Option<Request>> {
    let mut head = Vec::with_capacity(512);
    let mut byte = [0u8; 1];
    loop {
        match reader.read(&mut byte).await? {
            0 => {
                return if head.is_empty() {
                    Ok(None)
                } else {
                    Err(std::io::ErrorKind::UnexpectedEof.into())
                };
            }
            _ => head.push(byte[0]),
        }
        if head.ends_with(b"\r\n\r\n") {
            break;
        }
        if head.len() > MAX_HEAD_BYTES {
            return Err(std::io::ErrorKind::InvalidData.into());
        }
    }

    let text = String::from_utf8_lossy(&head);
    let mut lines = text.lines();
    let request_line = lines.next().unwrap_or_default();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_owned();
    let target = parts.next().unwrap_or_default();
    let (path, query) = match target.split_once('?') {
        Some((path, query)) => (path.to_owned(), query.to_owned()),
        None => (target.to_owned(), String::new()),
    };

    let mut content_length = 0;
    let mut chunked = false;
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value.trim().parse().unwrap_or(0);
            }
            if name.eq_ignore_ascii_case("transfer-encoding")
                && value.trim().eq_ignore_ascii_case("chunked")
            {
                chunked = true;
            }
        }
    }
    Ok(Some(Request {
        method,
        path,
        query,
        content_length,
        chunked,
    }))
}

/// Consumes exactly `length` body bytes.
async fn drain_body(
    reader: &mut BufReader<tokio::net::tcp::OwnedReadHalf>,
    length: u64,
) -> std::io::Result<()> {
    let mut remaining = length;
    let mut buffer = [0u8; CHUNK_BYTES];
    while remaining > 0 {
        let take = (remaining as usize).min(buffer.len());
        let read = reader.read(&mut buffer[..take]).await?;
        if read == 0 {
            return Err(std::io::ErrorKind::UnexpectedEof.into());
        }
        remaining -= read as u64;
    }
    Ok(())
}

/// Consumes a `Transfer-Encoding: chunked` body (streaming uploads send
/// these) up to and including the terminating zero-length chunk.
async fn drain_chunked_body(
    reader: &mut BufReader<tokio::net::tcp::OwnedReadHalf>,
) -> std::io::Result<()> {
    loop {
        // Chunk-size line: hex digits terminated by CRLF.
        let mut line = Vec::with_capacity(16);
        let mut byte = [0u8; 1];
        loop {
            if reader.read(&mut byte).await? == 0 {
                return Err(std::io::ErrorKind::UnexpectedEof.into());
            }
            line.push(byte[0]);
            if line.ends_with(b"\r\n") {
                break;
            }
            if line.len() > 32 {
                return Err(std::io::ErrorKind::InvalidData.into());
            }
        }
        let size_text = String::from_utf8_lossy(&line);
        let size_text = size_text.trim().split(';').next().unwrap_or_default();
        let size = u64::from_str_radix(size_text, 16)
            .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidData))?;

        // Chunk data plus its trailing CRLF; the zero chunk ends the body.
        drain_body(reader, size + 2).await?;
        if size == 0 {
            return Ok(());
        }
    }
}

/// Payload size from the query string: `bytes=N` (netspd) or
/// `ckSize=<MB>` (LibreSpeed), clamped.
fn requested_bytes(query: &str) -> u64 {
    for pair in query.split('&') {
        if let Some(value) = pair.strip_prefix("bytes=") {
            if let Ok(bytes) = value.parse::<u64>() {
                return bytes.clamp(1, MAX_DOWNLOAD_BYTES);
            }
        }
        if let Some(value) = pair.strip_prefix("ckSize=") {
            if let Ok(megabytes) = value.parse::<u64>() {
                return (megabytes * 1024 * 1024).clamp(1, MAX_DOWNLOAD_BYTES);
            }
        }
    }
    DEFAULT_DOWNLOAD_BYTES
}

/// A reusable incompressible chunk, so nothing on the path can inflate
/// the measurement by compressing zeros.
fn incompressible_chunk() -> std::sync::Arc<Vec<u8>> {
    let mut data = vec![0u8; CHUNK_BYTES];
    let mut state: u64 = 0x2545_f491_4f6c_dd1d;
    for slot in &mut data {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        *slot = (state & 0xff) as u8;
    }
    std::sync::Arc::new(data)
}
