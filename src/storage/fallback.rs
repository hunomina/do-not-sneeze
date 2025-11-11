use crate::{
    common::{
        Message,
        header::{Header, MessageType, QueryType, ResponseCode},
        question::Question,
        resource_record::ResourceRecord,
    },
    decoder::{Decoder, DecodingError},
    encoder::Encoder,
    storage::{RepositoryError, ResourceRecordRepository},
    transport::EDNS_STANDARD_UDP_PAYLOAD_SIZE,
};

use std::{
    net::{ToSocketAddrs, UdpSocket},
    time,
};

pub struct FallbackRepository<T: ToSocketAddrs + Clone, D: Decoder, E: Encoder> {
    pub fallback_server_address: T,
    pub decoder: D,
    pub encoder: E,
}

impl<T: ToSocketAddrs + Clone, D: Decoder, E: Encoder> ResourceRecordRepository
    for FallbackRepository<T, D, E>
{
    fn get_resource_records(
        &mut self,
        question: Question,
    ) -> Result<Vec<ResourceRecord>, RepositoryError> {
        //println!("Question to fallback server: {:?}", question);

        let response_from_fallback_server = fetch_from_other_server(
            &self.encoder,
            &self.decoder,
            self.fallback_server_address.clone(),
            generate_message_with_question(question),
        );

        //println!(
        //    "Response from fallback server: {:?}",
        //    response_from_fallback_server
        //);

        match response_from_fallback_server {
            Ok(message) => Ok(message.answers),
            Err(e) => {
                println!("Error fetching from fallback server: {:?}", e);
                // If we cannot get a response from the fallback server, we return an empty vector
                Ok(vec![]) // todo transform DecodingError into RepositoryError
            }
        }
    }
}

fn fetch_from_other_server<T: ToSocketAddrs + Clone, D: Decoder, E: Encoder>(
    encoder: &E,
    decoder: &D,
    fallback_server_address: T,
    message: Message,
) -> Result<Message, DecodingError> {
    let mut buf = [0; EDNS_STANDARD_UDP_PAYLOAD_SIZE / 8]; // could be improved by only allocating based on if EDNS is enabled
    let encode_message = encoder.encode(message);

    let fallback_server = UdpSocket::bind("0.0.0.0:0").unwrap();
    fallback_server
        .connect(fallback_server_address)
        .expect("Failed to connect to fallback server");

    fallback_server.send(encode_message.as_slice()).unwrap();

    match fallback_server.recv_from(&mut buf) {
        Ok((amt, _)) => decoder.decode(&buf[..amt]),
        Err(e) => {
            panic!("couldn't receive a datagram: {}", e);
        }
    }
}

fn generate_message_with_question(question: Question) -> Message {
    // todo IMPROVEMENT: problem with building our own headers (and message) is that we potentially discard some of the original request properties
    Message::new(
        Header {
            id: time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as u16,
            qr: MessageType::Query,
            opcode: QueryType::Standard,
            authoritative_answer: false,
            truncated: false,
            recursion_desired: true,
            recursion_available: false,
            reserved: false,
            response_code: ResponseCode::NoError,
            questions_count: 1,
            answers_count: 0,
            authority_count: 0,
            additional_count: 0,
        },
        vec![question],
        vec![],
        vec![],
        vec![],
        None,
    )
}
