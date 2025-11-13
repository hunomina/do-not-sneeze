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
        Type::MX => decode_type_mx_data(buffer),
        Type::CNAME | Type::NS => decode_record_type_as_domain_name(buffer),
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

fn decode_type_mx_data(buffer: &[u8]) -> Vec<u8> {
    // MX record format: preference (2 bytes) + domain name
    assert!(
        buffer.len() >= 2,
        "MX record must have at least 2 bytes for preference"
    );

    let (_, domain_buffer) = extract_next_sixteen_bits_from_buffer(buffer);

    // Verify domain name is present and properly formatted
    assert!(
        !domain_buffer.is_empty(),
        "MX record must contain a domain name after preference"
    );

    // Decode the domain name to verify it's valid and buffer is fully consumed
    let (_, remaining) = decode_domain_name(domain_buffer, domain_buffer);

    // Assert that the entire buffer has been consumed
    assert!(
        remaining.is_empty(),
        "Buffer must be empty after decoding MX record"
    );

    // Return the original buffer as-is
    buffer.to_vec()
}

fn decode_record_type_as_domain_name(buffer: &[u8]) -> Vec<u8> {
    let (_, remaining) = decode_domain_name(buffer, buffer);

    // Assert that the entire buffer has been consumed
    assert!(
        remaining.is_empty(),
        "Buffer must be empty after decoding MX record"
    );

    // Domain names are stored as-is in the resource data
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

    #[test]
    fn decode_ns_resource_record() {
        // NS record for example.com pointing to ns1.example.com
        let buffer = [
            7, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 3, b'c', b'o', b'm',
            0, // name: "example.com"
            0, 2, // type: NS (2)
            0, 1, // class: IN (1)
            0, 1, 81, 128, // ttl: 86400 seconds (1 day)
            0, 17, // resource data length: 17 bytes
            3, b'n', b's', b'1', 7, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 3, b'c', b'o', b'm',
            0, // ns target: "ns1.example.com"
        ];

        let (rr, buffer) = decode(&buffer, &buffer).unwrap();

        let expected = ResourceRecord::new(
            DomainName::from("example.com"),
            Type::NS,
            Class::IN,
            86400,
            vec![
                3, b'n', b's', b'1', 7, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 3, b'c', b'o',
                b'm', 0,
            ],
        );

        assert_eq!(expected, rr);
        assert!(buffer.is_empty(), "Buffer should be empty after decoding");
    }

    #[test]
    fn decode_mx_resource_record() {
        // MX record for example.com pointing to mail.example.com with preference 10
        let buffer = [
            7, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 3, b'c', b'o', b'm',
            0, // name: "example.com"
            0, 15, // type: MX (15)
            0, 1, // class: IN (1)
            0, 0, 14, 16, // ttl: 3600 seconds
            0, 20, // resource data length: 19 bytes (2 for preference + 17 for domain)
            0, 10, // preference: 10
            4, b'm', b'a', b'i', b'l', 7, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 3, b'c', b'o',
            b'm', 0, // exchange: "mail.example.com"
        ];

        let (rr, buffer) = decode(&buffer, &buffer).unwrap();

        let expected = ResourceRecord::new(
            DomainName::from("example.com"),
            Type::MX,
            Class::IN,
            3600,
            vec![
                0, 10, // preference: 10
                4, b'm', b'a', b'i', b'l', 7, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 3, b'c',
                b'o', b'm', 0, // exchange: "mail.example.com"
            ],
        );

        assert_eq!(expected, rr);
        assert!(buffer.is_empty(), "Buffer should be empty after decoding");
    }

    #[test]
    fn decode_type_mx_data_with_preference_and_domain() {
        // MX record: preference 10, mail.example.com
        let buffer = [
            0, 10, // preference: 10
            4, b'm', b'a', b'i', b'l', 7, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 3, b'c', b'o',
            b'm', 0, // exchange: "mail.example.com"
        ];

        let result = decode_type_mx_data(&buffer);

        // Should return the entire buffer
        assert_eq!(result, buffer.to_vec());
        assert_eq!(result.len(), 20, "MX data should be 20 bytes");
    }

    #[test]
    #[should_panic(expected = "MX record must have at least 2 bytes for preference")]
    fn decode_type_mx_data_empty_buffer() {
        decode_type_mx_data(&[]);
    }

    #[test]
    #[should_panic(expected = "MX record must contain a domain name after preference")]
    fn decode_type_mx_data_missing_domain() {
        // Only preference, no domain name
        decode_type_mx_data(&[0, 10]);
    }

    #[test]
    #[should_panic(expected = "Buffer must be empty after decoding MX record")]
    fn decode_type_mx_data_extra_bytes_after_domain() {
        // Valid MX record but with extra bytes at the end
        let buffer = [
            0, 10, // preference: 10
            4, b'm', b'a', b'i', b'l', 3, b'c', b'o', b'm', 0, // exchange: "mail.com"
            1, 2, 3, // extra bytes that shouldn't be here
        ];

        decode_type_mx_data(&buffer);
    }
}
