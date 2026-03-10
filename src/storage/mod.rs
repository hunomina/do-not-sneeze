use std::{collections::HashMap, sync::RwLock};

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
        &self,
        question: &Question,
    ) -> Result<Vec<ResourceRecord>, RepositoryError>;
}

pub struct InMemoryResourceRecordRepository {
    inner: RwLock<HashMap<DomainName, Vec<ResourceRecord>>>,
}

impl InMemoryResourceRecordRepository {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }
}

impl InMemoryResourceRecordRepository {
    // todo: what about authoritative answers and additional answers?
    pub fn save(&self, resource_record: ResourceRecord) {
        let mut inner = self.inner.write().unwrap();
        let entry = inner.entry(resource_record.name.clone()).or_default();
        entry.push(resource_record);
    }
}

impl ResourceRecordRepository for InMemoryResourceRecordRepository {
    // todo: deal with TTLs
    fn get_resource_records(
        &self,
        question: &Question,
    ) -> Result<Vec<ResourceRecord>, RepositoryError> {
        let inner = self.inner.read().unwrap();
        let entries_for_domain_name = inner.get(&question.name);

        if entries_for_domain_name.is_none() {
            return Ok(vec![]);
        }

        Ok(entries_for_domain_name
            .unwrap()
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
