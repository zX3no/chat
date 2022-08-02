use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    str::from_utf8,
    thread,
    time::Duration,
};

use crossbeam_channel::{unbounded, Receiver, Sender};

#[derive(Debug)]
enum Event {
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

fn client_thread(mut stream: TcpStream, send: Sender<Event>, recv: Receiver<Event>) {
    stream
        .set_read_timeout(Some(Duration::from_millis(1000)))
        .unwrap();

    let ip = stream.peer_addr().unwrap();

    loop {
        if let Ok(msg) = read_message(&mut stream) {
            send.send(Event::Message((ip, msg))).unwrap();
        }

        if let Ok(event) = recv.try_recv() {
            println!("{:?}", event);
            match event {
                Event::Message((other_ip, msg)) if ip != other_ip => {
                    println!("Sending message to client! {}", ip);
                    match stream.write_all(&msg) {
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

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7777").unwrap();

    // thread::spawn(move || {
    //     let mut clients = HashMap::new();
    //     loop {
    //         if let Ok(c) = recv.try_recv() {
    //             let ip = c.peer_addr().unwrap();
    //             clients.insert(ip, c);
    //         }

    //         let mut messages = Vec::new();

    //         for (k, v) in &mut clients {
    //             if let Ok(msg) = read_message(v) {
    //                 messages.push((*k, msg));
    //             }
    //         }

    //         let mut delete = Vec::new();

    //         for (ip, msg) in messages {
    //             for (k, v) in &mut clients {
    //                 if &ip == k {
    //                     continue;
    //                 }

    //                 println!("Sending message to client! {}", k);
    //                 match v.write_all(&msg) {
    //                     Ok(_) => (),
    //                     Err(err) => {
    //                         println!("Warning: {}", err);
    //                         delete.push(ip)
    //                     }
    //                 };
    //             }
    //         }

    //         //Delete the broken clients
    //         for ip in delete {
    //             println!("Dropping connection to {}", ip);
    //             clients.remove(&ip);
    //         }
    //     }
    // });

    let (send, recv) = unbounded();

    for stream in listener.incoming() {
        let client = stream.unwrap();
        println!("Client connected: {}", client.peer_addr().unwrap());

        let send = send.clone();
        let recv = recv.clone();
        thread::spawn(|| client_thread(client, send, recv));
    }
}
