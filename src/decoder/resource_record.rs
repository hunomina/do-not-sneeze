use crate::{
    common::{
        question::Class,
        resource_record::{ResourceRecord, Type},
    },
    decoder::DecodingError,
    utils::{extract_next_sixteen_bits_from_buffer, extract_next_thirty_two_bits_from_buffer},
};
use std::net::{Ipv4Addr, Ipv6Addr};

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

    assert!(
        resource_data.len() == resource_data_length as usize,
        "Decoded resource data length does not match the expected length"
    );

    Ok((
        ResourceRecord::new(name, type_, class, ttl, resource_data),
        buffer,
    ))
}

fn decode_data_from_type_and_buffer(type_: Type, buffer: &[u8]) -> Vec<u8> {
    match type_ {
        Type::A => decode_type_a_data(buffer),
        Type::AAAA => decode_type_aaaa_data(buffer),
        Type::TXT => decode_type_txt_data(buffer),
        Type::CNAME => decode_type_cname_data(buffer),
        _ => "Unknown RR type: {:?}".into(),
    }
}

fn decode_type_a_data(buffer: &[u8]) -> Vec<u8> {
    assert!(buffer.len() == 4);
    Ipv4Addr::new(buffer[0], buffer[1], buffer[2], buffer[3])
        .octets()
        .to_vec()
}

fn decode_type_aaaa_data(buffer: &[u8]) -> Vec<u8> {
    assert_eq!(buffer.len(), 16, "AAAA record must be exactly 16 bytes");
    let octets: [u8; 16] = buffer.try_into().expect("Invalid AAAA record length");
    Ipv6Addr::from(octets).octets().to_vec()
}

fn decode_type_txt_data(buffer: &[u8]) -> Vec<u8> {
    let expected_length = buffer[0] as usize;
    String::from_utf8_lossy(&buffer[1..=expected_length])
        .as_bytes()
        .to_vec()
}

fn decode_type_cname_data(buffer: &[u8]) -> Vec<u8> {
    // CNAME RDATA is a domain name in DNS wire format
    // Return the buffer as-is (it's already encoded domain name bytes)
    buffer.to_vec()
}

#[cfg(test)]
mod tests {
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
            vec![192, 168, 0, 1],
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
            vec![
                0x20, 0x01, 0x0d, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x01,
            ],
        );

        assert_eq!(expected, rr);
        assert!(buffer.is_empty(), "Buffer should be empty after decoding");
    }

    #[test]
    fn decode_aaaa_loopback() {
        let buffer = "::1".parse::<std::net::Ipv6Addr>().unwrap().octets();

        let result = decode_type_aaaa_data(&buffer);
        assert_eq!(
            result,
            vec![
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x01,
            ]
        );
    }

    #[test]
    fn decode_aaaa_full_address() {
        let buffer = "2607:f8b0:4004:c07::71"
            .parse::<std::net::Ipv6Addr>()
            .unwrap()
            .octets();

        let result = decode_type_aaaa_data(&buffer);
        assert_eq!(
            result,
            vec![
                0x26, 0x07, 0xf8, 0xb0, 0x40, 0x04, 0x0c, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x71,
            ]
        );
    }

    #[test]
    #[should_panic(expected = "AAAA record must be exactly 16 bytes")]
    fn decode_aaaa_invalid_length() {
        decode_type_aaaa_data(&[1, 2, 3, 4]);
    }

    #[test]
    fn decode_cname_resource_record() {
        // CNAME for www.example.com pointing to example.com
        let buffer = [
            3, b'w', b'w', b'w', 7, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 3, b'c', b'o', b'm',
            0, // name: "www.example.com"
            0, 5, // type: CNAME (5)
            0, 1, // class: IN (1)
            0, 0, 14, 16, // ttl: 3600 seconds
            0, 13, // resource data length: 13 bytes (including trailing 0)
            7, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 3, b'c', b'o', b'm',
            0, // cname target: "example.com"
        ];

        let (rr, buffer) = decode(&buffer, &buffer).unwrap();

        let expected = ResourceRecord::new(
            DomainName::from("www.example.com"),
            Type::CNAME,
            Class::IN,
            3600,
            vec![
                7, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 3, b'c', b'o', b'm', 0,
            ],
        );

        assert_eq!(expected, rr);
        assert!(buffer.is_empty(), "Buffer should be empty after decoding");
    }
}
