fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Re-run builder if proto changes
    println!("cargo:rerun-if-changed=proto/matrixserver.proto");

    let mut config = prost_build::Config::new();
    config.compile_protos(&["proto/matrixserver.proto"], &["proto/"])?;
    Ok(())
}
