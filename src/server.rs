use std::{
    net::UdpSocket,
    sync::{Arc, Mutex},
    thread,
};

use crate::{
    decoder::Decoder,
    encoder::Encoder,
    storage::ResourceRecordRepository,
    transport::{DNS_PORT, UDP_MAX_MESSAGE_SIZE},
};

pub struct Server<D, E, R>
where
    D: Decoder + Send + Sync + 'static,
    E: Encoder + Send + Sync + 'static,
    R: ResourceRecordRepository + Send + 'static,
{
    decoder: D,
    encoder: E,
    storage: R,
}

impl<D, E, R> Server<D, E, R>
where
    D: Decoder + Send + Sync + 'static,
    E: Encoder + Send + Sync + 'static,
    R: ResourceRecordRepository + Send + 'static,
{
    pub fn new(decoder: D, encoder: E, storage: R) -> Self {
        Server {
            decoder,
            encoder,
            storage,
        }
    }

    pub fn run(self) {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", DNS_PORT)).unwrap();

        let decoder = Arc::new(self.decoder);
        let encoder = Arc::new(self.encoder);
        let storage = Arc::new(Mutex::new(self.storage));

        let mut buf = [0; UDP_MAX_MESSAGE_SIZE / 8];
        loop {
            let socket_clone = socket.try_clone().unwrap();
            match socket.recv_from(&mut buf) {
                Ok((amt, src)) => {
                    let decoder = Arc::clone(&decoder);
                    let encoder = Arc::clone(&encoder);
                    let storage = Arc::clone(&storage);

                    thread::spawn(move || {
                        let message = decoder.decode(&buf[..amt]).unwrap();

                        let answers = message
                            .questions
                            .iter()
                            .flat_map(|question| {
                                storage
                                    .lock()
                                    .unwrap()
                                    .get_resource_records(question.clone())
                                    .unwrap_or_default()
                            })
                            .collect::<Vec<_>>();

                        let mut response = message.into_response();
                        response.set_answers(answers);

                        let encoded_response = encoder.encode(response);

                        // todo: check if encoded_response is bigger than UDP_MAX_MESSAGE_SIZE + check if EDNS is enabled
                        // if it is the case, we need to send an empty response

                        socket_clone.send_to(&encoded_response, src).unwrap();
                    });
                }
                Err(e) => {
                    println!("couldn't receive a datagram: {}", e);
                }
            }
        }
    }
}
