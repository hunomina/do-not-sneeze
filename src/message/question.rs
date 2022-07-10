use crate::resource_record::Type as RRType;

pub struct Question {
    name: String,
    type_: Type,
    class: Class,
}

impl Question {
    fn from_buffer<'a>(bytes: &'a [u8]) -> (Self, &'a [u8]) {
        // let name = Name::from(bytes);
        let name = String::from("");
        let type_ = Type::from(0);
        let class = Class::from(0);
        (Question { name, type_, class }, &[])
    }
}

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
