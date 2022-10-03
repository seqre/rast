use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

fn handle_client(mut stream: TcpStream) {
    let mut buf = [0; 128];

    match stream.read(&mut buf) {
        Ok(bytes_read) => {
            if bytes_read > 0 {
                println!("{}", String::from_utf8_lossy(&buf));
            }
        }
        Err(e) => println!("Error: {e:?}"),
    }

    let msg = b"Pong!";
    let _ = stream.write(msg).unwrap();
}

fn main() -> std::io::Result<()> {
    println!("Hello from server!");

    let listener = TcpListener::bind("127.0.0.1:42069")?;

    match listener.accept() {
        Ok((stream, addr)) => {
            println!("Got connection from: {addr:?}");
            handle_client(stream);
        }
        Err(e) => println!("Error: {e:?}"),
    }
    Ok(())
}