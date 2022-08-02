use std::{
    io::{stdin, BufRead, Read, StdinLock, Write},
    net::{Shutdown, TcpStream},
    str::from_utf8,
};

fn _read_message(stream: &mut TcpStream) {
    let mut packet_size = [0; 2];
    stream.read_exact(&mut packet_size).unwrap();

    let size = u16::from_le_bytes(packet_size);

    let mut packet = vec![0; size as usize];
    stream.read_exact(&mut packet).unwrap();

    println!("Server sent: {}", from_utf8(&packet).unwrap());

    stream.shutdown(Shutdown::Both).unwrap();
}

fn send_msg(msg: &str, stream: &mut TcpStream) {
    if msg.is_empty() {
        return;
    }

    //Get the size of the string.
    //Keep in mind if the size is larger than a u16 it will clip to 65535
    let size = msg.len() as u16;

    //Convert the size to little endian bytes to be send.
    let size_bytes = size.to_le_bytes();

    //Send the string size as little endian bytes
    stream.write_all(&size_bytes).unwrap();

    //Now we send the string
    stream.write_all(msg.as_bytes()).unwrap();
}

fn get_message(stdin: &mut StdinLock) -> String {
    let mut buf = String::new();
    let _ = stdin.read_line(&mut buf);
    buf.trim_end().to_string()
}

fn main() {
    // _send_message();
    let mut stdin = stdin().lock();
    let mut stream = TcpStream::connect("127.0.0.1:7777").unwrap();

    loop {
        let msg = get_message(&mut stdin);
        send_msg(&msg, &mut stream);
    }
}
