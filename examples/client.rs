use my_redis::connection::Connection;
use my_redis::protocol::Frame;

#[tokio::main]
async fn main() {
    let mut connection = Connection::new("127.0.0.1:1234").await.unwrap();
    let r = connection.sendrecv(Frame::new_get_request("lav", None)).await.unwrap();
    println!("{:?}", r);
}