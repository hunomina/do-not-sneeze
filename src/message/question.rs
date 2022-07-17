use crate::{
    domain_name::DomainName, resource_record::Type as RRType,
    utils::extract_next_sixteen_bits_from_buffer,
};

#[derive(Debug, PartialEq)]
pub struct Question {
    pub name: DomainName,
    pub type_: Type,
    pub class: Class,
}

impl Question {
    pub fn from_buffer<'a>(buffer: &'a [u8]) -> (Self, &'a [u8]) {
        let (name, buffer) = DomainName::from_buffer(buffer);
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
        match value {
            1..=16 => Type::RRType(RRType::from(value)),
            252 => Self::AXFR,
            253 => Self::MAILB,
            254 => Self::MAILA,
            255 => Self::ALL,
            _ => panic!("Unknown QType: {}", value),
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
