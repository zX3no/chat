use std::{
    io::{self, stdin, stdout, BufRead, Read, Write},
    net::TcpStream,
    str::from_utf8,
    sync::mpsc,
    thread,
    time::Duration,
};

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

fn read_message(stream: &mut TcpStream) -> io::Result<String> {
    stream.set_read_timeout(Some(Duration::from_millis(10)))?;
    let mut packet_size = [0; 2];
    stream.read_exact(&mut packet_size)?;

    let size = u16::from_le_bytes(packet_size);

    let mut packet = vec![0; size as usize];
    stream.read_exact(&mut packet).unwrap();

    let message = from_utf8(&packet).unwrap().to_string();
    Ok(message)
}

fn main() {
    let (send, recv) = mpsc::channel();

    //Input thread
    thread::spawn(move || {
        let mut stdin = stdin().lock();
        let mut stdout = stdout().lock();
        loop {
            print!("> ");
            stdout.flush().unwrap();

            //Read user input
            let mut buf = String::new();
            let _ = stdin.read_line(&mut buf);
            let msg = buf.trim_end().to_string();
            send.send(msg).unwrap();
        }
    });

    let mut stream = TcpStream::connect("127.0.0.1:7777").unwrap();
    loop {
        if let Ok(msg) = recv.try_recv() {
            send_msg(&msg, &mut stream);
        }

        if let Ok(msg) = read_message(&mut stream) {
            println!("Server Sent: {}", msg);
        }
    }
}
