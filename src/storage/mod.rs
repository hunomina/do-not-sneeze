use std::collections::HashMap;

use crate::{
    common::{
        domain_name::DomainName,
        question::{Class as QuestionClass, Question, Type as QuestionType},
        resource_record::{ResourceRecord, Type},
    },
    decoder::DecodingError,
};

pub mod combined;
pub mod fallback;

#[derive(Debug)]
pub enum RepositoryError {
    ContactingFallbackServerError(String),
    DecodingFallbackServerResponseError(DecodingError),
}

pub trait ResourceRecordRepository {
    fn get_resource_records(
        &mut self,
        question: Question,
    ) -> Result<Vec<ResourceRecord>, RepositoryError>;
}

#[derive(Clone)]
pub struct InMemoryResourceRecordRepository {
    inner: HashMap<DomainName, Vec<ResourceRecord>>,
}

impl InMemoryResourceRecordRepository {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }
}

impl InMemoryResourceRecordRepository {
    // todo: what about authoritative answers and additional answers?
    pub fn save(&mut self, resource_record: ResourceRecord) {
        let entry = self.inner.entry(resource_record.name.clone()).or_default();
        entry.push(resource_record);
    }
}

impl ResourceRecordRepository for InMemoryResourceRecordRepository {
    // todo: deal with TTLs
    fn get_resource_records(
        &mut self,
        question: Question,
    ) -> Result<Vec<ResourceRecord>, RepositoryError> {
        let entries_for_domain_name = self.inner.get(&question.name);

        Ok(entries_for_domain_name
            .unwrap_or(&vec![])
            .iter()
            .filter(|record| {
                question.name == record.name
                    && match question.class {
                        QuestionClass::ALL => true,
                        _ => question.class == record.class,
                    }
                    && match question.type_ {
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
            .cloned()
            .collect())
    }
}
