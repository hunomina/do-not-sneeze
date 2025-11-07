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

    // Write the actual length of the encoded data
    push_u16_to_u8_vec(&mut r, resource_record.resource_data_length);

    // Encode the resource data first to get its actual length
    let encoded_data = encode_resource_data_from_type_and_string(
        resource_record.type_,
        resource_record.resource_data,
    );

    assert!(resource_record.resource_data_length as usize == encoded_data.len());

    // Write the encoded data
    r.extend(encoded_data);

    r
}

fn encode_resource_data_from_type_and_string(type_: Type, value: String) -> Vec<u8> {
    match type_ {
        Type::A => encode_type_a_string(value).to_vec(),
        Type::AAAA => encode_type_aaaa_string(value).to_vec(),
        Type::TXT => encode_type_txt_string(value),
        t => unimplemented!("Unimplemented resource record data type encoding {:?}", t),
    }
}

fn encode_type_txt_string(value: String) -> Vec<u8> {
    let bytes = value.as_bytes();
    let mut result = Vec::with_capacity(bytes.len() + 1);
    // TXT records must be prefixed with a length byte
    result.push(bytes.len() as u8);
    result.extend_from_slice(bytes);
    result
}

fn encode_type_a_string(value: String) -> [u8; 4] {
    value
        .parse::<Ipv4Addr>()
        .unwrap_or_else(|_| panic!("Invalid IPv4 address: {}", value))
        .octets()
}

fn encode_type_aaaa_string(value: String) -> [u8; 16] {
    value
        .parse::<Ipv6Addr>()
        .unwrap_or_else(|_| panic!("Invalid IPv6 address: {}", value))
        .octets()
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr};

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
            4,
            Ipv4Addr::new(192, 168, 0, 1).to_string(),
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
        let text = "192.168.0.1".to_string();
        let encoded = encode_type_a_string(text);

        // Expected: length byte (11) followed by the text
        let expected = vec![192, 168, 0, 1];

        assert_eq!(expected, encoded);
    }

    #[test]
    fn encode_type_txt_string_with_simple_text() {
        let text = "hello world".to_string();
        let encoded = encode_type_txt_string(text);

        // Expected: length byte (11) followed by the text
        let expected = vec![
            11, b'h', b'e', b'l', b'l', b'o', b' ', b'w', b'o', b'r', b'l', b'd',
        ];

        assert_eq!(expected, encoded);
    }

    #[test]
    fn encode_type_txt_string_empty() {
        let text = String::new();
        let encoded = encode_type_txt_string(text);

        // Expected: just a length byte of 0
        let expected = vec![0];

        assert_eq!(expected, encoded);
    }

    #[test]
    fn encode_type_txt_string_max_length() {
        // TXT records can have up to 255 characters per string
        let text = "a".repeat(255);
        let encoded = encode_type_txt_string(text);

        // Expected: length byte (255) followed by 255 'a' characters
        assert_eq!(256, encoded.len());
        assert_eq!(255, encoded[0]);
        assert!(encoded[1..].iter().all(|&b| b == b'a'));
    }

    #[test]
    fn encode_txt_resource_record() {
        let text_content = "some content for google.com";
        let rr = ResourceRecord::new(
            DomainName::from("google.com"),
            Type::TXT,
            Class::IN,
            3600,
            (text_content.len() + 1) as u16, // +1 for length byte
            text_content.to_string(),
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
    fn encode_type_aaaa_string_compressed() {
        let ipv6_str = "2001:db8::1".to_string();
        let encoded = encode_type_aaaa_string(ipv6_str);

        let expected = [
            0x20, 0x01, 0x0d, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01,
        ];

        assert_eq!(expected, encoded);
    }

    #[test]
    fn encode_type_aaaa_string_loopback() {
        let ipv6_str = "::1".to_string();
        let encoded = encode_type_aaaa_string(ipv6_str);

        let expected = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01,
        ];

        assert_eq!(expected, encoded);
    }

    #[test]
    fn encode_type_aaaa_string_full() {
        let ipv6_str = "2607:f8b0:4004:c07::71".to_string();
        let encoded = encode_type_aaaa_string(ipv6_str);

        let expected = [
            0x26, 0x07, 0xf8, 0xb0, 0x40, 0x04, 0x0c, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x71,
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
            16,
            "2001:db8::1".to_string(),
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
    #[should_panic(expected = "Invalid IPv6 address")]
    fn encode_type_aaaa_string_invalid() {
        encode_type_aaaa_string("not an ipv6 address".to_string());
    }
}
