/// EDNS(0) OPT pseudo-RR for DNS extension mechanism
/// OPT is a special record type (41) that carries control information
/// and does not represent actual DNS data.
#[derive(Debug, PartialEq, Clone)]
pub struct OptRecord {
    pub udp_payload_size: u16,
    pub extended_rcode: u8,
    pub version: u8,
    pub dnssec_ok: bool,
    pub options: Vec<EdnsOption>,
}

impl OptRecord {
    pub fn new(
        udp_payload_size: u16,
        extended_rcode: u8,
        version: u8,
        dnssec_ok: bool,
        options: Vec<EdnsOption>,
    ) -> Self {
        Self {
            udp_payload_size,
            extended_rcode,
            version,
            dnssec_ok,
            options,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct EdnsOption {
    pub code: u16,
    pub data: Vec<u8>,
}

impl EdnsOption {
    pub fn new(code: u16, data: Vec<u8>) -> Self {
        Self { code, data }
    }
}
