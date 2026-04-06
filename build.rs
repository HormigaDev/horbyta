// build.rs
// Compiles proto/horbyta_service.proto into Rust types via tonic-build.
// Add to your Cargo.toml:
//
// [build-dependencies]
// tonic-build = "0.11"
//
// [dependencies]
// tonic  = "0.11"
// prost  = "0.12"

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false) // Rust is the client; Python is the server
        .build_client(true)
        .compile(&["proto/horbyta_service.proto"], &["proto"])?;

    // Re-run build.rs if the proto file changes
    println!("cargo:rerun-if-changed=proto/horbyta_service.proto");
    Ok(())
}
