use std::net::TcpStream;
use std::io::{Read, Write};

fn main() -> std::io::Result<()> {
    println!("Hello from client!");

    let mut stream = TcpStream::connect("127.0.0.1:42069")?;

    let msg = b"Ping!";
    stream.write(msg)?;

    let mut buf = [0; 128];
    match stream.read(&mut buf) {
        Ok(bytes_read) => {
            if bytes_read > 0 {
                println!("{}", String::from_utf8_lossy(&buf));
            }
        },
        Err(e) => println!("Error: {e:?}"),
    }

    Ok(())
}
