use crate::common::{
    domain_name::DomainName,
    question::{Class as QuestionClass, Question, Type as QuestionType},
    resource_record::{ResourceRecord, Type},
    Message,
};
use crate::decoder::{Decoder, MessageDecoder};
use crate::transport::UDP_MAX_MESSAGE_SIZE;
use std::collections::HashMap;
use std::net::{ToSocketAddrs, UdpSocket};

pub enum RepositoryError {}

pub trait ResourceRecordRepository {
    fn get_resource_records<T: IntoResourceRecordRepositoryQuery>(
        &self,
        query: T,
    ) -> Result<Vec<&ResourceRecord>, RepositoryError>;
    //    fn add_record_record(resource_record: ResourceRecord);
}

struct ExternalRepository {
    in_memory_repository: InMemoryResourceRecordRepository,
    fallback_server: UdpSocket,
    decoder: MessageDecoder, // todo: replace by trait
}

impl ExternalRepository {
    fn new<T: ToSocketAddrs>(fallback_server_address: T) -> Self {
        Self {
            in_memory_repository: InMemoryResourceRecordRepository::new(),
            fallback_server: UdpSocket::bind(fallback_server_address).unwrap(),
            decoder: MessageDecoder {},
        }
    }

    fn fetch_from_other_server(&self, message: Message) -> Vec<&ResourceRecord> {
        let mut buf = [0; UDP_MAX_MESSAGE_SIZE / 8];
        //self.fallback_server.send(request).unwrap();

        match self.fallback_server.recv_from(&mut buf) {
            Ok((amt, _)) => {
                println!("amt {:?}", amt);
                println!("buf {:?}", &buf[..amt]);
                let _ = self.decoder.decode(&buf[..amt]);
            }
            Err(e) => {
                panic!("couldn't receive a datagram: {}", e);
            }
        }

        vec![]
    }
}

impl ResourceRecordRepository for ExternalRepository {
    fn get_resource_records<T: IntoResourceRecordRepositoryQuery>(
        &self,
        query: T,
    ) -> Result<Vec<&ResourceRecord>, RepositoryError> {
        let records = self.in_memory_repository.get_resource_records(query)?;

        if !records.is_empty() {
            return Ok(records);
        }

        //self.fetch_from_other_server(query);
        // todo: add to cache
        Ok(vec![])
    }
}

#[derive(Clone)]
pub struct InMemoryResourceRecordRepository {
    inner: HashMap<DomainName, Vec<ResourceRecord>>,
}

impl InMemoryResourceRecordRepository {
    pub fn new() -> Self {
        let google = ResourceRecord::new(
            DomainName::from("google.com"),
            Type::A,
            QuestionClass::IN,
            3600,
            14,
            "74.125.193.101".to_string(),
        );
        Self {
            inner: HashMap::from([(google.name.clone(), vec![google])]),
        }
    }
}

impl ResourceRecordRepository for InMemoryResourceRecordRepository {
    fn get_resource_records<T: IntoResourceRecordRepositoryQuery>(
        &self,
        query: T,
    ) -> Result<Vec<&ResourceRecord>, RepositoryError> {
        let query = query.into_query();
        let entries_for_domain_name = self.inner.get(&query.name);

        if entries_for_domain_name.is_none() {
            return Ok(vec![]);
        }

        Ok(entries_for_domain_name
            .unwrap()
            .iter()
            .filter(|record| {
                query.name == record.name
                    && match query.class {
                        QuestionClass::ALL => true,
                        _ => query.class == record.class,
                    }
                    && match query.type_ {
                        QuestionType::ALL => true,
                        QuestionType::MAILA => record.type_ == Type::MX,
                        QuestionType::MAILB => {
                            record.type_ == Type::MB_EXP
                                || record.type_ == Type::MG_EXP
                                || record.type_ == Type::MR_EXP
                        }
                        QuestionType::AXFR => unimplemented!("AXFR not implemented"), // no idea how to handle that
                        QuestionType::RRType(t) => record.type_ == t,
                    }
            })
            .collect())
    }
}

#[derive(Debug, Clone)]
pub struct ResourceRecordRepositoryQuery {
    name: DomainName,
    type_: QuestionType,
    class: QuestionClass,
}

pub trait IntoResourceRecordRepositoryQuery {
    fn into_query(&self) -> ResourceRecordRepositoryQuery;
}

impl IntoResourceRecordRepositoryQuery for Question {
    fn into_query(&self) -> ResourceRecordRepositoryQuery {
        ResourceRecordRepositoryQuery {
            name: self.name.clone(),
            type_: self.type_.clone(),
            class: self.class.clone(),
        }
    }
}
