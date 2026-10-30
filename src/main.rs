use lumina_inference::{
    batch::InferenceRequest,
    config::EngineConfig,
    engine::InferenceEngine,
    prometheus_server::spawn_metrics_server,
    registry::{ModelInfo, ModelRegistry},
    worker::{InferenceWorker, LocalWorker},
    Metrics, OnnxWorker, Result,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    // Client mode: LUMINA_GRPC_CLIENT=http://127.0.0.1:50051
    #[cfg(feature = "grpc")]
    if let Ok(endpoint) = std::env::var("LUMINA_GRPC_CLIENT") {
        return run_grpc_client(&endpoint).await;
    }
    #[cfg(not(feature = "grpc"))]
    if std::env::var("LUMINA_GRPC_CLIENT").is_ok() {
        eprintln!("LUMINA_GRPC_CLIENT set but binary built without `--features grpc`");
        return Err(lumina_inference::LuminaError::Config(
            "rebuild with --features grpc".into(),
        ));
    }

    let metrics = Arc::new(Metrics::default());

    if let Ok(addr_str) = std::env::var("LUMINA_METRICS_ADDR") {
        let addr: SocketAddr = addr_str
            .parse()
            .map_err(|e| lumina_inference::LuminaError::Config(format!("metrics addr: {e}")))?;
        let bound = spawn_metrics_server(addr, Arc::clone(&metrics))
            .await
            .map_err(|e| lumina_inference::LuminaError::Internal(format!("metrics bind: {e}")))?;
        println!("Prometheus metrics: http://{bound}/metrics");
    }

    let mut registry = ModelRegistry::new();
    registry.register(ModelInfo {
        id: "default".into(),
        version: "0.1".into(),
        path: None,
        loaded: true,
    });

    let workers: Vec<Box<dyn InferenceWorker>> =
        if let Ok(path) = std::env::var("LUMINA_ONNX_MODEL") {
            match OnnxWorker::load("onnx-0", "default", &path) {
                Ok(w) => {
                    registry.register(ModelInfo {
                        id: "default".into(),
                        version: "onnx".into(),
                        path: Some(path),
                        loaded: true,
                    });
                    vec![Box::new(w)]
                }
                Err(e) => {
                    eprintln!("ONNX load failed ({e}); using LocalWorker echo backend");
                    vec![Box::new(LocalWorker::new("worker-0"))]
                }
            }
        } else {
            vec![Box::new(LocalWorker::new("worker-0"))]
        };

    let demo_cfg = EngineConfig {
        max_batch_size: 4,
        worker_slots: workers.len().max(1),
        ..Default::default()
    };
    let engine = InferenceEngine::with_workers(demo_cfg, workers, Arc::clone(&metrics))?;

    println!(
        "Lumina-Inference {} — workers={} batch_max={} models={}",
        env!("CARGO_PKG_VERSION"),
        engine.worker_count(),
        engine.config().max_batch_size,
        registry.list().len()
    );
    #[cfg(feature = "onnx")]
    println!("ONNX feature: enabled");
    #[cfg(not(feature = "onnx"))]
    println!("ONNX feature: disabled (cargo run --features onnx)");
    #[cfg(feature = "grpc")]
    println!("gRPC feature: enabled");
    #[cfg(not(feature = "grpc"))]
    println!("gRPC feature: disabled (cargo run --features grpc)");

    // Server mode: LUMINA_GRPC_ADDR=127.0.0.1:50051 (Ctrl+C to stop)
    #[cfg(feature = "grpc")]
    if let Ok(addr_str) = std::env::var("LUMINA_GRPC_ADDR") {
        let addr: SocketAddr = addr_str
            .parse()
            .map_err(|e| lumina_inference::LuminaError::Config(format!("grpc addr: {e}")))?;
        println!("gRPC Inference service on {addr} (Ctrl+C to stop)");
        lumina_inference::serve_grpc_ctrl_c(addr, engine)
            .await
            .map_err(|e| lumina_inference::LuminaError::Internal(format!("grpc: {e}")))?;
        return Ok(());
    }

    #[cfg(not(feature = "grpc"))]
    if std::env::var("LUMINA_GRPC_ADDR").is_ok() {
        eprintln!("LUMINA_GRPC_ADDR set but binary built without `--features grpc`");
    }

    let mut demo = engine;
    for i in 0..4 {
        let req = InferenceRequest {
            id: format!("req-{i}"),
            model: demo.config().default_model.clone(),
            input: (i as f32).to_le_bytes().to_vec(),
        };
        if let Some(batch) = demo.submit(req)? {
            for resp in batch {
                println!(
                    "  {} model={} out_len={} latency_ms={}",
                    resp.id,
                    resp.model,
                    resp.output.len(),
                    resp.latency_ms
                );
            }
        }
    }

    let m = demo.metrics();
    println!(
        "metrics: requests={} batches={} items={} errors={} latency_count={}",
        m.requests_total,
        m.batches_total,
        m.batch_items_total,
        m.errors_total,
        m.inference_latency_ms_count
    );

    if std::env::var("LUMINA_METRICS_ADDR").is_ok() {
        println!("metrics server running — Ctrl+C to exit (or wait 30s)");
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    } else {
        println!("demo complete");
    }
    Ok(())
}

#[cfg(feature = "grpc")]
async fn run_grpc_client(endpoint: &str) -> Result<()> {
    use lumina_inference::InferenceClient;

    println!("gRPC client → {endpoint}");
    let mut client = InferenceClient::connect(endpoint).await?;

    let (ok, version) = client.health().await?;
    println!("Health: ok={ok} version={version}");

    let resp = client
        .predict(InferenceRequest {
            id: "client-1".into(),
            model: "default".into(),
            input: 1.0f32.to_le_bytes().to_vec(),
        })
        .await?;
    println!(
        "Predict: id={} model={} out_len={} latency_ms={}",
        resp.id,
        resp.model,
        resp.output.len(),
        resp.latency_ms
    );

    let batch = client
        .predict_batch(vec![
            InferenceRequest {
                id: "b0".into(),
                model: "default".into(),
                input: 2.0f32.to_le_bytes().to_vec(),
            },
            InferenceRequest {
                id: "b1".into(),
                model: "default".into(),
                input: 3.0f32.to_le_bytes().to_vec(),
            },
        ])
        .await?;
    println!("PredictBatch: {} responses", batch.len());
    for r in &batch {
        println!("  {} out_len={} latency_ms={}", r.id, r.output.len(), r.latency_ms);
    }
    Ok(())
}
