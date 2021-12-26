#[macro_use]
extern crate log;
extern crate base64;
use std::net::{TcpListener, TcpStream};
use std::io::{BufReader, BufRead, Error, ErrorKind, Write, stdin};
use std::thread;
use env_logger::Env;

fn get_data(stream: &TcpStream) -> std::io::Result<String> {
    let mut reader = BufReader::new(stream);
    let mut buf = String::new();
    reader.read_line(&mut buf)?;
    debug!("Received {} bytes of data from {}.", buf.len(), stream.peer_addr()?);

    let b64_data: Vec<u8>;
    match base64::decode(&buf.trim()) {
        Ok(d) => b64_data = d,
        Err(e) => {
            return Err(Error::new(ErrorKind::InvalidData, format!("BASE64 {}", e)));
        }
    }

    Ok(String::from_utf8(b64_data).unwrap())
}

fn send_data(mut stream: &TcpStream, data: &[u8]) -> std::io::Result<()> {
    let mut encoded_data = base64::encode(data);
    encoded_data.push_str("\n");
    stream.write(encoded_data.as_bytes())?;
    Ok(())
}


fn handle_client(stream: TcpStream) -> std::io::Result<()>{
    let stream_clone = stream.try_clone().expect("Stream clone failed.");
    thread::spawn(move || {
        loop {
            if let Ok(data_received) = get_data(&stream_clone) {
                if !data_received.is_empty() {
                    print!("{}", data_received);
                }
            } else {
                break;
            }
        }
    } );

    loop {
        let mut command = String::new();
        let stdin  = stdin();
        stdin.read_line(&mut command).unwrap();
        send_data(&stream, command.as_bytes())?
    }
}

fn main() -> std::io::Result<()> {
    let env = Env::default()
        .default_filter_or("info");
    env_logger::init_from_env(env);

    info!("Starting server...");
    let listener = TcpListener::bind("127.0.0.1:8000")?;
    info!("Server listening on {}.", listener.local_addr().unwrap());

    for stream in listener.incoming() {
        info!("Client {} connected", stream.as_ref().unwrap().peer_addr()?);
        match handle_client(stream?) {
            Ok(_) => info!("Closing connection."),
            Err(e) => {
                error!("{}. Closing connection.", e);
            }
        }
    }
    Ok(())
}