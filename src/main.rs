use lumina_inference::{
    batch::InferenceRequest, config::EngineConfig, engine::InferenceEngine,
    registry::{ModelInfo, ModelRegistry}, Result,
};
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

    let demo_cfg = EngineConfig {
        max_batch_size: 4,
        worker_slots: 2,
        ..Default::default()
    };
    let mut demo = InferenceEngine::new(demo_cfg)?;

    println!(
        "Lumina-Inference {} — workers={} batch_max={} models={}",
        env!("CARGO_PKG_VERSION"),
        demo.config().worker_slots,
        demo.config().max_batch_size,
        registry.list().len()
    );

    for i in 0..4 {
        let req = InferenceRequest {
            id: format!("req-{i}"),
            model: demo.config().default_model.clone(),
            input: format!("token-{i}").into_bytes(),
        };
        if let Some(batch) = demo.submit(req)? {
            for resp in batch {
                println!(
                    "  {} model={} out={} latency_ms={}",
                    resp.id,
                    resp.model,
                    String::from_utf8_lossy(&resp.output),
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
