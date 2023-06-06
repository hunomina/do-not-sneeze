use crate::common::{domain_name::DomainName, resource_record::Type as RRType};

#[derive(Debug, PartialEq, Clone)]
pub struct Question {
    pub name: DomainName,
    pub type_: Type,
    pub class: Class,
}

#[derive(Debug, PartialEq, Clone)]
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

impl From<Type> for u16 {
    fn from(value: Type) -> Self {
        match value {
            Type::RRType(r) => r.into(),
            Type::AXFR => 252,
            Type::MAILB => 253,
            Type::MAILA => 254,
            Type::ALL => 255,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
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

impl From<Class> for u16 {
    fn from(value: Class) -> Self {
        match value {
            Class::IN => 1,
            Class::CS => 2,
            Class::CH => 3,
            Class::HS => 4,
            Class::ALL => 255,
        }
    }
}
