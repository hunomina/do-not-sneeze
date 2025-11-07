use header::Header;
use question::Question;
use resource_record::ResourceRecord;

use self::header::MessageType;

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

#[derive(Debug, PartialEq)]
pub struct Message {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answers: Vec<ResourceRecord>,
    pub authorities: Vec<ResourceRecord>,
    pub additionnals: Vec<ResourceRecord>,
}

impl Message {
    pub fn new(
        header: Header,
        questions: Vec<Question>,
        answers: Vec<ResourceRecord>,
        authorities: Vec<ResourceRecord>,
        additionnals: Vec<ResourceRecord>,
    ) -> Self {
        assert_eq!(header.questions_count, questions.len() as u16);
        assert_eq!(header.answers_count, answers.len() as u16);
        assert_eq!(header.authority_count, authorities.len() as u16);
        assert_eq!(header.additional_count, additionnals.len() as u16);

        Self {
            header,
            questions,
            answers,
            authorities,
            additionnals,
        }
    }

    pub fn into_response(mut self) -> Self {
        self.header.qr = MessageType::Response;

        self
    }

    pub fn set_answers(&mut self, answers: Vec<ResourceRecord>) {
        self.header.answers_count = answers.len() as u16;
        self.answers = answers;
    }
}
