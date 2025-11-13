use std::net::{Ipv4Addr, Ipv6Addr};

use crate::{
    common::resource_record::{ResourceRecord, Type},
    encoder::domain_name::encode as encode_domain_name,
    utils::{push_u16_to_u8_vec, push_u32_to_u8_vec},
};

// todo: better error handling returning Result<Vec<u8>, ..> instead
pub fn encode(resource_record: ResourceRecord) -> Vec<u8> {
    let mut r = vec![];

    r.extend(encode_domain_name(resource_record.name));

    push_u16_to_u8_vec(&mut r, resource_record.type_.into());
    push_u16_to_u8_vec(&mut r, resource_record.class.into());

    push_u32_to_u8_vec(&mut r, resource_record.ttl);

    // Encode the resource data first to get its actual length
    let encoded_data = encode_resource_data_from_type_and_string(
        resource_record.type_,
        resource_record.resource_data,
    );

    // Write the actual length of the encoded data
    push_u16_to_u8_vec(&mut r, encoded_data.len() as u16);
    // Write the encoded data
    r.extend(encoded_data);

    r
}

fn encode_resource_data_from_type_and_string(type_: Type, value: Vec<u8>) -> Vec<u8> {
    match type_ {
        Type::A => encode_type_a_string(value).to_vec(),
        Type::AAAA => encode_type_aaaa_string(value).to_vec(),
        Type::TXT => encode_type_txt_string(value),
        Type::CNAME | Type::NS => value,
        t => unimplemented!("Unimplemented resource record data type encoding {:?}", t),
    }
}

fn encode_type_txt_string(value: Vec<u8>) -> Vec<u8> {
    let s = String::from_utf8_lossy(&value);
    let mut result = Vec::with_capacity(value.len() + 1);
    // TXT records must be prefixed with a length byte
    result.push(s.len() as u8);
    result.extend_from_slice(value.as_slice());
    result
}

fn encode_type_a_string(bytes: Vec<u8>) -> [u8; 4] {
    assert!(bytes.len() == 4, "A record must be exactly 4 bytes");

    Ipv4Addr::from([bytes[0], bytes[1], bytes[2], bytes[3]]).octets()
}

fn encode_type_aaaa_string(bytes: Vec<u8>) -> [u8; 16] {
    assert!(bytes.len() == 16, "AAAA record must be exactly 16 bytes");

    Ipv6Addr::from([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8],
        bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    ])
    .octets()
}

#[cfg(test)]
mod tests {
    use crate::common::{
        domain_name::DomainName,
        question::Class,
        resource_record::{ResourceRecord, Type},
    };

    use super::*;

    #[test]
    fn encode_resource_record() {
        let rr = ResourceRecord::new(
            DomainName::from("google.com"),
            Type::A,
            Class::IN,
            60,
            vec![192, 168, 0, 1],
        );

        let encoded_rr = encode(rr);

        let expected = [
            6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm',
            0, // name: "google.com"
            0, 1, // type: A (1)
            0, 1, // class: IN (1)
            0, 0, 0, 60, // ttl: 60 seconds
            0, 4, // resource data length: 4 bytes
            192, 168, 0, 1, // resource data: IP address (192.168.0.1)
        ];

        assert_eq!(expected, encoded_rr.as_slice());
    }

    #[test]
    fn encode_type_a() {
        let ipv4 = "192.168.0.1"
            .parse::<std::net::Ipv4Addr>()
            .unwrap()
            .octets()
            .to_vec();
        let encoded = encode_type_a_string(ipv4);

        // Expected: length byte (11) followed by the text
        let expected = vec![192, 168, 0, 1];

        assert_eq!(expected, encoded);
    }

    #[test]
    fn encode_type_txt_string_with_simple_text() {
        let text = "hello world".as_bytes().to_vec();
        let encoded = encode_type_txt_string(text);

        // Expected: length byte (11) followed by the text
        let expected = vec![
            11, b'h', b'e', b'l', b'l', b'o', b' ', b'w', b'o', b'r', b'l', b'd',
        ];

        assert_eq!(expected, encoded);
    }

    #[test]
    fn encode_type_txt_string_empty() {
        let text = String::new().as_bytes().to_vec();
        let encoded = encode_type_txt_string(text);

        // Expected: just a length byte of 0
        let expected = vec![0];

        assert_eq!(expected, encoded);
    }

    #[test]
    fn encode_type_txt_string_max_length() {
        // TXT records can have up to 255 characters per string
        let text = "a".repeat(255).as_bytes().to_vec();
        let encoded = encode_type_txt_string(text);

        // Expected: length byte (255) followed by 255 'a' characters
        assert_eq!(256, encoded.len());
        assert_eq!(255, encoded[0]);
        assert!(encoded[1..].iter().all(|&b| b == b'a'));
    }

