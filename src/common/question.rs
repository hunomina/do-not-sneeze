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

impl TryFrom<u16> for Type {
    type Error = String;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match RRType::try_from(value) {
            Ok(rr_type) => Ok(Type::RRType(rr_type)),
            Err(_) => match value {
                252 => Ok(Self::AXFR),
                253 => Ok(Self::MAILB),
                254 => Ok(Self::MAILA),
                255 => Ok(Self::ALL),
                _ => Err(format!("Unknown QType: {}", value)),
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

impl TryFrom<u16> for Class {
    type Error = String;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::IN),
            2 => Ok(Self::CS),
            3 => Ok(Self::CH),
            4 => Ok(Self::HS),
            255 => Ok(Self::ALL),
            _ => Err(format!("Unknown QClass: {}", value)),
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
