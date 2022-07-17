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
            .into_iter()
            .collect();

        // println!("after buffer: {:?}", buffer);

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

    const BUFFER: &[u8] = &[
        226, 44, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, //header
        4, 119, 112, 97, 100, 11, 110, 117, 109, 101, 114, 105, 99, 97, 98, 108, 101, 2, 102, 114,
        0, // questions
        0, 1, // question type
        0, 1, // question class
    ];

    #[test]
    fn test() {
        let m = Message::from(BUFFER);

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

        let expected_questions = vec![Question {
            name: DomainName {
                labels: vec!["wpad".into(), "numericable".into(), "fr".into(), "".into()],
            },
            type_: Type::RRType(RRType::A),
            class: Class::IN,
        }];

        let expected_message = Message {
            source: BUFFER.to_owned(),
            header: expected_header,
            questions: expected_questions,
        };

        assert_eq!(expected_message, m);

        println!("{:?}", m);
    }
}
