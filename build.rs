use std::io::Result;
fn main() -> Result<()> {
    prost_build::compile_protos(&["src/proto/network_protocol.proto"], &["src/proto/"])?;
    Ok(())
}