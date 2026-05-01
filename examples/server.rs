use tokio::net::{TcpListener, TcpStream};
use my_redis::connection::Connection;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:1234").await.unwrap();
    loop {
        let (listener,_) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            process(listener).await;
        });
    }
}

async fn process(stream:TcpStream){
    let mut connection = Connection::new(stream);
    loop {
        if let Ok(frame) = connection.read_frame().await {
            println!("{:?}", frame);
        } else {break;}
    }
}