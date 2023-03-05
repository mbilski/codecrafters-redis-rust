use tokio::net::TcpListener;

use redis_starter_rust::server;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    println!("Listening on {}", listener.local_addr().unwrap());

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                println!("New client connected");

                tokio::spawn(async move {
                    match server::run(stream).await {
                        Ok(_) => println!("Client disconnected"),
                        Err(e) => eprintln!("error handling connection: {}", e),
                    }
                });
            }
            Err(e) => {
                eprintln!("error accepting connection: {}", e);
            }
        }
    }
}
