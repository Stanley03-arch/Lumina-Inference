use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Only generate stubs when building with `--features grpc`.
    if env::var_os("CARGO_FEATURE_GRPC").is_some() {
        let out_dir = PathBuf::from(env::var("OUT_DIR")?);
        let descriptor_path = out_dir.join("lumina_descriptor.bin");

        tonic_build::configure()
            .build_server(true)
            .build_client(true)
            .file_descriptor_set_path(&descriptor_path)
            .compile_protos(&["proto/lumina/v1/inference.proto"], &["proto"])?;
    }
    Ok(())
}
