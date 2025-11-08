use header::Header;
use opt_record::OptRecord;
use question::Question;
use resource_record::ResourceRecord;

use crate::transport::UDP_MAX_MESSAGE_SIZE;

use self::header::MessageType;

pub mod domain_name;
pub mod header;
pub mod opt_record;
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

#[derive(Debug, PartialEq, Clone)]
pub struct Message {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answers: Vec<ResourceRecord>,
    pub authorities: Vec<ResourceRecord>,
    pub additionnals: Vec<ResourceRecord>,
    pub opt_record: Option<OptRecord>,
}

impl Message {
    pub fn new(
        header: Header,
        questions: Vec<Question>,
        answers: Vec<ResourceRecord>,
        authorities: Vec<ResourceRecord>,
        additionnals: Vec<ResourceRecord>,
        opt_record: Option<OptRecord>,
    ) -> Self {
        // If OPT record is present, it counts as one additional record
        let expected_additional = additionnals.len() + if opt_record.is_some() { 1 } else { 0 };

        assert_eq!(header.questions_count, questions.len() as u16);
        assert_eq!(header.answers_count, answers.len() as u16);
        assert_eq!(header.authority_count, authorities.len() as u16);
        assert_eq!(header.additional_count, expected_additional as u16);

        Self {
            header,
            questions,
            answers,
            authorities,
            additionnals,
            opt_record,
        }
    }

    pub fn into_response(mut self) -> Self {
        self.header.qr = MessageType::Response;

        self
    }

    pub fn truncate(mut self) -> Self {
        self.header.truncated = true;
        self.set_answers(vec![]);
        self.set_authorities(vec![]);
        self.set_additionnals(vec![]);

        self.header.response_code = header::ResponseCode::NoError;

        self
    }

    pub fn set_answers(&mut self, answers: Vec<ResourceRecord>) {
        self.header.answers_count = answers.len() as u16;
        self.answers = answers;
    }

    pub fn set_authorities(&mut self, authorities: Vec<ResourceRecord>) {
        self.header.authority_count = authorities.len() as u16;
        self.authorities = authorities;
    }

    pub fn set_additionnals(&mut self, additionnals: Vec<ResourceRecord>) {
        self.header.additional_count = additionnals.len() as u16;
        self.additionnals = additionnals;
    }

    pub fn max_message_size(&self) -> usize {
        if let Some(opt_record) = &self.opt_record {
            opt_record.udp_payload_size as usize
        } else {
            UDP_MAX_MESSAGE_SIZE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use header::{MessageType, QueryType, ResponseCode};

    #[test]
    fn truncate() {
        let questions = vec![Question {
            name: domain_name::DomainName::from("example.com."),
            type_: question::Type::RRType(resource_record::Type::A),
            class: question::Class::IN,
        }];

        let message = Message::new(
            Header {
                id: 1234,
                qr: MessageType::Query,
                opcode: QueryType::Standard,
                authoritative_answer: false,
                truncated: false,
                recursion_desired: true,
                recursion_available: false,
                reserved: false,
                response_code: ResponseCode::NoError,
                questions_count: 1,
                answers_count: 2,
                authority_count: 1,
                additional_count: 1,
            },
            questions.clone(),
            vec![
                ResourceRecord::new(
                    domain_name::DomainName::from("example.com."),
                    resource_record::Type::A,
                    question::Class::IN,
                    300,
                    4,
                    "192.0.2.1".to_string(),
                ),
                ResourceRecord::new(
                    domain_name::DomainName::from("example.com."),
                    resource_record::Type::A,
                    question::Class::IN,
                    300,
                    4,
                    "192.0.2.2".to_string(),
                ),
            ],
            vec![ResourceRecord::new(
                domain_name::DomainName::from("example.com."),
                resource_record::Type::NS,
                question::Class::IN,
                3600,
                17,
                "ns1.example.com.".to_string(),
            )],
            vec![ResourceRecord::new(
                domain_name::DomainName::from("ns1.example.com."),
                resource_record::Type::A,
                question::Class::IN,
                3600,
                4,
                "192.0.2.3".to_string(),
            )],
            None,
        );

        // Truncate the message
        let truncated_message = message.truncate();

        let expected_truncated_message = Message::new(
            Header {
                id: 1234,
                qr: MessageType::Query,
                opcode: QueryType::Standard,
                authoritative_answer: false,
                truncated: true,
                recursion_desired: true,
                recursion_available: false,
                reserved: false,
                response_code: ResponseCode::NoError,
                questions_count: 1,
                answers_count: 0,
                authority_count: 0,
                additional_count: 0,
            },
            questions.clone(),
            vec![],
            vec![],
            vec![],
            None,
        );

        assert_eq!(truncated_message, expected_truncated_message);
    }
}
