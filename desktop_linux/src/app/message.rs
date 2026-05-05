use crate::domain::{ContentView, NoteDetailData, NoteLayout, Theme};
use synap_core::{dto::NoteDTO, error::ServiceError};

#[derive(Debug)]
pub enum AppMsg {
    Navigate(ContentView),
    SearchChanged(String),
    LayoutChanged(NoteLayout),
    DeleteNote,
    SaveNote {
        id: Option<String>,
        content: String,
        tags: Vec<String>,
    },
    SaveReply {
        parent_id: String,
        content: String,
        tags: Vec<String>,
    },
    CreateNote,
    EditNote,
    ReplyToNote,
    ThemeChanged(Theme),
    NoteSelected(u32),
    NoteActivated(u32),
    NoteDetailLoaded(Result<NoteDetailData, ServiceError>),
    OpenNoteDetail(String),
    LoadMoreNotes,
    MoreNotesLoaded(Result<(Vec<NoteDTO>, Option<String>, bool), ServiceError>),
    TagSelected(String),
    TagsLoaded(Result<Vec<String>, ServiceError>),
    TagNotesLoaded(Result<Vec<NoteDTO>, ServiceError>),
    TagSuggestionsLoaded(Result<Vec<String>, ServiceError>),
    ClearFilters,
    TimelineLoaded(Result<Vec<synap_core::dto::TimelineSessionDTO>, ServiceError>),
}
