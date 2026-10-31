//! Lightweight admin HTTP surface (no extra crates).
//!
//! - `GET /health` → plain `ok`
//! - `GET /workers` → JSON list of known workers (+ optional load)
//! - `GET /metrics` is served by [`crate::prometheus_server`]

use crate::discovery::{WorkerNode, WorkerRegistry};
use crate::scheduler::LoadTracker;
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

#[derive(Serialize)]
struct WorkerView {
    id: String,
    endpoint: String,
    models: Vec<String>,
    capacity: u32,
    last_seen: u64,
    in_flight: u32,
    healthy: bool,
}

#[derive(Serialize)]
struct WorkersResponse {
    workers: Vec<WorkerView>,
    count: usize,
}

fn snapshot(reg: &WorkerRegistry, load: Option<&LoadTracker>) -> String {
    let ttl = reg.ttl;
    let workers: Vec<WorkerView> = reg
        .healthy()
        .into_iter()
        .chain(
            // include stale for visibility — re-list all via get is hard; only healthy here
            std::iter::empty::<&WorkerNode>(),
        )
        .map(|n| WorkerView {
            id: n.id.clone(),
            endpoint: n.endpoint.clone(),
            models: n.models.clone(),
            capacity: n.capacity,
            last_seen: n.last_seen,
            in_flight: load.map(|l| l.in_flight(&n.id)).unwrap_or(0),
            healthy: n.is_fresh(ttl),
        })
        .collect();
    // Also dump full registry via internal access: serialize healthy only is OK.
    let body = WorkersResponse {
        count: workers.len(),
        workers,
    };
    serde_json::to_string_pretty(&body).unwrap_or_else(|_| "{}".into())
}

async fn handle(
    mut stream: TcpStream,
    registry: Arc<Mutex<WorkerRegistry>>,
    load: Option<Arc<Mutex<LoadTracker>>>,
) -> std::io::Result<()> {
    let mut buf = vec![0u8; 2048];
    let n = stream.read(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }
    let req = String::from_utf8_lossy(&buf[..n]);
    let first = req.lines().next().unwrap_or("");

    let (status, ctype, body) = if first.starts_with("GET /workers") {
        let reg = registry.lock().await;
        let load_guard = match &load {
            Some(l) => Some(l.lock().await),
            None => None,
        };
        let json = snapshot(&reg, load_guard.as_deref());
        ("200 OK", "application/json; charset=utf-8", json)
    } else if first.starts_with("GET /health") {
        ("200 OK", "text/plain; charset=utf-8", "ok\n".into())
    } else {
        (
            "404 Not Found",
            "text/plain; charset=utf-8",
            "not found\n".into(),
        )
    };

    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    if let Err(e) = stream.write_all(response.as_bytes()).await {
        warn!(error = %e, "admin write failed");
    }
    Ok(())
}

/// Spawn admin server (`/workers`, `/health`). Returns bound address.
pub async fn spawn_admin_server(
    addr: SocketAddr,
    registry: Arc<Mutex<WorkerRegistry>>,
    load: Option<Arc<Mutex<LoadTracker>>>,
) -> std::io::Result<SocketAddr> {
    let listener = TcpListener::bind(addr).await?;
    let local = listener.local_addr()?;
    info!(%local, "admin listening on /workers and /health");

    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    let reg = Arc::clone(&registry);
                    let load = load.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle(stream, reg, load).await {
                            debug!(%peer, error = %e, "admin connection closed");
                        }
                    });
                }
                Err(e) => warn!(error = %e, "admin accept failed"),
            }
        }
    });

    Ok(local)
}
