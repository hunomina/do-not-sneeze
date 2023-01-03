// RR: resource record
// see: https://datatracker.ietf.org/doc/html/rfc1035#section-3.2

use crate::{
    common::domain_name::DomainName,
    common::question::Class as QuestionClass,
    resource_data::get_data_from_type_and_buffer,
    utils::{extract_next_sixteen_bits_from_buffer, extract_next_thirty_two_bits_from_buffer},
};

type TimeToLive = u32;

#[derive(Debug, PartialEq)]
pub struct ResourceRecord {
    name: DomainName,
    type_: Type,
    class: QuestionClass,
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

    pub fn from_buffer<'a>(buffer: &'a [u8], source: &'a [u8]) -> (Self, &'a [u8]) {
        let (name, buffer) = DomainName::from_buffer(buffer, source);

        let (type_, buffer) = extract_next_sixteen_bits_from_buffer(buffer);
        let type_ = Type::from(type_);

        let (class, buffer) = extract_next_sixteen_bits_from_buffer(buffer);
        let class = QuestionClass::from(class);

        let (ttl, buffer) = extract_next_thirty_two_bits_from_buffer(buffer);

        let (resource_data_length, buffer) = extract_next_sixteen_bits_from_buffer(buffer);

        let (data, buffer) = buffer.split_at(resource_data_length as usize);

        let resource_data = get_data_from_type_and_buffer(type_, data);

        (
            Self {
                name,
                type_,
                class,
                ttl,
                resource_data_length,
                resource_data,
            },
            buffer,
        )
    }
}

// *_OBS: obsolete
// *_EXP: experimental
#[derive(Debug, PartialEq, Clone, Copy)]
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
