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

pub const IS_AUTHORITATIVE_ANSWER_BIT_MASK: u16 = 0b0000010000000000;
pub const IS_TRUNCATED_BIT_MASK: u16 = 0b0000001000000000;
pub const IS_RECUSTION_DESIRED_BIT_MASK: u16 = 0b0000000100000000;
pub const IS_RECURSION_AVAILABLE_BIT_MASK: u16 = 0b0000000010000000;

// bit 1 of second line of header is the message type
pub const MESSAGE_TYPE_BIT_MASK: u16 = 0b1000000000000000;

// bit 2, 3, 4 and 5 of second line of header contain the query type
// named OpCode
const QUERY_TYPE_BIT_MASK: u16 = 0b0111100000000000;
pub const QUERY_TYPE_STANDARD_BIT_VALUE: u16 = 0b0000000000000000;
pub const QUERY_TYPE_INVERSE_BIT_VALUE: u16 = 0b0000100000000000;
pub const QUERY_TYPE_SERVER_STATUS_REQUEST_BIT_VALUE: u16 = 0b0001000000000000;

const RESPONSE_CODE_BIT_MASK: u16 = 0b0000000000001111;

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
            0 => Self::NoError,
            1 => Self::FormatError,
            2 => Self::ServerFailure,
            3 => Self::NameError,
            4 => Self::NotImplemented,
            5 => Self::Refused,
            _ => panic!("Invalid response code: {}", value),
        }
    }
}

impl ResponseCode {
    pub fn value(&self) -> u16 {
        match self {
            Self::NoError => 0,
            Self::FormatError => 1,
            Self::ServerFailure => 2,
            Self::NameError => 3,
            Self::NotImplemented => 4,
            Self::Refused => 5,
        }
    }
}

pub fn extract_header_bits_from_buffer(buffer: &[u8]) -> (&[u8], &[u8]) {
    // divide by 8 since buffer is composed of 8 bits unsigned integers
    buffer.split_at(HEADER_BIT_SIZE / 8)
}
