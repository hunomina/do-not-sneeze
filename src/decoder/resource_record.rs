use crate::{
    common::{
        question::Class,
        resource_record::{ResourceRecord, Type},
    },
    decoder::DecodingError,
    utils::{extract_next_sixteen_bits_from_buffer, extract_next_thirty_two_bits_from_buffer},
};
use std::{
    borrow::Cow,
    net::{Ipv4Addr, Ipv6Addr},
};

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
        Type::AAAA => decode_type_aaaa_data(buffer).to_string(),
        Type::TXT => decode_type_txt_data(buffer).to_string(),
        _ => "Unknown RR type: {:?}".into(),
    }
}

fn decode_type_a_data(buffer: &[u8]) -> Ipv4Addr {
    assert!(buffer.len() == 4);
    Ipv4Addr::new(buffer[0], buffer[1], buffer[2], buffer[3])
}

fn decode_type_aaaa_data(buffer: &[u8]) -> Ipv6Addr {
    assert_eq!(buffer.len(), 16, "AAAA record must be exactly 16 bytes");
    let octets: [u8; 16] = buffer.try_into().expect("Invalid AAAA record length");
    Ipv6Addr::from(octets)
}

fn decode_type_txt_data(buffer: &[u8]) -> Cow<'_, str> {
    let expected_length = buffer[0] as usize;
    String::from_utf8_lossy(&buffer[1..=expected_length])
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr};

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

    #[test]
    fn decode_aaaa_resource_record() {
        // IPv6 address: 2001:db8::1
        // Bytes: 20 01 0d b8 00 00 00 00 00 00 00 00 00 00 00 01
        let buffer = [
            6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm',
            0, // name: "google.com"
            0, 28, // type: AAAA (28)
            0, 1, // class: IN (1)
            0, 0, 0, 60, // ttl: 60 seconds
            0x00, 0x10, // resource data length: 16 bytes
            0x20, 0x01, 0x0d, 0xb8, 0x00, 0x00, 0x00, 0x00, // IPv6 address
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // 2001:db8::1
        ];

        let (rr, buffer) = decode(&buffer, &buffer).unwrap();

        let expected = ResourceRecord::new(
            DomainName::from("google.com"),
            Type::AAAA,
            Class::IN,
            60,
            16,
            "2001:db8::1".to_string(),
        );

        assert_eq!(expected, rr);
        assert!(buffer.is_empty(), "Buffer should be empty after decoding");
    }

    #[test]
    fn decode_aaaa_loopback() {
        // IPv6 loopback: ::1
        // Bytes: 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 01
        let buffer = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01,
        ];

        let result = decode_type_aaaa_data(&buffer);
        assert_eq!(result, Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
        assert_eq!(result.to_string(), "::1");
    }

    #[test]
    fn decode_aaaa_full_address() {
        // IPv6 address: 2607:f8b0:4004:c07::71 (Google IPv6)
        // Full form: 2607:f8b0:4004:0c07:0000:0000:0000:0071
        let buffer = [
            0x26, 0x07, 0xf8, 0xb0, 0x40, 0x04, 0x0c, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x71,
        ];

        let result = decode_type_aaaa_data(&buffer);
        assert_eq!(
            result,
            Ipv6Addr::new(0x2607, 0xf8b0, 0x4004, 0x0c07, 0, 0, 0, 0x71)
        );
        assert_eq!(result.to_string(), "2607:f8b0:4004:c07::71");
    }

    #[test]
    #[should_panic(expected = "AAAA record must be exactly 16 bytes")]
    fn decode_aaaa_invalid_length() {
        decode_type_aaaa_data(&[1, 2, 3, 4]);
    }
}
