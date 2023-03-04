use std::{
    io::{self, prelude::*, BufReader, ErrorKind},
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    println!("Listening on {}", listener.local_addr().unwrap());

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New client connected");
                match handle_connection(stream) {
                    Ok(_) => println!("Client disconnected"),
                    Err(e) => eprintln!("Error handling client: {}", e),
                }
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) -> Result<(), io::Error> {
    loop {
        let reader = BufReader::new(&mut stream);

        let request = read_request(reader)?;

        dbg!(request);

        let response = "+PONG\r\n";

        stream.write_all(response.as_bytes())?
    }
}

fn read_request(reader: BufReader<&mut TcpStream>) -> Result<Vec<String>, io::Error> {
    let mut args = Vec::new();

    let mut lines = reader.lines();

    let header = lines
        .next()
        .and_then(|r| r.ok())
        .ok_or(io::Error::new(ErrorKind::Other, "No header"))?;

    let number_of_parameters = header[1..header.len()].parse::<usize>().map_err(|_| {
        io::Error::new(
            ErrorKind::Other,
            "Failed to parse number of parameters from header",
        )
    })?;

    (0..number_of_parameters).for_each(|_| {
        lines.next();

        if let Some(l) = lines.next().and_then(|r| r.ok()) {
            args.push(l)
        }
    });

    Ok(args)
}
