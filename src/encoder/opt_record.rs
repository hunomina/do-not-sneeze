use crate::{
    common::{
        opt_record::{EdnsOption, OptRecord},
        resource_record::Type,
    },
    utils::push_u16_to_u8_vec,
};

pub fn encode(opt: &OptRecord) -> Vec<u8> {
    let mut r = vec![];

    // NAME: must be 0 (root domain)
    r.push(0);

    // TYPE: 41 (OPT)
    push_u16_to_u8_vec(&mut r, Type::OPT.into());

    // CLASS: UDP payload size
    push_u16_to_u8_vec(&mut r, opt.udp_payload_size);

    // TTL: extended_rcode + version + flags
    let ttl = encode_ttl(opt);
    r.push((ttl >> 24) as u8);
    r.push((ttl >> 16) as u8);
    r.push((ttl >> 8) as u8);
    r.push(ttl as u8);

    // Encode options
    let options_data = encode_options(&opt.options);

    // RDLEN
    push_u16_to_u8_vec(&mut r, options_data.len() as u16);

    // RDATA
    r.extend(options_data);

    r
}

fn encode_ttl(opt: &OptRecord) -> u32 {
    let flags = if opt.dnssec_ok { 0x8000u16 } else { 0u16 };

    ((opt.extended_rcode as u32) << 24) | ((opt.version as u32) << 16) | (flags as u32)
}

fn encode_options(options: &[EdnsOption]) -> Vec<u8> {
    let mut data = vec![];

    for option in options {
        // Option code
        push_u16_to_u8_vec(&mut data, option.code);

        // Option length
        push_u16_to_u8_vec(&mut data, option.data.len() as u16);

        // Option data
        data.extend(&option.data);
    }

    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_opt_record_basic() {
        let opt = OptRecord::new(4096, 0, 0, false, vec![]);
        let encoded = encode(&opt);

        let expected = vec![
            0, // NAME: root (0)
            0, 41, // TYPE: OPT (41)
            0x10, 0x00, // CLASS: UDP payload size 4096
            0x00, 0x00, 0x00, 0x00, // TTL: extended_rcode=0, version=0, flags=0
            0x00, 0x00, // RDLEN: 0 (no options)
        ];

        assert_eq!(expected, encoded);
    }

    #[test]
    fn encode_opt_record_with_dnssec() {
        let opt = OptRecord::new(4096, 0, 0, true, vec![]);
        let encoded = encode(&opt);

        let expected = vec![
            0, // NAME: root (0)
            0, 41, // TYPE: OPT (41)
            0x10, 0x00, // CLASS: UDP payload size 4096
            0x00, 0x00, 0x80, 0x00, // TTL: DO flag set (bit 15)
            0x00, 0x00, // RDLEN: 0
        ];

        assert_eq!(expected, encoded);
    }

    #[test]
    fn encode_opt_record_with_option() {
        let option = EdnsOption::new(10, vec![0xAB, 0xCD]);
        let opt = OptRecord::new(4096, 0, 0, false, vec![option]);
        let encoded = encode(&opt);

        let expected = vec![
            0, // NAME: root (0)
            0, 41, // TYPE: OPT (41)
            0x10, 0x00, // CLASS: UDP payload size 4096
            0x00, 0x00, 0x00, 0x00, // TTL
            0x00, 0x06, // RDLEN: 6 bytes
            0x00, 0x0A, // Option code: 10
            0x00, 0x02, // Option length: 2
            0xAB, 0xCD, // Option data
        ];

        assert_eq!(expected, encoded);
    }

    #[test]
    fn test_opt_record_encode_ttl_no_dnssec() {
        let opt = OptRecord::new(4096, 0, 0, false, vec![]);
        assert_eq!(encode_ttl(&opt), 0x00000000);
    }

    #[test]
    fn test_opt_record_encode_ttl_with_dnssec() {
        let opt = OptRecord::new(4096, 0, 0, true, vec![]);
        assert_eq!(encode_ttl(&opt), 0x00008000);
    }

    #[test]
    fn test_opt_record_encode_ttl_with_extended_rcode() {
        let opt = OptRecord::new(4096, 0x12, 0, false, vec![]);
        assert_eq!(encode_ttl(&opt), 0x12000000);
    }

    #[test]
    fn test_opt_record_encode_ttl_full() {
        let opt = OptRecord::new(4096, 0xAB, 0, true, vec![]);
        assert_eq!(encode_ttl(&opt), 0xAB008000);
    }
}
