fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Only generate stubs when building with `--features grpc`.
    if std::env::var_os("CARGO_FEATURE_GRPC").is_some() {
        tonic_build::configure()
            .build_server(true)
            .build_client(true)
            .compile_protos(&["proto/lumina/v1/inference.proto"], &["proto"])?;
    }
    Ok(())
}
