use header::Header;
use question::Question;
use resource_record::ResourceRecord;

pub mod domain_name;
pub mod header;
pub mod question;
pub mod resource_record;

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
    header: Header,
    questions: Vec<Question>,
    answers: Vec<ResourceRecord>,
    authorities: Vec<ResourceRecord>,
    additionnals: Vec<ResourceRecord>,
}

impl Message {
    pub fn new(
        header: Header,
        questions: Vec<Question>,
        answers: Vec<ResourceRecord>,
        authorities: Vec<ResourceRecord>,
        additionnals: Vec<ResourceRecord>,
    ) -> Self {
        Self {
            header,
            questions,
            answers,
            authorities,
            additionnals,
        }
    }
}
