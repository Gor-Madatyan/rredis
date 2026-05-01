pub mod protocol;
pub mod connection;
pub(crate) mod repr{
    include!(concat!(env!("OUT_DIR"), "/network_protocol.rs"));
}