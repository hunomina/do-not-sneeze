// see: https://datatracker.ietf.org/doc/html/rfc1035#section-4.2

/*
    TCP:
      - two byte prefixed to specify the message size (excluded from the count)
*/

pub const DNS_PORT: u16 = 53;

pub const UDP_MAX_MESSAGE_SIZE: usize = 512;
