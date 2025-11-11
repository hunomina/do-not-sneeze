use std::{
    env,
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
        let port = env::var("DNS_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(DNS_PORT);
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).unwrap();

        println!("🚀 DNS server running on port {}", port);

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
                        let buffer = &buf[..amt];
                        let encoded_response = Self::handle(buffer, decoder, encoder, storage);

                        socket_clone.send_to(&encoded_response, src).unwrap();
                    });
                }
                Err(e) => {
                    println!("couldn't receive a datagram: {}", e);
                }
            }
        }
    }

    fn handle(buffer: &[u8], decoder: Arc<D>, encoder: Arc<E>, storage: Arc<Mutex<R>>) -> Vec<u8> {
        let message = decoder.decode(buffer).unwrap();

        println!(
            "👾 Received message: questions {:?}; EDNS: {}",
            message.questions,
            message.opt_record.is_some()
        );

        let max_message_size = message.max_message_size();

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

        let mut encoded_response = encoder.encode(response.clone());
        let encoded_response_len = encoded_response.len() * 8;

        if encoded_response_len > max_message_size {
            println!(
                "⚠️ Encoded message size ({}) exceeds max message size ({}). Truncating.",
                encoded_response_len, max_message_size
            );
            response = response.truncate();
            encoded_response = encoder.encode(response);
        }

        encoded_response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        common::{
            Message,
            domain_name::DomainName,
            header::{Header, MessageType, QueryType, ResponseCode},
            opt_record::OptRecord,
            question::{Class, Question, Type},
            resource_record::{ResourceRecord, Type as RRType},
        },
        decoder::{Decoder, DecodingError},
        encoder::Encoder,
        storage::{RepositoryError, ResourceRecordRepository},
    };

    const MOCKED_HEADER_SIZE: usize = 20;
    const MOCKED_QUESTIONS_SIZE: usize = 20;

    struct MockEncoder {
        bytes_per_record: usize,
    }

    impl Encoder for MockEncoder {
        fn encode(&self, message: Message) -> Vec<u8> {
            let mocked_answer_size = MOCKED_QUESTIONS_SIZE
                + message.answers.len() * self.bytes_per_record
                + message.authorities.len() * self.bytes_per_record
                + message.additionnals.len() * self.bytes_per_record;
            vec![0u8; MOCKED_HEADER_SIZE + mocked_answer_size]
        }
    }

    struct MockDecoder;

    impl Decoder for MockDecoder {
        fn decode(&self, _buffer: &[u8]) -> Result<Message, DecodingError> {
            Ok(Message::new(
                Header {
                    id: 1234,
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
                vec![Question {
                    name: DomainName::from("example.com."),
                    type_: Type::RRType(RRType::A),
                    class: Class::IN,
                }],
                vec![],
                vec![],
                vec![],
                None,
            ))
        }
    }

    struct MockDecoderWithEDNS;

    impl Decoder for MockDecoderWithEDNS {
        fn decode(&self, _buffer: &[u8]) -> Result<Message, DecodingError> {
            Ok(Message::new(
                Header {
                    id: 1234,
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
                    additional_count: 1,
                },
                vec![Question {
                    name: DomainName::from("example.com."),
                    type_: Type::RRType(RRType::A),
                    class: Class::IN,
                }],
                vec![],
                vec![],
                vec![],
                Some(OptRecord {
                    udp_payload_size: 4096,
                    extended_rcode: 0,
                    version: 0,
                    dnssec_ok: false,
                    options: vec![],
                }),
            ))
        }
    }

    struct MockStorage {
        records_to_return: Vec<ResourceRecord>,
    }

    impl ResourceRecordRepository for MockStorage {
        fn get_resource_records(
            &mut self,
            _question: Question,
        ) -> Result<Vec<ResourceRecord>, RepositoryError> {
            Ok(self.records_to_return.clone())
        }
    }

    #[test]
    fn handle_regular_request() {
        let decoder = Arc::new(MockDecoder);

        const MOCKED_ANSWER_SIZE: usize = 10;
        let encoder = Arc::new(MockEncoder {
            bytes_per_record: MOCKED_ANSWER_SIZE, // Small size per answer
        });

        let mocked_answers = vec![
            build_type_a_record("example.com.", "192.0.2.1"),
            build_type_a_record("example.com.", "192.0.2.2"),
        ];
        let mocked_answers_len = mocked_answers.len();
        let storage = Arc::new(Mutex::new(MockStorage {
            records_to_return: mocked_answers,
        }));

        let response = Server::<MockDecoder, MockEncoder, MockStorage>::handle(
            &[0u8; UDP_MAX_MESSAGE_SIZE / 8],
            decoder,
            encoder,
            storage,
        );

        assert_eq!(
            response.len(),
            MOCKED_HEADER_SIZE + MOCKED_QUESTIONS_SIZE + mocked_answers_len * MOCKED_ANSWER_SIZE
        );
    }

    #[test]
    fn handle_truncated_request() {
        let decoder = Arc::new(MockDecoder);

        let encoder = Arc::new(MockEncoder {
            bytes_per_record: 50,
        });

        // 2 records size = MOCKED_HEADER_SIZE + MOCKED_QUESTIONS_SIZE + 2 * MOCKED_ANSWER_SIZE = 140 bytes
        // 140 * 8 = 1120 bits > 512 (DNS default UDP limit)
        let mocked_answers = vec![
            build_type_a_record("example.com.", "192.0.2.1"),
            build_type_a_record("example.com.", "192.0.2.2"),
        ];
        let storage = Arc::new(Mutex::new(MockStorage {
            records_to_return: mocked_answers,
        }));

        let response = Server::<MockDecoder, MockEncoder, MockStorage>::handle(
            &[0u8; UDP_MAX_MESSAGE_SIZE / 8],
            decoder,
            encoder,
            storage,
        );

        assert_eq!(response.len(), MOCKED_HEADER_SIZE + MOCKED_QUESTIONS_SIZE);
    }

    #[test]
    fn handle_with_edns_larger_limit() {
        let decoder = Arc::new(MockDecoderWithEDNS);

        const MOCKED_ANSWER_SIZE: usize = 50;
        let encoder = Arc::new(MockEncoder {
            bytes_per_record: MOCKED_ANSWER_SIZE,
        });

        // 3 records size = MOCKED_HEADER_SIZE + MOCKED_QUESTIONS_SIZE + 3 * MOCKED_ANSWER_SIZE = 190 bytes
        // 190 * 8 = 1520 bits < 4096 (EDNS limit)
        let mocked_answers = vec![
            build_type_a_record("example.com.", "192.0.2.1"),
            build_type_a_record("example.com.", "192.0.2.2"),
            build_type_a_record("example.com.", "192.0.2.3"),
        ];
        let mocked_answers_len = mocked_answers.len();

        let storage = Arc::new(Mutex::new(MockStorage {
            records_to_return: mocked_answers,
        }));

        let response = Server::<MockDecoderWithEDNS, MockEncoder, MockStorage>::handle(
            &[0u8; UDP_MAX_MESSAGE_SIZE / 8],
            decoder,
            encoder,
            storage,
        );

        assert_eq!(
            response.len(),
            MOCKED_HEADER_SIZE + MOCKED_QUESTIONS_SIZE + mocked_answers_len * MOCKED_ANSWER_SIZE
        );
    }

    #[test]
    fn handle_with_edns_still_truncated() {
        let decoder = Arc::new(MockDecoderWithEDNS);

        const MOCKED_ANSWER_SIZE: usize = 1000;
        let encoder = Arc::new(MockEncoder {
            bytes_per_record: MOCKED_ANSWER_SIZE,
        });

        // 3 records size = MOCKED_HEADER_SIZE + MOCKED_QUESTIONS_SIZE + 3 * MOCKED_ANSWER_SIZE = 3040 bytes
        // 3040 * 8 = 24320 bits > 4096 (EDNS limit)
        let mocked_answers = vec![
            build_type_a_record("example.com.", "192.0.2.1"),
            build_type_a_record("example.com.", "192.0.2.2"),
            build_type_a_record("example.com.", "192.0.2.3"),
        ];

        let storage = Arc::new(Mutex::new(MockStorage {
            records_to_return: mocked_answers,
        }));

        let response = Server::<MockDecoderWithEDNS, MockEncoder, MockStorage>::handle(
            &[0u8; UDP_MAX_MESSAGE_SIZE / 8],
            decoder,
            encoder,
            storage,
        );

        assert_eq!(response.len(), MOCKED_HEADER_SIZE + MOCKED_QUESTIONS_SIZE);
    }

    fn build_type_a_record(name: &str, ip: &str) -> ResourceRecord {
        ResourceRecord::new(
            DomainName::from(name),
            RRType::A,
            Class::IN,
            300,
            ip.parse::<std::net::Ipv4Addr>().unwrap().octets().to_vec(),
        )
    }
}
