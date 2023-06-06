// RR: resource record
// see: https://datatracker.ietf.org/doc/html/rfc1035#section-3.2

use crate::{common::domain_name::DomainName, common::question::Class as QuestionClass};

type TimeToLive = u32;

#[derive(Debug, PartialEq, Clone)]
pub struct ResourceRecord {
    pub name: DomainName,
    pub type_: Type,
    pub class: QuestionClass,
    ttl: TimeToLive,
    resource_data_length: u16,
    resource_data: String,
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

impl From<u16> for Type {
    fn from(value: u16) -> Self {
        match value {
            1 => Self::A,
            2 => Self::NS,
            3 => Self::MD_OBS,
            4 => Self::MF_OBS,
            5 => Self::CNAME,
            6 => Self::SOA,
            7 => Self::MB_EXP,
            8 => Self::MG_EXP,
            9 => Self::MR_EXP,
            10 => Self::NULL_EXP,
            11 => Self::WKS,
            12 => Self::PTR,
            13 => Self::HINFO,
            14 => Self::MINFO,
            15 => Self::MX,
            16 => Self::TXT,
            28 => Self::AAAA,
            41 => Self::OPT,
            64 => Self::SVCB,
            65 => Self::HTTPS,
            _ => panic!("Unknown RR type: {}", value),
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
