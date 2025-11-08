use std::net::ToSocketAddrs;

use crate::{
    common::resource_record::ResourceRecord,
    decoder::Decoder,
    encoder::Encoder,
    storage::{
        InMemoryResourceRecordRepository, RepositoryError, ResourceRecordRepository,
        fallback::FallbackRepository,
    },
};

pub struct CombinedRepository<T: ToSocketAddrs + Clone, D: Decoder, E: Encoder> {
    in_memory_repository: InMemoryResourceRecordRepository,
    fallback_repository: FallbackRepository<T, D, E>,
}

impl<T: ToSocketAddrs + Clone, D: Decoder, E: Encoder> CombinedRepository<T, D, E> {
    pub fn new(
        in_memory_repository: InMemoryResourceRecordRepository,
        fallback_repository: FallbackRepository<T, D, E>,
    ) -> Self {
        Self {
            in_memory_repository,
            fallback_repository,
        }
    }
}

impl<T: ToSocketAddrs + Clone, D: Decoder, E: Encoder> ResourceRecordRepository
    for CombinedRepository<T, D, E>
{
    fn get_resource_records(
        &mut self,
        question: crate::common::question::Question,
    ) -> Result<Vec<ResourceRecord>, RepositoryError> {
        let in_memory_records = self
            .in_memory_repository
            .get_resource_records(question.clone())?;

        if !in_memory_records.is_empty() {
            println!(
                "💾 Found records in in-memory repository: {:?}",
                in_memory_records.len()
            );
            return Ok(in_memory_records);
        }

        let fallback_repository_records =
            self.fallback_repository.get_resource_records(question)?;

        println!(
            "🔍 Found records in fallback repository: {:?}",
            fallback_repository_records.len()
        );

        for record in fallback_repository_records.iter() {
            self.in_memory_repository.save(record.clone());
        }

        Ok(fallback_repository_records)
    }
}
