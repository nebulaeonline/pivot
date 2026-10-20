fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/pivot/v1/protocol.proto");
    println!("cargo:rerun-if-changed=proto/pivot/v1/types.proto");
    println!("cargo:rerun-if-changed=proto/pivot/authoring/v1/authoring.proto");
    prost_build::Config::new()
        .file_descriptor_set_path(
            std::path::PathBuf::from(std::env::var("OUT_DIR")?).join("schema.bin"),
        )
        .protoc_executable(protoc_bin_vendored::protoc_bin_path()?)
        .compile_protos(&["proto/pivot/v1/protocol.proto"], &["proto"])?;
    println!("cargo:rerun-if-changed=proto/pivot/application/v1/session.proto");
    // Separate compilation keeps the frozen hosted descriptor set unchanged.
    prost_build::Config::new()
        .extern_path(".pivot.v1", "crate::v1")
        .protoc_executable(protoc_bin_vendored::protoc_bin_path()?)
        .compile_protos(&["proto/pivot/application/v1/session.proto"], &["proto"])?;
    Ok(())
}
