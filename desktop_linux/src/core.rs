use synap_core::{dto::NoteDTO, error::ServiceError, service::SynapService};

pub type CoreResult<T> = Result<T, ServiceError>;

pub trait DesktopCore {
    fn recent_notes(&self, cursor: Option<&str>, limit: Option<usize>) -> CoreResult<Vec<NoteDTO>>;
    fn deleted_notes(&self, cursor: Option<&str>, limit: Option<usize>)
        -> CoreResult<Vec<NoteDTO>>;
    fn search(&self, query: &str, limit: usize) -> CoreResult<Vec<NoteDTO>>;

    fn get_note(&self, id: &str) -> CoreResult<NoteDTO>;
    fn replies(
        &self,
        parent_id: &str,
        cursor: Option<String>,
        limit: usize,
    ) -> CoreResult<Vec<NoteDTO>>;
    fn origins(&self, note_id: &str) -> CoreResult<Vec<NoteDTO>>;
    fn other_versions(&self, note_id: &str) -> CoreResult<Vec<NoteDTO>>;

    fn create_note(&self, content: String, tags: Vec<String>) -> CoreResult<NoteDTO>;
    fn reply_note(
        &self,
        parent_id: &str,
        content: String,
        tags: Vec<String>,
    ) -> CoreResult<NoteDTO>;
    fn edit_note(&self, note_id: &str, content: String, tags: Vec<String>) -> CoreResult<NoteDTO>;
    fn delete_note(&self, note_id: &str) -> CoreResult<()>;
    fn restore_note(&self, note_id: &str) -> CoreResult<()>;
}

pub struct SynapCoreAdapter {
    service: SynapService,
}

impl SynapCoreAdapter {
    pub fn new_from_env() -> CoreResult<Self> {
        let db_path =
            std::env::var("SYNAP_DESKTOP_DB").unwrap_or_else(|_| "synap-desktop.redb".to_string());
        let service = SynapService::new(Some(db_path))?;
        Ok(Self { service })
    }
}

impl DesktopCore for SynapCoreAdapter {
    fn recent_notes(&self, cursor: Option<&str>, limit: Option<usize>) -> CoreResult<Vec<NoteDTO>> {
        self.service.get_recent_note(cursor, limit)
    }

    fn deleted_notes(
        &self,
        cursor: Option<&str>,
        limit: Option<usize>,
    ) -> CoreResult<Vec<NoteDTO>> {
        self.service.get_deleted_notes(cursor, limit)
    }

    fn search(&self, query: &str, limit: usize) -> CoreResult<Vec<NoteDTO>> {
        self.service.search(query, limit)
    }

    fn get_note(&self, id: &str) -> CoreResult<NoteDTO> {
        self.service.get_note(id)
    }

    fn replies(
        &self,
        parent_id: &str,
        cursor: Option<String>,
        limit: usize,
    ) -> CoreResult<Vec<NoteDTO>> {
        self.service.get_replies(parent_id, cursor, limit)
    }

    fn origins(&self, note_id: &str) -> CoreResult<Vec<NoteDTO>> {
        self.service.get_origins(note_id)
    }

    fn other_versions(&self, note_id: &str) -> CoreResult<Vec<NoteDTO>> {
        self.service.get_other_versions(note_id)
    }

    fn create_note(&self, content: String, tags: Vec<String>) -> CoreResult<NoteDTO> {
        self.service.create_note(content, tags)
    }

    fn reply_note(
        &self,
        parent_id: &str,
        content: String,
        tags: Vec<String>,
    ) -> CoreResult<NoteDTO> {
        self.service.reply_note(parent_id, content, tags)
    }

    fn edit_note(&self, note_id: &str, content: String, tags: Vec<String>) -> CoreResult<NoteDTO> {
        self.service.edit_note(note_id, content, tags)
    }

    fn delete_note(&self, note_id: &str) -> CoreResult<()> {
        self.service.delete_note(note_id)
    }

    fn restore_note(&self, note_id: &str) -> CoreResult<()> {
        self.service.restore_note(note_id)
    }
}
