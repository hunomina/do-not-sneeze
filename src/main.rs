// https://datatracker.ietf.org/doc/html/rfc1035

use std::{net::UdpSocket, thread};

use decoder::{Decoder, MessageDecoder};
use transport::{DNS_PORT, UDP_MAX_MESSAGE_SIZE};

mod common;
mod decoder;
mod resource_data;
mod transport;
mod utils;

fn main() {
    // let socket = UdpSocket::bind(format!("0.0.0.0:{}", DNS_PORT)).unwrap();
    //
    // let mut buf = [0; UDP_MAX_MESSAGE_SIZE / 8];
    // loop {
    //     match socket.recv_from(&mut buf) {
    //         Ok((amt, src)) => {
    //             thread::spawn(move || {
    //                 println!("from: {}", src);
    //                 println!("message: {:?}", Message::from(&buf[..amt]));
    //                 src.answer(...)
    //             });
    //         }
    //         Err(e) => {
    //             println!("couldn't receive a datagram: {}", e);
    //         }
    //     }
    // }

    let request: &[u8] = &[
        226, 44, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, // header
        6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm', 0, // question domain name
        0, 1, // question type
        0, 1, // question class
    ];
    send_dns_request_to(request, format!("8.8.8.8:{}", DNS_PORT))
}

fn send_dns_request_to(request: &[u8], to: String) {
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", DNS_PORT)).unwrap();
    socket.connect(to).unwrap();
    socket.send(request).unwrap();

    let mut buf = [0; UDP_MAX_MESSAGE_SIZE / 8];

    match socket.recv_from(&mut buf) {
        Ok((amt, _)) => {
            println!("buffer: {:?}", &buf[..amt]);
            println!("message: {:?}", MessageDecoder {}.decode(&buf[..amt]));
        }
        Err(e) => {
            println!("couldn't receive a datagram: {}", e);
        }
    }
}