    #[test]
    fn encode_txt_resource_record() {
        let text_content = "some content for google.com".as_bytes().to_vec();
        let rr = ResourceRecord::new(
            DomainName::from("google.com"),
            Type::TXT,
            Class::IN,
            3600,
            text_content,
        );

        let encoded_rr = encode(rr);

        let expected = [
            6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm',
            0, // name: "google.com"
            0, 16, // type: TXT (16)
            0, 1, // class: IN (1)
            0, 0, 14, 16, // ttl: 3600 seconds
            0, 28, // resource data length: 28 bytes (1 length byte + 27 text bytes)
            27, // length byte
            b's', b'o', b'm', b'e', b' ', b'c', b'o', b'n', b't', b'e', b'n', b't', b' ', b'f',
            b'o', b'r', b' ', b'g', b'o', b'o', b'g', b'l', b'e', b'.', b'c', b'o', b'm',
        ];

        assert_eq!(expected, encoded_rr.as_slice());
    }

    #[test]
    fn encode_regular_type_aaaa_string() {
        let ipv6 = "2607:f8b0:4004:0c07::71"
            .parse::<std::net::Ipv6Addr>()
            .unwrap()
            .octets()
            .to_vec();
        let encoded = encode_type_aaaa_string(ipv6);

        let expected = [
            0x26, 0x07, 0xf8, 0xb0, 0x40, 0x04, 0x0c, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x71,
        ];

        assert_eq!(expected, encoded);
    }

    #[test]
    fn encode_type_aaaa_string_loopback() {
        let ipv6 = "::1"
            .parse::<std::net::Ipv6Addr>()
            .unwrap()
            .octets()
            .to_vec();
        let encoded = encode_type_aaaa_string(ipv6);

        let expected = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01,
        ];

        assert_eq!(expected, encoded);
    }

    #[test]
    fn encode_aaaa_resource_record() {
        let rr = ResourceRecord::new(
            DomainName::from("google.com"),
            Type::AAAA,
            Class::IN,
            3600,
            "2001:db8::1"
                .parse::<std::net::Ipv6Addr>()
                .unwrap()
                .octets()
                .to_vec(),
        );

        let encoded_rr = encode(rr);

        let expected = [
            6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm',
            0, // name: "google.com"
            0, 28, // type: AAAA (28)
            0, 1, // class: IN (1)
            0, 0, 14, 16, // ttl: 3600 seconds
            0, 16, // resource data length: 16 bytes
            0x20, 0x01, 0x0d, 0xb8, 0x00, 0x00, 0x00, 0x00, // IPv6 address
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // 2001:db8::1
        ];

        assert_eq!(expected, encoded_rr.as_slice());
    }

    #[test]
    fn encode_cname_resource_record() {
        // CNAME for www.example.com pointing to example.com
        let cname_target = DomainName::from("example.com");
        let cname_data = encode_domain_name(cname_target);

        let rr = ResourceRecord::new(
            DomainName::from("www.example.com"),
            Type::CNAME,
            Class::IN,
            3600,
            cname_data,
        );

        let encoded_rr = encode(rr);

        let expected = [
            3, b'w', b'w', b'w', 7, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 3, b'c', b'o', b'm',
            0, // name: "www.example.com"
            0, 5, // type: CNAME (5)
            0, 1, // class: IN (1)
            0, 0, 14, 16, // ttl: 3600 seconds
            0, 13, // resource data length: 13 bytes (including trailing 0)
            7, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 3, b'c', b'o', b'm',
            0, // cname target: "example.com"
        ];

        assert_eq!(expected, encoded_rr.as_slice());
    }

    #[test]
    fn encode_ns_resource_record() {
        // NS record for example.com pointing to ns1.example.com
        let ns_target = DomainName::from("ns1.example.com");
        let ns_data = encode_domain_name(ns_target);

        let rr = ResourceRecord::new(
            DomainName::from("example.com"),
            Type::NS,
            Class::IN,
            86400,
            ns_data,
        );

        let encoded_rr = encode(rr);

        let expected = [
            7, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 3, b'c', b'o', b'm',
            0, // name: "example.com"
            0, 2, // type: NS (2)
            0, 1, // class: IN (1)
            0, 1, 81, 128, // ttl: 86400 seconds (1 day)
            0, 17, // resource data length: 17 bytes
            3, b'n', b's', b'1', 7, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 3, b'c', b'o', b'm',
            0, // ns target: "ns1.example.com"
        ];

        assert_eq!(expected, encoded_rr.as_slice());
    }
}
