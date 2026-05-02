use my_redis::connection::Connection;
use my_redis::protocol::{Data, Frame, NetworkFrame};

#[tokio::main]
async fn main() {
    let mut connection = Connection::new("127.0.0.1:1234").await.unwrap();
    let frame: NetworkFrame = Frame::new_set_request("seto".into(), Data::String("{'hi':true}".into()), Some(Data::Array(vec![Data::UInteger(12), Data::UInteger(1782)])));
    connection.write_frame(frame).await.unwrap();
}