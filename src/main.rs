// https://datatracker.ietf.org/doc/html/rfc1035

use std::net::UdpSocket;

use decoder::MessageDecoder;
use encoder::MessageEncoder;
use server::Server;
use transport::UDP_MAX_MESSAGE_SIZE;

use crate::storage::InMemoryResourceRecordRepository;

mod common;
mod decoder;
mod encoder;
mod resource_data;
mod server;
mod storage;
mod transport;
mod utils;

fn main() {
    Server::new(
        MessageDecoder {},
        MessageEncoder {},
        InMemoryResourceRecordRepository::new(),
    )
    .run();
}

fn send_dns_request_to(request: &[u8], to: &UdpSocket) -> [u8; UDP_MAX_MESSAGE_SIZE / 8] {
    to.send(request).unwrap();

    let mut buf = [0; UDP_MAX_MESSAGE_SIZE / 8];

    // todo: loop until received everything, not only the first UDP_MAX_MESSAGE_SIZE bits
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
