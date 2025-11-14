mod domain_name;
mod header;
mod opt_record;
mod question;
mod resource_record;

use crate::{
    common::{
        Message,
        header::{HEADER_BIT_SIZE, extract_header_bits_from_buffer},
        question::Question,
        resource_record::{ResourceRecord, Type},
    },
    decoder::domain_name::is_alias_flag,
};

pub trait Decoder {
    fn decode(&self, buffer: &[u8]) -> Result<Message, DecodingError>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum DecodingError {
    InvalidHeaderSize,
    InvalidHeaderQueryType(String),
    InvalidHeaderResponseCode(String),
    InvalidResourceRecordType(String),
    InvalidResourceRecordClass(String),
    InvalidQuestionType(String),
    InvalidQuestionClass(String),
    InvalidOptRecord(String),
    MultipleOptRecords,
    ResourceDataLengthMismatch { expected: usize, actual: usize },
}

pub struct MessageDecoder {}

impl Decoder for MessageDecoder {
    fn decode(&self, buffer: &[u8]) -> Result<Message, DecodingError> {
        let source = buffer;

        let (header_bits, mut buffer) = extract_header_bits_from_buffer(buffer);
        let header_bits: &[u8; HEADER_BIT_SIZE / 8] = header_bits
            .try_into()
            .map_err(|_| DecodingError::InvalidHeaderSize)?;
        let header = header::decode(header_bits)?;

        let questions = (0..header.questions_count)
            .map(|_| {
                // todo: add source to question::decode, we might have multiple questions and thus using alias
                let (question, rest) = question::decode(buffer)?;
                buffer = rest;
                Ok(question)
            })
            .collect::<Result<Vec<Question>, DecodingError>>()?;

        let answers = (0..header.answers_count)
            .map(|_| {
                let (rr, rest) = resource_record::decode(buffer, source)?;
                buffer = rest;
                Ok(rr)
            })
            .collect::<Result<Vec<ResourceRecord>, DecodingError>>()?;

        let authorities = (0..header.authority_count)
            .map(|_| {
                let (rr, rest) = resource_record::decode(buffer, source)?;
                buffer = rest;
                Ok(rr)
            })
            .collect::<Result<Vec<ResourceRecord>, DecodingError>>()?;

        // Decode additional section, checking for OPT record
        let mut additionnals = Vec::new();
        let mut opt_record = None;

        for _ in 0..header.additional_count {
            if is_opt_record(buffer)? {
                if opt_record.is_some() {
                    return Err(DecodingError::MultipleOptRecords);
                }
                let (opt, rest) = opt_record::decode(buffer)?;
                opt_record = Some(opt);
                buffer = rest;
            } else {
                let (rr, rest) = resource_record::decode(buffer, source)?;
                additionnals.push(rr);
                buffer = rest;
            }
        }

        Ok(Message::new(
            header,
            questions,
            answers,
            authorities,
            additionnals,
            opt_record,
        ))
    }
}

fn is_opt_record(buffer: &[u8]) -> Result<bool, DecodingError> {
    let type_value = peek_rr_type(buffer)?;

    Ok(type_value == Type::OPT)
}

/// Peek at the resource record type without consuming the buffer
/// Assumes we're at the start of a resource record
fn peek_rr_type(buffer: &[u8]) -> Result<Type, DecodingError> {
    // Skip domain name to get to type field
    let mut pos = 0;

    // Handle domain name (labels or compression pointers)
    loop {
        if pos >= buffer.len() {
            return Err(DecodingError::InvalidOptRecord(
                "Buffer too short to peek type".to_string(),
            ));
        }

        let byte = buffer[pos];

        if byte == 0 {
            // End of domain name
            pos += 1;
            break;
        } else if is_alias_flag(byte) {
            // Compression pointer (2 bytes)
            pos += 2;
            break;
        } else {
            // Regular label: byte contains the label length
            pos += 1 + byte as usize;
        }
    }

    // Now we're at the type field (2 bytes)
    if pos + 2 > buffer.len() {
        return Err(DecodingError::InvalidOptRecord(
            "Buffer too short for type field".to_string(),
        ));
    }

    let type_value = ((buffer[pos] as u16) << 8) | (buffer[pos + 1] as u16);
    Type::try_from(type_value).map_err(DecodingError::InvalidResourceRecordType)
}

#[cfg(test)]
mod tests {
    use crate::common::{
        domain_name::DomainName,
        header::{Header, MessageType, QueryType, ResponseCode},
        question::{Class, Question, Type},
        resource_record::{ResourceRecord, Type as RRType},
    };

