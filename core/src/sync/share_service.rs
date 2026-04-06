use std::time::Instant;

use uuid::Uuid;

use crate::{
    error::ServiceError,
    models::note::{Note, NoteRecord},
    service::SynapService,
    sync::share::{SharePackage, ShareStats},
};

pub struct ShareService<'a> {
    core: &'a SynapService,
}

impl<'a> ShareService<'a> {
    pub fn new(core: &'a SynapService) -> Self {
        Self { core }
    }

    pub fn export_bytes(&self, note_ids: &[Uuid]) -> Result<Vec<u8>, ServiceError> {
        let records = self.export_records(note_ids)?;
        SharePackage::new(records)
            .encode()
            .map_err(|err| ServiceError::ShareProtocol(err.to_string()))
    }

    pub fn import_bytes(&self, bytes: &[u8]) -> Result<ShareStats, ServiceError> {
        let started = Instant::now();
        let package = SharePackage::decode(bytes)
            .map_err(|err| ServiceError::ShareProtocol(err.to_string()))?;
        let record_count = package.records.len();
        let applied = self.apply_records(package.records)?;

        Ok(ShareStats {
            records: record_count,
            applied,
            bytes: bytes.len(),
            duration_ms: started.elapsed().as_millis() as u64,
        })
    }

    pub(crate) fn export_records(
        &self,
        note_ids: &[Uuid],
    ) -> Result<Vec<NoteRecord>, ServiceError> {
        self.core
            .with_read(|_tx, reader| reader.export_records(note_ids).map_err(Into::into))
    }

    fn apply_records(&self, records: Vec<NoteRecord>) -> Result<usize, ServiceError> {
        let applied = self
            .core
            .with_write(|tx| Note::import_records(tx, records).map_err(Into::into))?;

        if applied > 0 {
            self.core.refresh_search_indexes()?;
        }

        Ok(applied)
    }
}
