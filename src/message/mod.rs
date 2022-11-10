use crate::message::header::extract_header_bits_from_buffer;

use header::Header;
use question::Question;

pub mod header;
pub mod question;

/*
 Message format:
    +---------------------+
    |        Header       |
    +---------------------+
    |       Question      | the question for the name server
    +---------------------+
    |        Answer       | RRs answering the question
    +---------------------+
    |      Authority      | RRs pointing toward an authority
    +---------------------+
    |      Additional     | RRs holding additional information
    +---------------------+
*/

// first bit of second line of header is the message type
const MESSAGE_TYPE_BIT_MASK: u16 = 0b1000000000000000;

pub type MessageId = u16;

#[derive(Debug, PartialEq)]
pub enum MessageType {
    Query,
    Response,
}

impl From<u16> for MessageType {
    fn from(value: u16) -> Self {
        match value & MESSAGE_TYPE_BIT_MASK {
            0 => MessageType::Query,
            _ => MessageType::Response,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Message {
    source: Vec<u8>,
    header: Header,
    questions: Vec<Question>,
}

impl From<&[u8]> for Message {
    fn from(mut buffer: &[u8]) -> Self {
        let source = buffer.clone().to_owned();

        let (header_bits, rest) = extract_header_bits_from_buffer(buffer);
        buffer = rest;
        let header_bits: &[u8; header::HEADER_BIT_SIZE / 8] = header_bits.try_into().unwrap();
        let header = Header::from(header_bits);

        let questions = (0..header.questions_count)
            .map(|_| {
                let (question, rest) = Question::from_buffer(buffer);
                buffer = rest;
                question
            })
            .collect();

        Message {
            source,
            header,
            questions,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        domain_name::DomainName,
        message::header::{QueryType, ResponseCode},
        message::question::{Class, Type},
        resource_record::Type as RRType,
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
        let m = Message::from(MESSAGE_WITH_ONE_QUESTION);

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

        let expected_message = Message {
            source: MESSAGE_WITH_ONE_QUESTION.to_owned(),
            header: expected_header,
            questions: vec![numericable_question],
        };

        assert_eq!(expected_message, m);
    }

    #[test]
    fn test_message_with_two_questions() {
        let m = Message::from(MESSAGE_WITH_TWO_QUESTIONS);

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
            name: DomainName {
                labels: vec!["wpad".into(), "numericable".into(), "fr".into(), "".into()],
            },
            type_: Type::RRType(RRType::A),
            class: Class::IN,
        };

        let google_question = Question {
            name: DomainName {
                labels: vec!["google".into(), "com".into(), "".into()],
            },
            type_: Type::RRType(RRType::MX),
            class: Class::ALL,
        };

        let expected_message = Message {
            source: MESSAGE_WITH_TWO_QUESTIONS.to_owned(),
            header: expected_header,
            questions: vec![numericable_question, google_question],
        };

        assert_eq!(expected_message, m);
    }
}
