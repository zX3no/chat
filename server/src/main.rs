#![allow(clippy::new_without_default)]
use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    str::from_utf8,
    thread,
    time::Duration,
};

#[derive(Debug)]
pub enum Event {
    Message((SocketAddr, Vec<u8>)),
}

fn read_message(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    stream.set_read_timeout(Some(Duration::from_millis(10)))?;

    //Create an array with a size of 2.
    let mut packet_size = [0; 2];
    //Read exactly 2 bytes into the buffer.
    stream.read_exact(&mut packet_size)?;

    //Convert the two bytes into a u16
    let size = u16::from_le_bytes(packet_size);

    //Create a dynamic array on the heap aka a vector.
    let mut packet = vec![0; size as usize];

    //Fill up the packet to the size previously sent.
    stream.read_exact(&mut packet).unwrap();

    println!(
        "{} sent message: '{}'",
        stream.peer_addr().unwrap(),
        from_utf8(&packet).unwrap()
    );

    let mut message = packet_size.to_vec();
    message.extend(packet);

    Ok(message)
}

fn client_thread(mut stream: TcpStream) {
    stream
        .set_read_timeout(Some(Duration::from_millis(1000)))
        .unwrap();

    let (mut send, mut recv) = channel();
    let ip = stream.peer_addr().unwrap();

    loop {
        if let Ok(msg) = read_message(&mut stream) {
            send.send(Event::Message((ip, msg)));
        }

        if let Some(event) = recv.try_recv() {
            println!("{:?}", event);
            match event {
                Event::Message((other_ip, msg)) if &ip != other_ip => {
                    println!("Sending message to client! {}", ip);
                    match stream.write_all(msg) {
                        Ok(_) => (),
                        //Close the thread.
                        Err(err) => return println!("Warning: {}", err),
                    }
                }
                Event::Message((other_ip, _)) => println!("Did not send: {} {}", ip, other_ip),
            }
        }
    }
}

static mut CHANNEL: Vec<Event> = Vec::new();

pub fn channel() -> (Sender, Receiver) {
    (Sender::new(), Receiver::new())
}

pub struct Sender {
    pub pos: usize,
}

impl Sender {
    pub fn new() -> Self {
        Self { pos: 0 }
    }
    pub fn send(&mut self, event: Event) {
        unsafe {
            CHANNEL.push(event);

            if CHANNEL.len() > 10 {
                CHANNEL.remove(0);
            }
            self.pos = CHANNEL.len()
        }
    }
}

pub struct Receiver {
    pub pos: usize,
}

impl Receiver {
    pub fn new() -> Self {
        unsafe { Self { pos: CHANNEL.len() } }
    }
    pub fn try_recv(&mut self) -> Option<&Event> {
        let event = unsafe { CHANNEL.get(self.pos) };
        if event.is_some() {
            self.pos += 1;
        }
        event
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7777").unwrap();

    for stream in listener.incoming() {
        let client = stream.unwrap();
        println!("Client connected: {}", client.peer_addr().unwrap());

        thread::spawn(|| client_thread(client));
    }
}
