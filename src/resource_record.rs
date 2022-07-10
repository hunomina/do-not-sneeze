// RR: resource record
// see: https://datatracker.ietf.org/doc/html/rfc1035#section-3.2

use crate::message::question::Class as QuestionClass;

pub struct ResourceRecord {
    name: String,
    type_: Type,
    class: QuestionClass,
    ttl: TimeToLive,
    resource_data_length: u16,
    resource_data: String,
}

type TimeToLive = u32;

// *_OBS: obsolete
// *_EXP: experimental
pub enum Type {
    A,        // host address
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
            _ => panic!("Unknown RR type: {}", value),
        }
    }
}
