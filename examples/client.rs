use my_redis::connection::Connection;
use my_redis::protocol::Data;
use my_redis::protocol::Frame;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut connection = Connection::to("127.0.0.1:1234").await?;

    loop {
        let mut buf = String::new();
        println!("Please enter command type (Get smt|Set smt smt):");
        std::io::stdin().read_line(&mut buf)?;
        let mut command = buf.trim().split(' ');
        let name = command.next().unwrap();

        let frame = if name == "GET" {
            Frame::new_get_request(command.next().unwrap().to_string(), None)
        } else if name == "SET" {
            Frame::new_set_request(
                command.next().unwrap().to_string(),
                Data::String(command.next().unwrap().to_string()),
                None,
            )
        } else { break; };

        let resp = connection.sendrecv(frame).await;
        println!("{:?}", resp);
    }

    Ok(())
}