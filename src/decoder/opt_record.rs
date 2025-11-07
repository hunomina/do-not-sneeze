use crate::{
    common::opt_record::{EdnsOption, OptRecord},
    decoder::DecodingError,
    utils::extract_next_sixteen_bits_from_buffer,
};

pub fn decode(buffer: &[u8]) -> Result<(OptRecord, &[u8]), DecodingError> {
    // record name must always be 0 (root domain) for OPT
    if buffer[0] != 0 {
        return Err(DecodingError::InvalidOptRecord(
            "OPT record NAME must be 0 (root)".to_string(),
        ));
    }
    let buffer = &buffer[1..];

    // TYPE field is always 41 for OPT, so we can skip it
    let (_, buffer) = extract_next_sixteen_bits_from_buffer(buffer);

    // CLASS field is UDP payload size
    let (udp_payload_size, buffer) = extract_next_sixteen_bits_from_buffer(buffer);

    // TTL field contains: extended_rcode (8 bits) + version (8 bits) + flags (16 bits)
    let (ttl_high, buffer) = extract_next_sixteen_bits_from_buffer(buffer);
    let (ttl_low, buffer) = extract_next_sixteen_bits_from_buffer(buffer);
    let ttl = ((ttl_high as u32) << 16) | (ttl_low as u32);

    let (extended_rcode, version, dnssec_ok) = decode_ttl(ttl);

    // RDLEN
    let (rdlen, buffer) = extract_next_sixteen_bits_from_buffer(buffer);

    // RDATA contains options
    let (rdata, buffer) = buffer.split_at(rdlen as usize);

    let options = decode_options(rdata)?;

    Ok((
        OptRecord::new(
            udp_payload_size,
            extended_rcode,
            version,
            dnssec_ok,
            options,
        ),
        buffer,
    ))
}

fn decode_ttl(ttl: u32) -> (u8, u8, bool) {
    let extended_rcode = (ttl >> 24) as u8;
    let version = ((ttl >> 16) & 0xFF) as u8;
    let flags = (ttl & 0xFFFF) as u16;
    let dnssec_ok = (flags & 0x8000) != 0;

    (extended_rcode, version, dnssec_ok)
}

fn decode_options(mut buffer: &[u8]) -> Result<Vec<EdnsOption>, DecodingError> {
    let mut options = Vec::new();

    while !buffer.is_empty() {
        if buffer.len() < 4 {
            return Err(DecodingError::InvalidOptRecord(
                "Malformed EDNS option".to_string(),
            ));
        }

        let (code, rest) = extract_next_sixteen_bits_from_buffer(buffer);
        let (length, rest) = extract_next_sixteen_bits_from_buffer(rest);

        if rest.len() < length as usize {
            return Err(DecodingError::InvalidOptRecord(
                "EDNS option length exceeds buffer".to_string(),
            ));
        }

        let (data, rest) = rest.split_at(length as usize);
        options.push(EdnsOption::new(code, data.to_vec()));

        buffer = rest;
    }

    Ok(options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_opt_record_basic() {
        // OPT record with no options
        let buffer = [
            0, // NAME: root (0)
            0, 41, // TYPE: OPT (41)
            0x10, 0x00, // CLASS: UDP payload size 4096
            0x00, 0x00, 0x00, 0x00, // TTL: extended_rcode=0, version=0, flags=0
            0x00, 0x00, // RDLEN: 0 (no options)
        ];

        let (opt, remaining) = decode(&buffer).unwrap();

        assert_eq!(opt.udp_payload_size, 4096);
        assert_eq!(opt.extended_rcode, 0);
        assert_eq!(opt.version, 0);
        assert_eq!(opt.dnssec_ok, false);
        assert!(opt.options.is_empty());
        assert!(remaining.is_empty());
    }

    #[test]
    fn decode_opt_record_with_dnssec() {
        // OPT record with DNSSEC OK flag set
        let buffer = [
            0, // NAME: root (0)
            0, 41, // TYPE: OPT (41)
            0x10, 0x00, // CLASS: UDP payload size 4096
            0x00, 0x00, 0x80, 0x00, // TTL: DO flag set (bit 15)
            0x00, 0x00, // RDLEN: 0
        ];

        let (opt, _) = decode(&buffer).unwrap();

        assert_eq!(opt.dnssec_ok, true);
    }

    #[test]
    fn decode_opt_record_with_option() {
        // OPT record with one option (code 10, length 2, data [0xAB, 0xCD])
        let buffer = [
            0, // NAME: root (0)
            0, 41, // TYPE: OPT (41)
            0x10, 0x00, // CLASS: UDP payload size 4096
            0x00, 0x00, 0x00, 0x00, // TTL
            0x00, 0x06, // RDLEN: 6 bytes (4 for option header + 2 for data)
            0x00, 0x0A, // Option code: 10
            0x00, 0x02, // Option length: 2
            0xAB, 0xCD, // Option data
        ];

        let (opt, _) = decode(&buffer).unwrap();

        assert_eq!(opt.options.len(), 1);
        assert_eq!(opt.options[0].code, 10);
        assert_eq!(opt.options[0].data, vec![0xAB, 0xCD]);
    }

    #[test]
    fn decode_opt_record_invalid_name() {
        // OPT record with non-zero NAME (invalid)
        let buffer = [
            3, b'c', b'o', b'm', 0, // NAME: com. (invalid for OPT)
            0, 41, // TYPE: OPT (41)
            0x10, 0x00, // CLASS
            0x00, 0x00, 0x00, 0x00, // TTL
            0x00, 0x00, // RDLEN
        ];

        let result = decode(&buffer);
        assert!(matches!(result, Err(DecodingError::InvalidOptRecord(_))));
    }

    #[test]
    fn test_opt_record_decode_ttl() {
        let (extended_rcode, version, dnssec_ok) = decode_ttl(0xAB008000);
        assert_eq!(extended_rcode, 0xAB);
        assert_eq!(version, 0);
        assert_eq!(dnssec_ok, true);
    }

    #[test]
    fn test_opt_record_decode_ttl_no_dnssec() {
        let (extended_rcode, version, dnssec_ok) = decode_ttl(0x12000000);
        assert_eq!(extended_rcode, 0x12);
        assert_eq!(version, 0);
        assert_eq!(dnssec_ok, false);
    }
}