    use super::*;

    const MESSAGE_WITH_ONE_QUESTION: &[u8] = &[
        226, 44, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, // header
        4, b'w', b'p', b'a', b'd', 11, b'n', b'u', b'm', b'e', b'r', b'i', b'c', b'a', b'b', b'l',
        b'e', 2, b'f', b'r', 0, // question domain name
        0, 1, // question type
        0, 1, // question class
    ];

    const MESSAGE_WITH_TWO_QUESTIONS: &[u8] = &[
        226, 44, 1, 0, 0, 2, 0, 0, 0, 0, 0, 0, // header
        // Question 1
        4, b'w', b'p', b'a', b'd', 11, b'n', b'u', b'm', b'e', b'r', b'i', b'c', b'a', b'b', b'l',
        b'e', 2, b'f', b'r', 0, // question 1 domain name
        0, 1, // question 1 type
        0, 1, // question 1 class
        // Question 2
        6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm',
        0, // question 2 domain name
        0, 15, // question 2 type
        0, 255, // question 2 class
    ];

    #[test]
    fn test_message_with_one_question() {
        let decoder = MessageDecoder {};
        let m = decoder.decode(MESSAGE_WITH_ONE_QUESTION);

        let expected_header = Header {
            id: 57900,
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
        };

        let numericable_question = Question {
            name: DomainName {
                labels: vec!["wpad".into(), "numericable".into(), "fr".into(), "".into()],
            },
            type_: Type::RRType(RRType::A),
            class: Class::IN,
        };

        let expected_message = Message::new(
            expected_header,
            vec![numericable_question],
            vec![],
            vec![],
            vec![],
            None,
        );

        assert_eq!(expected_message, m.unwrap());
    }

    #[test]
    fn test_message_with_two_questions() {
        let decoder = MessageDecoder {};

        let expected_header = Header {
            id: 57900,
            qr: MessageType::Query,
            opcode: QueryType::Standard,
            authoritative_answer: false,
            truncated: false,
            recursion_desired: true,
            recursion_available: false,
            reserved: false,
            response_code: ResponseCode::NoError,
            questions_count: 2,
            answers_count: 0,
            authority_count: 0,
            additional_count: 0,
        };

        let numericable_question = Question {
            name: DomainName::from("wpad.numericable.fr."),
            type_: Type::RRType(RRType::A),
            class: Class::IN,
        };

        let google_question = Question {
            name: DomainName::from("google.com."),
            type_: Type::RRType(RRType::MX),
            class: Class::ALL,
        };

        let expected_message = Message::new(
            expected_header,
            vec![numericable_question, google_question],
            vec![],
            vec![],
            vec![],
            None,
        );

        let message = decoder.decode(MESSAGE_WITH_TWO_QUESTIONS).unwrap();

        assert_eq!(expected_message, message);
    }

    #[test]
    fn test_decode_answer() {
        let decoder = MessageDecoder {};

        let response_buffer = &[
            226, 44, 129, 128, 0, 1, 0, 1, 0, 0, 0, 0, // header
            6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm',
            0, // question domain name
            0, 1, // question type
            0, 1, // question class
            192, 12, // alias to question named
            0, 1, // response type
            0, 1, // response class
            0, 0, 0, 244, //ttl
            0, 4, // response data length
            216, 58, 214, 174, // response data
        ];

        let expected_message = Message::new(
            Header {
                id: 57900,
                qr: MessageType::Response,
                opcode: QueryType::Standard,
                authoritative_answer: false,
                truncated: false,
                recursion_desired: true,
                recursion_available: true,
                reserved: false,
                response_code: ResponseCode::NoError,
                questions_count: 1,
                answers_count: 1,
                authority_count: 0,
                additional_count: 0,
            },
            vec![Question {
                name: DomainName::from("google.com."),
                type_: Type::RRType(RRType::A),
                class: Class::IN,
            }],
            vec![ResourceRecord::new(
                DomainName::from("google.com."),
                RRType::A,
                Class::IN,
                244,
                vec![216, 58, 214, 174],
            )],
            vec![],
            vec![],
            None,
        );

        let message = decoder.decode(response_buffer).unwrap();

        assert_eq!(expected_message, message);
    }

