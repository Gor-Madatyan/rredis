use std::io::Result;
fn main() -> Result<()> {
    prost_build::compile_protos(&["network_protocol/network_protocol.proto"], &["./"])?;
    Ok(())
}
