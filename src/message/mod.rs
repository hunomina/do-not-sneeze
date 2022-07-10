use crate::message::header::extract_header_bits_from_buffer;

use header::Header;

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

#[derive(Debug)]
pub struct Message {
    source: Vec<u8>,
    // might be useful to store the original buffer
    header: Header,
    // question: question::Question,
}

impl From<&[u8]> for Message {
    fn from(buffer: &[u8]) -> Self {
        let source = buffer.clone().to_owned();
        let (header_bits, buffer) = extract_header_bits_from_buffer(buffer);
        let header_bits: &[u8; header::HEADER_BIT_SIZE / 8] = header_bits.try_into().unwrap();
        Message {
            source,
            header: Header::from(header_bits),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::message::header::{QueryType, ResponseCode};

    use super::*;

    const BUFFER: &[u8] = &[
        226, 44, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 4, 119, 112, 97, 100, 11, 110, 117, 109, 101, 114,
        105, 99, 97, 98, 108, 101, 2, 102, 114, 0, 0, 1, 0, 1,
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

        assert!(m.header == expected_header);
    }
}
