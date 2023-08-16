fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/book.proto")?;
    Ok(())
}