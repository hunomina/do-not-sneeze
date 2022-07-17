// https://datatracker.ietf.org/doc/html/rfc1035

use std::{net::UdpSocket, thread};

use message::Message;
use transport::{DNS_PORT, UDP_MAX_MESSAGE_SIZE};

mod domain_name;
mod message;
mod resource_record;
mod transport;
mod utils;

// use std::io::prelude::*;
// use std::net::{TcpListener, TcpStream};
//
// fn main() {
//     let listener = TcpListener::bind(format!("0.0.0.0:{}", DNS_PORT)).unwrap();
//
//     for stream in listener.incoming() {
//         let stream = stream.unwrap();
//         println!("Connection established!");
//
//         handle_connection(stream)
//     }
// }
//
// fn handle_connection(mut stream: TcpStream) {
//     let mut buffer = [0; 1024];
//
//     stream.read(&mut buffer).unwrap();
//
//     println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
// }

fn main() {
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", DNS_PORT)).unwrap();

    let mut buf = [0; UDP_MAX_MESSAGE_SIZE / 8];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                thread::spawn(move || {
                    println!("from: {}", src);
                    println!("message: {:?}", Message::from(&buf[..amt]));
                });
            }
            Err(e) => {
                println!("couldn't receive a datagram: {}", e);
            }
        }
    }
}
