use tokio::net::TcpStream;
use my_redis::connection::Connection;
use my_redis::protocol::{Data, Frame};

#[tokio::main]
async fn main() {
let stream = TcpStream::connect("127.0.0.1:1234").await.unwrap();
    let mut connection = Connection::new(stream);
    let frame: Frame<String> = Frame::new_set_request("set".into(),Data::String("{'hi':true}".into()),Some(Data::Array(vec![Data::UInteger(12),Data::UInteger(1782)])));
    connection.write_frame(frame).await.unwrap();
}