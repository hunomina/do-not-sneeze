use crate::{
    common::{
        Message,
        header::{Header, MessageType, QueryType, ResponseCode},
        question::Question,
        resource_record::ResourceRecord,
    },
    decoder::Decoder,
    encoder::Encoder,
    storage::{RepositoryError, ResourceRecordRepository},
    transport::EDNS_STANDARD_UDP_PAYLOAD_SIZE,
};

use std::{
    io::Error,
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

        response_from_fallback_server.map(|m| m.answers)
    }
}

fn fetch_from_other_server<T: ToSocketAddrs + Clone, D: Decoder, E: Encoder>(
    encoder: &E,
    decoder: &D,
    fallback_server_address: T,
    message: Message,
) -> Result<Message, RepositoryError> {
    let mut buf = [0; EDNS_STANDARD_UDP_PAYLOAD_SIZE / 8]; // could be improved by only allocating based on if EDNS is enabled
    let encode_message = encoder.encode(message);

    UdpSocket::bind("0.0.0.0:0")
        .and_then(|socket| {
            socket.connect(fallback_server_address)?;
            socket.send(encode_message.as_slice())?;
            socket.recv_from(&mut buf)
        })
        .map_err(|e: Error| RepositoryError::ContactingFallbackServerError(e.to_string()))
        .and_then(|(amt, _)| {
            decoder
                .decode(&buf[..amt])
                .map_err(RepositoryError::DecodingFallbackServerResponseError)
        })
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
