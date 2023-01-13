use crate::{
    common::{domain_name::DomainName, resource_record::Type as RRType},
    utils::extract_next_sixteen_bits_from_buffer,
};

#[derive(Debug, PartialEq)]
pub struct Question {
    pub name: DomainName,
    pub type_: Type,
    pub class: Class,
}

impl Question {
    pub fn from_buffer(buffer: &[u8]) -> (Self, &[u8]) {
        let (name, buffer) = DomainName::from_buffer(buffer, buffer);
        let (type_bytes, buffer) = extract_next_sixteen_bits_from_buffer(buffer);
        let (class_bytes, buffer) = extract_next_sixteen_bits_from_buffer(buffer);
        (
            Question {
                name,
                type_: Type::from(type_bytes),
                class: Class::from(class_bytes),
            },
            buffer,
        )
    }
}

#[derive(Debug, PartialEq)]
pub enum Type {
    RRType(RRType),
    AXFR,  // request for transfer of entire zone
    MAILB, // request for mailbox-related records (MB, MG or MR)
    MAILA, // request for mail agent RRs (Obsolete - see MX)
    ALL,   // request for all records
}

impl From<u16> for Type {
    fn from(value: u16) -> Self {
        match RRType::try_from(value) {
            Ok(rr_type) => Type::RRType(rr_type),
            Err(_) => match value {
                252 => Self::AXFR,
                253 => Self::MAILB,
                254 => Self::MAILA,
                255 => Self::ALL,
                _ => panic!("Unknown QType: {}", value),
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Class {
    IN, // the Internet
    CS, // the CSNET class (Obsolete - use IN)
    CH, // the CHAOS class
    HS, // Hesiod [Dyer 87]
    ALL,
}

impl From<u16> for Class {
    fn from(value: u16) -> Self {
        match value {
            1 => Self::IN,
            2 => Self::CS,
            3 => Self::CH,
            4 => Self::HS,
            255 => Self::ALL,
            _ => panic!("Unknown QClass: {}", value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_from_buffer() {
        let buffer = &[
            4, 119, 112, 97, 100, 11, 110, 117, 109, 101, 114, 105, 99, 97, 98, 108, 101, 2, 102,
            114, 0, // domain name
            0, 1, // question type
            0, 1, // question class
        ];
        let (question, rest) = Question::from_buffer(buffer);
        let expected_question = Question {
            name: DomainName {
                labels: vec!["wpad".into(), "numericable".into(), "fr".into(), "".into()],
            },
            type_: Type::RRType(RRType::A),
            class: Class::IN,
        };

        assert_eq!(expected_question, question);
        assert_eq!(rest, &[]);
    }
}
