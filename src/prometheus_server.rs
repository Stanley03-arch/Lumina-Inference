//! Tiny HTTP server for Prometheus scrapes — no extra crate required.
//!
//! Serves `GET /metrics` in Prometheus text format from a shared [`Metrics`] handle.

use crate::metrics::Metrics;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, info, warn};

/// Bind and serve `/metrics` until the runtime is dropped.
pub async fn serve_metrics(addr: SocketAddr, metrics: Arc<Metrics>) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    info!(%addr, "Prometheus metrics listening on /metrics");

    loop {
        let (stream, peer) = listener.accept().await?;
        let metrics = Arc::clone(&metrics);
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, metrics).await {
                debug!(%peer, error = %e, "metrics connection closed");
            }
        });
    }
}

async fn handle_connection(mut stream: TcpStream, metrics: Arc<Metrics>) -> std::io::Result<()> {
    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }
    let req = String::from_utf8_lossy(&buf[..n]);
    let first_line = req.lines().next().unwrap_or("");

    let (status, content_type, body) = if first_line.starts_with("GET /metrics") {
        (
            "200 OK",
            "text/plain; version=0.0.4; charset=utf-8",
            metrics.encode_prometheus(),
        )
    } else if first_line.starts_with("GET /health") || first_line.starts_with("GET / ") {
        ("200 OK", "text/plain; charset=utf-8", "ok\n".into())
    } else {
        (
            "404 Not Found",
            "text/plain; charset=utf-8",
            "not found\n".into(),
        )
    };

    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );

    if let Err(e) = stream.write_all(response.as_bytes()).await {
        warn!(error = %e, "failed to write metrics response");
    }
    Ok(())
}

/// Spawn the metrics server on a background task. Returns the bound address.
pub async fn spawn_metrics_server(
    addr: SocketAddr,
    metrics: Arc<Metrics>,
) -> std::io::Result<SocketAddr> {
    let listener = TcpListener::bind(addr).await?;
    let local = listener.local_addr()?;
    info!(%local, "Prometheus metrics listening on /metrics");

    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    let metrics = Arc::clone(&metrics);
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, metrics).await {
                            debug!(%peer, error = %e, "metrics connection closed");
                        }
                    });
                }
                Err(e) => {
                    warn!(error = %e, "metrics accept failed");
                }
            }
        }
    });

    Ok(local)
}
