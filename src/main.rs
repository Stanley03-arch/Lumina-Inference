use lumina_inference::{
    batch::InferenceRequest,
    config::EngineConfig,
    engine::InferenceEngine,
    registry::{ModelInfo, ModelRegistry},
    worker::{InferenceWorker, LocalWorker},
    OnnxWorker, Result,
};
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    let mut registry = ModelRegistry::new();
    registry.register(ModelInfo {
        id: "default".into(),
        version: "0.1".into(),
        path: None,
        loaded: true,
    });

    // Optional: LUMINA_ONNX_MODEL=/path/to/model.onnx
    // With --features onnx this uses OnnxWorker; otherwise falls back to echo.
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
        worker_slots: workers.len(),
        ..Default::default()
    };
    let mut demo = InferenceEngine::with_workers(demo_cfg, workers, Arc::new(Default::default()))?;

    println!(
        "Lumina-Inference {} — workers={} batch_max={} models={}",
        env!("CARGO_PKG_VERSION"),
        demo.worker_count(),
        demo.config().max_batch_size,
        registry.list().len()
    );
    #[cfg(feature = "onnx")]
    println!("ONNX feature: enabled");
    #[cfg(not(feature = "onnx"))]
    println!("ONNX feature: disabled (cargo run --features onnx)");

    for i in 0..4 {
        let req = InferenceRequest {
            id: format!("req-{i}"),
            model: demo.config().default_model.clone(),
            // f32 LE payload for ONNX path; echo path accepts any bytes
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
        "metrics: requests={} batches={} items={} errors={}",
        m.requests_total, m.batches_total, m.batch_items_total, m.errors_total
    );
    println!("demo complete");
    Ok(())
}
