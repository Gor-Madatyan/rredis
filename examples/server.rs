use my_redis::connection::Connection;
use my_redis::protocol::handler::Handler;
use my_redis::protocol::{Data, NetworkFrame, RRError};

struct MyHandler {}

impl Clone for MyHandler {
    fn clone(&self) -> Self {
        MyHandler {}
    }
}

impl Handler for MyHandler {
    async fn handle_set_request(&mut self, key: String, value: Data, payload: Option<Data>) -> Result<NetworkFrame, RRError> {
        println!("Handling set request with key: {}, value: {:?} payload: {:?}", key, value, payload);
        Ok(NetworkFrame::new_data_request(Data::NULL, None))
    }

    async fn handle_get_request(&mut self, key: String, payload: Option<Data>) -> Result<NetworkFrame, RRError> {
        println!("Handling get request with key: {:?} payload: {:?}", key, payload);
        Ok(NetworkFrame::new_data_request(Data::NULL, None))
    }
}

#[tokio::main]
async fn main() {
    if let Err(app) = Connection::app("127.0.0.1:1234", MyHandler {}).await {
        println!("{:?}", app);
        return;
    }
}