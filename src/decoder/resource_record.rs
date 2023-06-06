use crate::{
    common::{
        question::Class,
        resource_record::{ResourceRecord, Type},
    },
    resource_data::get_data_from_type_and_buffer,
    utils::{extract_next_sixteen_bits_from_buffer, extract_next_thirty_two_bits_from_buffer},
};

use super::domain_name::decode as decode_domain_name;

pub fn decode<'a>(buffer: &'a [u8], source: &'a [u8]) -> (ResourceRecord, &'a [u8]) {
    let (name, buffer) = decode_domain_name(buffer, source);

    let (type_, buffer) = extract_next_sixteen_bits_from_buffer(buffer);
    let type_ = Type::from(type_);

    let (class, buffer) = extract_next_sixteen_bits_from_buffer(buffer);
    let class = Class::from(class);

    let (ttl, buffer) = extract_next_thirty_two_bits_from_buffer(buffer);

    let (resource_data_length, buffer) = extract_next_sixteen_bits_from_buffer(buffer);

    let (data, buffer) = buffer.split_at(resource_data_length as usize);

    let resource_data = get_data_from_type_and_buffer(type_, data);

    (
        ResourceRecord::new(name, type_, class, ttl, resource_data_length, resource_data),
        buffer,
    )
}
