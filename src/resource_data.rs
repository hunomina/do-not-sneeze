use std::net::Ipv4Addr;

use crate::common::resource_record::Type;

pub fn get_data_from_type_and_buffer(type_: Type, buffer: &[u8]) -> String {
    match type_ {
        Type::A => get_type_A_data(buffer).to_string(),
        _ => "Unknown RR type: {:?}".into(),
    }
}

fn get_type_A_data(buffer: &[u8]) -> Ipv4Addr {
    assert!(buffer.len() == 4);
    Ipv4Addr::from([buffer[0], buffer[1], buffer[2], buffer[3]])
}
