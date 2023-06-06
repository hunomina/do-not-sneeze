// https://datatracker.ietf.org/doc/html/rfc1035

use std::{net::UdpSocket, sync::Arc, thread};

use decoder::{Decoder, MessageDecoder};
use transport::{DNS_PORT, UDP_MAX_MESSAGE_SIZE};

use crate::storage::{
    InMemoryResourceRecordRepository, IntoResourceRecordRepositoryQuery, ResourceRecordRepository,
};

mod common;
mod decoder;
mod resource_data;
mod storage;
mod transport;
mod utils;

struct Server<D, R>
where
    D: Decoder + Send + Sync + Clone + Copy + 'static,
    R: ResourceRecordRepository + Send + Sync + Clone  + 'static,
{
    decoder: D,
    storage: R,
}

impl<D, R> Server<D, R>
where
    D: Decoder + Send + Sync + Clone + Copy + 'static,
    R: ResourceRecordRepository + Send + Sync + Clone  + 'static,
{
    fn run(&self) {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", DNS_PORT)).unwrap();
        // google
        //let fallback_server = UdpSocket::bind(format!("0.0.0.0:{}", 12345)).unwrap();
        //fallback_server
        //    .connect(format!("8.8.8.8:{}", DNS_PORT))
        //    .unwrap();

        let decoder = Arc::new(self.decoder);
        let mut buf = [0; UDP_MAX_MESSAGE_SIZE / 8];
        loop {
            //let sock = socket.try_clone().expect("Failed to clone socket");
            match socket.recv_from(&mut buf) {
                Ok((amt, src)) => {
                    let decoder = Arc::clone(&decoder);
                    let storage = self.storage.clone();
                    thread::spawn(move || {
                        println!("from: {}", src);
                        let message = decoder.decode(&buf[..amt]).unwrap();
                        println!("message: {:?}", message);
                        println!(
                            "questions: {:?}",
                            message
                                .questions
                                .iter()
                                .map(|q| q.into_query())
                                .collect::<Vec<_>>()
                        );
                        //storage.get_resource_records(message.)
                        //src.answer(...)
                        //let google_resp = send_dns_request_to(&buf[..amt], &fallback_server_copy);
                        //sock.send_to(&google_resp, src);
                    });
                }
                Err(e) => {
                    println!("couldn't receive a datagram: {}", e);
                }
            }
        }
    }
}

fn main() {
    Server {
        decoder: MessageDecoder {},
        storage: InMemoryResourceRecordRepository::new(), // todo: this is not Copy because all the sub types are not Copy too.......
    }
    .run();

    // let request: &[u8] = &[
    //     226, 44, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, // header
    //     6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm', 0, // question domain name
    //     0, 1, // question type
    //     0, 1, // question class
    // ];
    // send_dns_request_to(request, format!("8.8.8.8:{}", DNS_PORT))
}

fn send_dns_request_to(request: &[u8], to: &UdpSocket) -> [u8; UDP_MAX_MESSAGE_SIZE / 8] {
    to.send(request).unwrap();

    let mut buf = [0; UDP_MAX_MESSAGE_SIZE / 8];

    // todo: loop until received everything
    // 64 bytes might not be enough
    loop {
        // if amt < UDP_MAX_MESSAGE_SIZE then all has been read
        match to.recv_from(&mut buf) {
            Ok((amt, _)) => {
                println!("amt {:?}", amt);
                println!("buf {:?}", &buf[..amt]);
                //buf[..amt].try_into().unwrap()
            }
            Err(e) => {
                panic!("couldn't receive a datagram: {}", e);
            }
        }
    }
}
