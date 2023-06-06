mod domain_name;
mod header;
mod question;
mod resource_record;

use crate::common::{
    header::{extract_header_bits_from_buffer, HEADER_BIT_SIZE},
    Message,
};

pub trait Decoder {
    fn decode(&self, buffer: &[u8]) -> Result<Message, DecodingError>;
}

#[derive(Debug)]
pub enum DecodingError {
    BOOM,
}

#[derive(Clone, Copy)]
pub struct MessageDecoder {}

impl Decoder for MessageDecoder {
    fn decode(&self, buffer: &[u8]) -> Result<Message, DecodingError> {
        let source = buffer;

        let (header_bits, mut buffer) = extract_header_bits_from_buffer(buffer);
        let header_bits: &[u8; HEADER_BIT_SIZE / 8] = header_bits.try_into().unwrap();
        let header = header::decode(header_bits);

        let questions = (0..header.questions_count)
            .map(|_| {
                let (question, rest) = question::decode(buffer);
                buffer = rest;
                question
            })
            .collect();

        let answers = (0..header.answers_count)
            .map(|_| {
                let (rr, rest) = resource_record::decode(buffer, source);
                buffer = rest;
                rr
            })
            .collect();

        let authorities = (0..header.authority_count)
            .map(|_| {
                let (rr, rest) = resource_record::decode(buffer, source);
                buffer = rest;
                rr
            })
            .collect();

        let additionnals = (0..header.additional_count)
            .map(|_| {
                let (rr, rest) = resource_record::decode(buffer, source);
                buffer = rest;
                rr
            })
            .collect();

        Ok(Message::new(
            header,
            questions,
            answers,
            authorities,
            additionnals,
        ))
    }
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
                4,
                String::from("216.58.214.174"),
            )],
            vec![],
            vec![],
        );

        let message = decoder.decode(response_buffer).unwrap();

        assert_eq!(expected_message, message);
    }
}
