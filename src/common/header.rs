use crate::utils::{concat_two_u8s, extract_next_sixteen_bits_from_buffer};

use super::{MessageId, MessageType};

/*
    Header format:
                                    1  1  1  1  1  1
      0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                      ID                       |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |QR|   Opcode  |AA|TC|RD|RA|   Z    |   RCODE   |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                    QDCOUNT                    |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                    ANCOUNT                    |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                    NSCOUNT                    |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                    ARCOUNT                    |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+

    see: https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
    Read bits 16 by 16
    Only the second line has to be decomposed this way
*/

pub const HEADER_BIT_SIZE: usize = 96;

const IS_AUTHORITATIVE_ANSWER_BIT_MASK: u16 = 0b0000010000000000;
const IS_TRUNCATED_BIT_MASK: u16 = 0b0000001000000000;
const IS_RECUSTION_DESIRED_BIT_MASK: u16 = 0b0000000100000000;
const IS_RECURSION_AVAILABLE_BIT_MASK: u16 = 0b0000000010000000;

// bit 2, 3, 4 and 5 of second line of header contain the query type
// named OpCode
const QUERY_TYPE_BIT_MASK: u16 = 0b0111100000000000;
const QUERY_TYPE_STANDARD_BIT_VALUE: u16 = 0b0000000000000000;
const QUERY_TYPE_INVERSE_BIT_VALUE: u16 = 0b0000100000000000;
const QUERY_TYPE_SERVER_STATUS_REQUEST_BIT_VALUE: u16 = 0b0001000000000000;

const RESPONSE_CODE_BIT_MASK: u16 = 0b0000000000001111;

#[derive(Debug, PartialEq)]
pub struct Header {
    pub id: MessageId,
    pub qr: MessageType,
    pub opcode: QueryType,
    pub authoritative_answer: bool,
    pub truncated: bool,
    pub recursion_desired: bool,
    pub recursion_available: bool,
    pub reserved: bool, // should always be false
    pub response_code: ResponseCode,
    pub questions_count: u16,
    pub answers_count: u16,
    pub authority_count: u16,
    pub additional_count: u16,
}

impl From<&[u8; 12]> for Header {
    fn from(buffer: &[u8; 12]) -> Self {
        let (id, buffer) = extract_next_sixteen_bits_from_buffer(buffer); // now buffer is [u8; 10]

        let (next_sixteen_bits, buffer) = buffer.split_at(2); // now buffer is [u8; 8]
        let next_sixteen_bits = concat_two_u8s(next_sixteen_bits[0], next_sixteen_bits[1]);

        let qr = MessageType::from(next_sixteen_bits.clone());
        let opcode = QueryType::from(next_sixteen_bits.clone());
        let response_code = ResponseCode::from(next_sixteen_bits.clone());

        let authoritative_answer = next_sixteen_bits & IS_AUTHORITATIVE_ANSWER_BIT_MASK != 0;
        let truncated = next_sixteen_bits & IS_TRUNCATED_BIT_MASK != 0;
        let recursion_desired = next_sixteen_bits & IS_RECUSTION_DESIRED_BIT_MASK != 0;
        let recursion_available = next_sixteen_bits & IS_RECURSION_AVAILABLE_BIT_MASK != 0;

        let (questions_count, buffer) = extract_next_sixteen_bits_from_buffer(buffer); // now buffer is [u8; 6]
        let (answers_count, buffer) = extract_next_sixteen_bits_from_buffer(buffer); // now buffer is [u8; 4]
        let (authority_count, buffer) = extract_next_sixteen_bits_from_buffer(buffer); // now buffer is [u8; 2]
        let (additional_count, buffer) = extract_next_sixteen_bits_from_buffer(buffer); // now buffer is [u8; 0]

        // buffer should be empty since we've read 12 u8 => 96 bits == header size
        assert!(buffer.is_empty());

        Header {
            id,
            qr,
            opcode,
            authoritative_answer,
            truncated,
            recursion_desired,
            recursion_available,
            reserved: false,
            response_code,
            questions_count,
            answers_count,
            authority_count,
            additional_count,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum QueryType {
    Standard,
    Inverse,
    ServerStatusRequest,
}

impl From<u16> for QueryType {
    fn from(value: u16) -> Self {
        match value & QUERY_TYPE_BIT_MASK {
            QUERY_TYPE_STANDARD_BIT_VALUE => QueryType::Standard,
            QUERY_TYPE_INVERSE_BIT_VALUE => QueryType::Inverse,
            QUERY_TYPE_SERVER_STATUS_REQUEST_BIT_VALUE => QueryType::ServerStatusRequest,
            _ => panic!("Invalid query type: {}", value),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ResponseCode {
    NoError,
    FormatError,
    ServerFailure,
    NameError,
    NotImplemented,
    Refused,
}

impl From<u16> for ResponseCode {
    fn from(value: u16) -> Self {
        // bit 2, 3, 4 and 5 contains the query type
        match value & RESPONSE_CODE_BIT_MASK {
            0 => ResponseCode::NoError,
            1 => ResponseCode::FormatError,
            2 => ResponseCode::ServerFailure,
            3 => ResponseCode::NameError,
            4 => ResponseCode::NotImplemented,
            5 => ResponseCode::Refused,
            _ => panic!("Invalid response code: {}", value),
        }
    }
}

pub fn extract_header_bits_from_buffer(buffer: &[u8]) -> (&[u8], &[u8]) {
    // divide by 8 since buffer is composed of 8 bits unsigned integers
    buffer.split_at(HEADER_BIT_SIZE / 8)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUFFER: &[u8; 12] = &[226, 44, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0];

    #[test]
    fn test() {
        let header = Header::from(BUFFER);

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

        assert!(header == expected_header);
    }
}
