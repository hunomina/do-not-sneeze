use crate::{common::Message, utils::split_two_bytes};

use self::{header::encode as encode_header, question::encode as encode_question};

mod domain_name;
mod header;
mod question;

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

        // todo: answers
        // todo: authorities
        // todo: additionals

        r
    }
}

fn push_u16_to_u8_vec(vec: &mut Vec<u8>, value: u16) {
    let splitted_value = split_two_bytes(value);
    vec.push(splitted_value.0);
    vec.push(splitted_value.1);
}
