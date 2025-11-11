use crate::common::Message;

use self::{
    header::encode as encode_header, question::encode as encode_question,
    resource_record::encode as encode_resource_record,
};

mod domain_name;
mod header;
mod opt_record;
mod question;
mod resource_record;

pub trait Encoder {
    fn encode(&self, message: Message) -> Vec<u8>;
}

pub struct MessageEncoder {}

impl Encoder for MessageEncoder {
    fn encode(&self, message: Message) -> Vec<u8> {
        let mut r = vec![];

        r.extend(encode_header(message.header));

        message
            .questions
            .into_iter()
            .for_each(|q| r.extend(encode_question(q)));

        message
            .answers
            .into_iter()
            .for_each(|rr| r.extend(encode_resource_record(rr)));

        message
            .authorities
            .into_iter()
            .for_each(|rr| r.extend(encode_resource_record(rr)));

        message
            .additionnals
            .into_iter()
            .for_each(|rr| r.extend(encode_resource_record(rr)));

        if let Some(opt) = &message.opt_record {
            r.extend(opt_record::encode(opt));
        }

        r
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

    #[test]
    fn test_decode_answer() {
        let encoder = MessageEncoder {};

        let message = Message::new(
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

        let encoded_message = encoder.encode(message);

        let expected_encoded_message = &[
            226, 44, 129, 128, 0, 1, 0, 1, 0, 0, 0, 0, // header
            6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm',
            0, // question domain name
            0, 1, // question type
            0, 1, // question class
            6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm',
            0, // response domain name
            0, 1, // response type
            0, 1, // response class
            0, 0, 0, 244, //ttl
            0, 4, // response data length
            216, 58, 214, 174, // response data
        ];

        assert_eq!(expected_encoded_message.to_vec(), encoded_message);
    }
}
