use std::{
    io::{Read, Write},
    net::TcpStream,
};

fn main() -> std::io::Result<()> {
    println!("Hello from client!");

    let mut stream = TcpStream::connect("127.0.0.1:42069")?;

    let msg = b"Ping!";
    let _ = stream.write(msg)?;

    let mut buf = [0; 128];
    match stream.read(&mut buf) {
        Ok(bytes_read) => {
            if bytes_read > 0 {
                println!("{}", String::from_utf8_lossy(&buf));
            }
        }
        Err(e) => println!("Error: {e:?}"),
    }

    Ok(())
}
