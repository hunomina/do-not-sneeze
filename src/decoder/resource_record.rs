use crate::{
    common::{
        question::Class,
        resource_record::{ResourceRecord, Type},
    },
    decoder::DecodingError,
    utils::{extract_next_sixteen_bits_from_buffer, extract_next_thirty_two_bits_from_buffer},
};
use std::{borrow::Cow, net::Ipv4Addr};

use super::domain_name::decode as decode_domain_name;

pub fn decode<'a>(
    buffer: &'a [u8],
    source: &'a [u8],
) -> Result<(ResourceRecord, &'a [u8]), DecodingError> {
    let (name, buffer) = decode_domain_name(buffer, source);

    let (type_, buffer) = extract_next_sixteen_bits_from_buffer(buffer);
    let type_ = Type::try_from(type_).map_err(DecodingError::InvalidResourceRecordType)?;

    let (class, buffer) = extract_next_sixteen_bits_from_buffer(buffer);
    let class = Class::try_from(class).map_err(DecodingError::InvalidResourceRecordClass)?;

    let (ttl, buffer) = extract_next_thirty_two_bits_from_buffer(buffer);

    let (resource_data_length, buffer) = extract_next_sixteen_bits_from_buffer(buffer);

    let (data, buffer) = buffer.split_at(resource_data_length as usize);

    let resource_data = decode_data_from_type_and_buffer(type_, data);

    Ok((
        ResourceRecord::new(name, type_, class, ttl, resource_data_length, resource_data),
        buffer,
    ))
}

fn decode_data_from_type_and_buffer(type_: Type, buffer: &[u8]) -> String {
    match type_ {
        Type::A => decode_type_a_data(buffer).to_string(),
        Type::TXT => decode_type_txt_data(buffer).to_string(),
        _ => "Unknown RR type: {:?}".into(),
    }
}

fn decode_type_a_data(buffer: &[u8]) -> Ipv4Addr {
    assert!(buffer.len() == 4);
    Ipv4Addr::new(buffer[0], buffer[1], buffer[2], buffer[3])
}

fn decode_type_txt_data(buffer: &[u8]) -> Cow<'_, str> {
    let expected_length = buffer[0] as usize;
    String::from_utf8_lossy(&buffer[1..=expected_length])
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use crate::common::domain_name::DomainName;

    use super::*;

    #[test]
    fn decode_resource_record() {
        let buffer = [
            6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm',
            0, // name: "google.com"
            0, 1, // type: A (1)
            0, 1, // class: IN (1)
            0, 0, 0, 60, // ttl: 60 seconds
            0x00, 0x04, // resource data length: 4 bytes
            192, 168, 0, 1, // resource data: IP address (192.168.0.1)
        ];

        let (rr, buffer) = decode(&buffer, &buffer).unwrap();

        let expected = ResourceRecord::new(
            DomainName::from("google.com"),
            Type::A,
            Class::IN,
            60,
            4,
            Ipv4Addr::new(192, 168, 0, 1).to_string(),
        );

        assert_eq!(expected, rr);
        assert!(buffer.is_empty(), "Buffer should be empty after decoding");
    }
}
