use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 512];

    match stream.read(&mut buffer) {
        Ok(_) => {
            let request = String::from_utf8_lossy(&buffer[..]);

            println!("Request: {}", request);

            if request.lines().nth(2).unwrap_or_default() == "ping" {
                let response = "+PONG\r\n";
                match stream.write(response.as_bytes()) {
                    Ok(_) => println!("Sent response to client"),
                    Err(e) => eprintln!("Failed to send response to client: {}", e),
                }
            }
        }
        Err(e) => eprintln!("Failed to read from client: {}", e),
    }
}

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("New client connected");
                handle_client(_stream);
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }
}
