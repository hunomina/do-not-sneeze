// https://datatracker.ietf.org/doc/html/rfc1035

use decoder::MessageDecoder;
use encoder::MessageEncoder;
use server::Server;

use crate::{
    common::{
        domain_name::DomainName,
        resource_record::{ResourceRecord, Type},
    },
    storage::{InMemoryResourceRecordRepository, fallback::FallbackRepository},
};

mod common;
mod decoder;
mod encoder;
mod server;
mod storage;
mod transport;
mod utils;

fn main() {
    let mut in_memory_repository = InMemoryResourceRecordRepository::new();

    in_memory_repository.save(ResourceRecord::new(
        DomainName::from("google.com"),
        Type::A,
        common::question::Class::IN,
        3600,
        vec![74, 125, 193, 101], // Google's IPv4 address
    ));

    let text_content = "some content for google.com".as_bytes().to_vec();
    in_memory_repository.save(ResourceRecord::new(
        DomainName::from("google.com"),
        Type::TXT,
        common::question::Class::IN,
        3600,
        text_content,
    ));

    in_memory_repository.save(ResourceRecord::new(
        DomainName::from("google.com"),
        Type::AAAA,
        common::question::Class::IN,
        3600,
        "2607:f8b0:4004:c07::71"
            .parse::<std::net::Ipv6Addr>()
            .unwrap()
            .octets()
            .to_vec(), // Google IPv6 address
    ));

    let fallback_repository = FallbackRepository {
        fallback_server_address: "8.8.8.8:53",
        decoder: MessageDecoder {},
        encoder: MessageEncoder {},
    };
    let storage =
        storage::combined::CombinedRepository::new(in_memory_repository, fallback_repository);

    Server::new(MessageDecoder {}, MessageEncoder {}, storage).run();
}
