// RR: resource record
// see: https://datatracker.ietf.org/doc/html/rfc1035#section-3.2

use crate::{common::domain_name::DomainName, common::question::Class as QuestionClass};

type TimeToLive = u32;

#[derive(Debug, PartialEq, Clone)]
pub struct ResourceRecord {
    pub name: DomainName,
    pub type_: Type,
    pub class: QuestionClass,
    pub ttl: TimeToLive,
    pub resource_data_length: u16,
    pub resource_data: String,
}

impl ResourceRecord {
    pub fn new(
        name: DomainName,
        type_: Type,
        class: QuestionClass,
        ttl: TimeToLive,
        resource_data_length: u16,
        resource_data: String,
    ) -> Self {
        Self {
            name,
            type_,
            class,
            ttl,
            resource_data_length,
            resource_data,
        }
    }
}

// *_OBS: obsolete
// *_EXP: experimental
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Type {
    A,        // IPv4 host address
    AAAA,     // IPv6 host address
    NS,       // authoritative name server
    MD_OBS,   // mail destination, obsolete, use MX instead
    MF_OBS,   // mail forwarder, obsolete, use MX instead
    CNAME,    // canonical name for an alias
    SOA,      // start of authority zone
    MB_EXP,   // mailbox domain name, experimental
    MG_EXP,   // mail group member, experimental
    MR_EXP,   // mail rename domain name, experimental
    NULL_EXP, // null RR, experimental
    WKS,      // well known service description
    PTR,      // domain name pointer
    HINFO,    // host information
    MINFO,    // mailbox information
    MX,       // mail exchange
    TXT,      // text strings
    SVCB,     // Service binding
    HTTPS,    // HTTPS binding
    OPT,      // pseudo-record type needed to support EDNS.
}

impl TryFrom<u16> for Type {
    type Error = String;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::A),
            2 => Ok(Self::NS),
            3 => Ok(Self::MD_OBS),
            4 => Ok(Self::MF_OBS),
            5 => Ok(Self::CNAME),
            6 => Ok(Self::SOA),
            7 => Ok(Self::MB_EXP),
            8 => Ok(Self::MG_EXP),
            9 => Ok(Self::MR_EXP),
            10 => Ok(Self::NULL_EXP),
            11 => Ok(Self::WKS),
            12 => Ok(Self::PTR),
            13 => Ok(Self::HINFO),
            14 => Ok(Self::MINFO),
            15 => Ok(Self::MX),
            16 => Ok(Self::TXT),
            28 => Ok(Self::AAAA),
            41 => Ok(Self::OPT),
            64 => Ok(Self::SVCB),
            65 => Ok(Self::HTTPS),
            _ => Err(format!("Unknown RR type: {}", value)),
        }
    }
}

impl From<Type> for u16 {
    fn from(value: Type) -> Self {
        match value {
            Type::A => 1,
            Type::NS => 2,
            Type::MD_OBS => 3,
            Type::MF_OBS => 4,
            Type::CNAME => 5,
            Type::SOA => 6,
            Type::MB_EXP => 7,
            Type::MG_EXP => 8,
            Type::MR_EXP => 9,
            Type::NULL_EXP => 10,
            Type::WKS => 11,
            Type::PTR => 12,
            Type::HINFO => 13,
            Type::MINFO => 14,
            Type::MX => 15,
            Type::TXT => 16,
            Type::AAAA => 28,
            Type::OPT => 41,
            Type::SVCB => 64,
            Type::HTTPS => 65,
        }
    }
}
