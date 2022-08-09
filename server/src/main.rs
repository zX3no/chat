use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    thread,
    time::Duration,
};

#[derive(Debug, Clone, PartialEq, Eq)]
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

    // println!(
    //     "{} sent message: '{}'",
    //     stream.peer_addr().unwrap(),
    //     from_utf8(&packet).unwrap()
    // );

    let mut message = packet_size.to_vec();
    message.extend(packet);

    Ok(message)
}

fn client_thread(mut stream: TcpStream) {
    stream
        .set_read_timeout(Some(Duration::from_millis(1000)))
        .unwrap();

    let mut channel = Channel::new();
    let ip = stream.peer_addr().unwrap();

    loop {
        if let Ok(msg) = read_message(&mut stream) {
            channel.send(Event::Message((ip, msg)));
        }

        if let Some(event) = channel.try_recv() {
            // println!("{:?}", event);
            match event {
                Event::Message((other_ip, msg)) if &ip != other_ip => {
                    // println!("Sending message to client! {}", ip);
                    match stream.write_all(msg) {
                        Ok(_) => (),
                        //Close the thread.
                        Err(err) => return println!("Warning: {}", err),
                    }
                }
                _ => (),
            }
        }
    }
}

static mut EVENT_COUNT: usize = 0;
static mut CHANNEL: Vec<Event> = Vec::new();

#[derive(Default)]
pub struct Channel {
    items_read: usize,
}

impl Channel {
    pub fn new() -> Self {
        unsafe {
            Self {
                items_read: EVENT_COUNT,
            }
        }
    }
    pub fn send(&mut self, event: Event) {
        unsafe {
            CHANNEL.push(event);
            EVENT_COUNT += 1;

            if CHANNEL.len() == 10 {
                CHANNEL.remove(0);
            }
        }
    }
    pub fn try_recv(&mut self) -> Option<&Event> {
        unsafe {
            if self.items_read != EVENT_COUNT {
                let dif = EVENT_COUNT - self.items_read;
                let len = CHANNEL.len();
                let event = &CHANNEL[len - dif];
                self.items_read += 1;
                Some(event)
            } else {
                None
            }
        }
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