    #[test]
    fn test_decode_message_with_cname_answers_using_aliases() {
        let buffer = [
            80, 162, 129, 128, 0, 1, 0, 4, 0, 0, 0, 0, //header
            3, 119, 119, 119, 5, 97, 112, 112, 108, 101, 3, 99, 111, 109, 0, 0, 1, 0,
            1, // question
            192, 12, 0, 5, 0, 1, 0, 0, 0, 198, 0, 26, 13, 119, 119, 119, 45, 97, 112, 112, 108,
            101, 45, 99, 111, 109, 1, 118, 7, 97, 97, 112, 108, 105, 109, 103, 192,
            22, // answer 1
            192, 43, 0, 5, 0, 1, 0, 0, 1, 44, 0, 27, 3, 119, 119, 119, 5, 97, 112, 112, 108, 101,
            3, 99, 111, 109, 7, 101, 100, 103, 101, 107, 101, 121, 3, 110, 101, 116,
            0, // answer 2
            192, 81, 0, 5, 0, 1, 0, 0, 0, 184, 0, 25, 5, 101, 54, 56, 53, 56, 5, 100, 115, 99, 101,
            57, 10, 97, 107, 97, 109, 97, 105, 101, 100, 103, 101, 192, 103, // answer 3
            192, 120, 0, 1, 0, 1, 0, 0, 0, 20, 0, 4, 23, 40, 113, 47, // answer 4
        ];

        let expected_message = Message {
            header: Header {
                id: 20642,
                qr: MessageType::Response,
                opcode: QueryType::Standard,
                authoritative_answer: false,
                truncated: false,
                recursion_desired: true,
                recursion_available: true,
                reserved: false,
                response_code: ResponseCode::NoError,
                questions_count: 1,
                answers_count: 4,
                authority_count: 0,
                additional_count: 0,
            },
            questions: vec![Question {
                name: DomainName::from("www.apple.com."),
                type_: Type::RRType(RRType::A),
                class: Class::IN,
            }],
            answers: vec![
                ResourceRecord {
                    name: DomainName::from("www.apple.com."),
                    type_: RRType::CNAME,
                    class: Class::IN,
                    ttl: 198,
                    resource_data: vec![
                        13, 119, 119, 119, 45, 97, 112, 112, 108, 101, 45, 99, 111, 109, 1, 118, 7,
                        97, 97, 112, 108, 105, 109, 103, 192, 22,
                    ],
                },
                ResourceRecord {
                    name: DomainName::from("www-apple-com.v.aaplimg.com."),
                    type_: RRType::CNAME,
                    class: Class::IN,
                    ttl: 300,
                    resource_data: vec![
                        3, 119, 119, 119, 5, 97, 112, 112, 108, 101, 3, 99, 111, 109, 7, 101, 100,
                        103, 101, 107, 101, 121, 3, 110, 101, 116, 0,
                    ],
                },
                ResourceRecord {
                    name: DomainName::from("www.apple.com.edgekey.net."),
                    type_: RRType::CNAME,
                    class: Class::IN,
                    ttl: 184,
                    resource_data: vec![
                        5, 101, 54, 56, 53, 56, 5, 100, 115, 99, 101, 57, 10, 97, 107, 97, 109, 97,
                        105, 101, 100, 103, 101, 192, 103,
                    ],
                },
                ResourceRecord {
                    name: DomainName::from("e6858.dsce9.akamaiedge.net."),
                    type_: RRType::A,
                    class: Class::IN,
                    ttl: 20,
                    resource_data: vec![23, 40, 113, 47],
                },
            ],
            authorities: vec![],
            additionnals: vec![],
            opt_record: None,
        };

        let message = MessageDecoder {}.decode(&buffer).unwrap();

        assert_eq!(expected_message, message);
    }
}
