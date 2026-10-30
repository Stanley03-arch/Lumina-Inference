use lumina_inference::{
    batch::InferenceRequest, config::EngineConfig, engine::InferenceEngine, Result,
};
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    let cfg = EngineConfig::default();
    let mut engine = InferenceEngine::new(cfg)?;

    println!(
        "Lumina-Inference {} — workers={} batch_max={}",
        env!("CARGO_PKG_VERSION"),
        engine.config().worker_slots,
        engine.config().max_batch_size
    );

    // Demo: fill a small batch and print echo results.
    let demo_cfg = EngineConfig {
        max_batch_size: 4,
        worker_slots: 2,
        ..Default::default()
    };
    let mut demo = InferenceEngine::new(demo_cfg)?;

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

    println!("demo complete");
    Ok(())
}
